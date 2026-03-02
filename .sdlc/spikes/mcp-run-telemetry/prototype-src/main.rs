/// Prototype: MCP Run Telemetry with redb
///
/// Proves:
/// 1. redb can store structured run events keyed by run_id
/// 2. We can capture the stream events we currently discard (ToolResult, TaskNotification)
/// 3. We can query all events for a run_id efficiently
/// 4. The API surface is simple enough to drop into sdlc-server

use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ─── Event types matching claude-agent's message stream ───────────────────

/// A captured telemetry event — maps to one message from the claude stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub seq: u64,
    pub run_id: String,
    pub timestamp: String,
    pub event_type: TelemetryEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TelemetryEventType {
    /// System init — model, MCP servers, tools available
    Init {
        model: String,
        tools_count: usize,
        mcp_servers: Vec<String>,
    },
    /// Tool call from the assistant — CURRENTLY CAPTURED
    ToolCall {
        tool_use_id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Tool result fed back to model — CURRENTLY DISCARDED
    ToolResult {
        tool_use_id: String,
        is_error: bool,
        content: String, // extracted text content
    },
    /// Assistant text output
    AssistantText {
        text: String,
    },
    /// Subagent started (Task tool spawn) — CURRENTLY DISCARDED
    SubagentStarted {
        task_id: String,
        tool_use_id: Option<String>,
        description: String,
    },
    /// Subagent progress update — CURRENTLY DISCARDED
    SubagentProgress {
        task_id: String,
        last_tool_name: Option<String>,
        total_tokens: u64,
        tool_uses: u64,
        duration_ms: u64,
    },
    /// Subagent completed — CURRENTLY DISCARDED
    SubagentCompleted {
        task_id: String,
        status: String,   // "success" | "error"
        summary: String,
        total_tokens: Option<u64>,
        duration_ms: Option<u64>,
    },
    /// Run finished with cost and turn data
    RunResult {
        is_error: bool,
        cost_usd: f64,
        turns: u32,
        duration_ms: u64,
    },
}

// ─── redb table definition ─────────────────────────────────────────────────
//
// Table: EVENTS
//   Key:   (run_id: &str, seq: u64) — composite, lexicographically ordered
//   Value: JSON-serialized TelemetryEvent
//
// This layout lets us:
//   - Efficiently scan all events for a run: range (run_id, 0)..(run_id+1, 0)
//   - Insert in O(log n) per event
//   - No index rebuilding needed — redb handles ordering

const EVENTS: TableDefinition<(&str, u64), &str> = TableDefinition::new("events");

// ─── TelemetryStore ────────────────────────────────────────────────────────

pub struct TelemetryStore {
    db: Database,
}

impl TelemetryStore {
    /// Open or create the telemetry database at the given path.
    pub fn open(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::create(path)?;
        // Ensure the table exists
        let write_txn = db.begin_write()?;
        write_txn.open_table(EVENTS)?;
        write_txn.commit()?;
        Ok(Self { db })
    }

    /// Append a single event for a run. Returns the assigned sequence number.
    pub fn append(
        &self,
        run_id: &str,
        event_type: TelemetryEventType,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let write_txn = self.db.begin_write()?;
        let seq = {
            let table = write_txn.open_table(EVENTS)?;
            // Find highest seq for this run_id by scanning the end of its range
            let prefix_end = next_string(run_id);
            let range = table.range((run_id, 0)..(prefix_end.as_str(), 0))?;
            let last_seq = range
                .rev()
                .next()
                .transpose()?
                .map(|(k, _)| k.value().1)
                .unwrap_or(0);
            last_seq + 1
        };

        let event = TelemetryEvent {
            seq,
            run_id: run_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type,
        };
        let json = serde_json::to_string(&event)?;

        {
            let mut table = write_txn.open_table(EVENTS)?;
            table.insert((run_id, seq), json.as_str())?;
        }
        write_txn.commit()?;
        Ok(seq)
    }

    /// Retrieve all events for a run_id in sequence order.
    pub fn events_for_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<TelemetryEvent>, Box<dyn std::error::Error>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(EVENTS)?;
        let prefix_end = next_string(run_id);
        let range = table.range((run_id, 0)..(prefix_end.as_str(), 0))?;

        let mut events = Vec::new();
        for entry in range {
            let (_, value) = entry?;
            let event: TelemetryEvent = serde_json::from_str(value.value())?;
            events.push(event);
        }
        Ok(events)
    }

    /// Count events by type for a given run — useful for cost/activity summary.
    pub fn summary_for_run(
        &self,
        run_id: &str,
    ) -> Result<RunSummary, Box<dyn std::error::Error>> {
        let events = self.events_for_run(run_id)?;
        let mut summary = RunSummary::default();
        for event in &events {
            match &event.event_type {
                TelemetryEventType::ToolCall { name, .. } => {
                    summary.tool_calls += 1;
                    *summary.tools_used.entry(name.clone()).or_insert(0) += 1;
                }
                TelemetryEventType::ToolResult { is_error, .. } => {
                    if *is_error {
                        summary.tool_errors += 1;
                    }
                }
                TelemetryEventType::SubagentStarted { .. } => {
                    summary.subagents_spawned += 1;
                }
                TelemetryEventType::SubagentCompleted { total_tokens, duration_ms, .. } => {
                    summary.subagent_tokens += total_tokens.unwrap_or(0);
                    summary.subagent_duration_ms += duration_ms.unwrap_or(0);
                }
                TelemetryEventType::RunResult { cost_usd, turns, .. } => {
                    summary.total_cost_usd = *cost_usd;
                    summary.total_turns = *turns;
                }
                _ => {}
            }
        }
        summary.total_events = events.len();
        Ok(summary)
    }

    /// List all distinct run_ids that have events.
    pub fn all_run_ids(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(EVENTS)?;
        let mut ids = Vec::new();
        let mut last_id = String::new();
        for entry in table.iter()? {
            let (key, _) = entry?;
            let run_id = key.value().0.to_string();
            if run_id != last_id {
                ids.push(run_id.clone());
                last_id = run_id;
            }
        }
        Ok(ids)
    }
}

#[derive(Debug, Default)]
pub struct RunSummary {
    pub total_events: usize,
    pub tool_calls: u64,
    pub tool_errors: u64,
    pub tools_used: std::collections::HashMap<String, u64>,
    pub subagents_spawned: u64,
    pub subagent_tokens: u64,
    pub subagent_duration_ms: u64,
    pub total_cost_usd: f64,
    pub total_turns: u32,
}

/// Increment a string lexicographically for range scanning.
/// "abc" → "abd", "abz" → "ac", etc.
fn next_string(s: &str) -> String {
    let mut bytes = s.as_bytes().to_vec();
    for b in bytes.iter_mut().rev() {
        if *b < u8::MAX {
            *b += 1;
            return String::from_utf8(bytes).unwrap();
        }
    }
    // All bytes were 0xFF — append a byte
    bytes.push(0);
    String::from_utf8(bytes).unwrap()
}

// ─── Main: simulate a real run and prove the store works ──────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::path::Path::new("/tmp/spike-mcp-run-telemetry/prototype/telemetry.redb");
    let store = TelemetryStore::open(db_path)?;

    println!("=== MCP Run Telemetry Spike ===");
    println!("Database: {}", db_path.display());
    println!();

    // Simulate two concurrent runs with realistic sdlc agent events
    let run_a = "20260301-120000-aaa";
    let run_b = "20260301-120100-bbb";

    println!("--- Simulating run A: sdlc-run feature-auth ---");

    // 1. Init event
    store.append(run_a, TelemetryEventType::Init {
        model: "claude-sonnet-4-6".to_string(),
        tools_count: 12,
        mcp_servers: vec!["sdlc".to_string()],
    })?;

    // 2. Tool call — Bash
    store.append(run_a, TelemetryEventType::ToolCall {
        tool_use_id: "tu_001".to_string(),
        name: "Bash".to_string(),
        input: serde_json::json!({"command": "sdlc next --for feature-auth --json"}),
    })?;

    // 3. Tool result — Bash output (CURRENTLY DISCARDED in production)
    store.append(run_a, TelemetryEventType::ToolResult {
        tool_use_id: "tu_001".to_string(),
        is_error: false,
        content: r#"{"action":"implement_task","message":"Implement task 3: Add JWT validation"}"#.to_string(),
    })?;

    // 4. Subagent spawned (CURRENTLY DISCARDED in production)
    store.append(run_a, TelemetryEventType::SubagentStarted {
        task_id: "task_abc123".to_string(),
        tool_use_id: Some("tu_002".to_string()),
        description: "Implement JWT validation middleware".to_string(),
    })?;

    // 5. Subagent progress (CURRENTLY DISCARDED)
    store.append(run_a, TelemetryEventType::SubagentProgress {
        task_id: "task_abc123".to_string(),
        last_tool_name: Some("Edit".to_string()),
        total_tokens: 8420,
        tool_uses: 7,
        duration_ms: 12500,
    })?;

    // 6. Subagent completed (CURRENTLY DISCARDED)
    store.append(run_a, TelemetryEventType::SubagentCompleted {
        task_id: "task_abc123".to_string(),
        status: "success".to_string(),
        summary: "Implemented JWT validation in src/auth/middleware.rs with 3 test cases".to_string(),
        total_tokens: Some(11200),
        duration_ms: Some(18300),
    })?;

    // 7. MCP tool call
    store.append(run_a, TelemetryEventType::ToolCall {
        tool_use_id: "tu_003".to_string(),
        name: "mcp__sdlc__sdlc_approve_artifact".to_string(),
        input: serde_json::json!({"slug": "feature-auth", "artifact_type": "tasks"}),
    })?;

    // 8. MCP tool result (CURRENTLY DISCARDED)
    store.append(run_a, TelemetryEventType::ToolResult {
        tool_use_id: "tu_003".to_string(),
        is_error: false,
        content: r#"{"status":"approved","artifact":"tasks"}"#.to_string(),
    })?;

    // 9. Run result
    store.append(run_a, TelemetryEventType::RunResult {
        is_error: false,
        cost_usd: 0.147,
        turns: 8,
        duration_ms: 45200,
    })?;

    println!("Run A stored: 9 events");

    println!();
    println!("--- Simulating run B: sdlc-ponder new-idea ---");

    store.append(run_b, TelemetryEventType::Init {
        model: "claude-sonnet-4-6".to_string(),
        tools_count: 8,
        mcp_servers: vec![],
    })?;
    store.append(run_b, TelemetryEventType::ToolCall {
        tool_use_id: "tu_b01".to_string(),
        name: "Read".to_string(),
        input: serde_json::json!({"file_path": ".sdlc/roadmap/new-idea/manifest.yaml"}),
    })?;
    store.append(run_b, TelemetryEventType::ToolResult {
        tool_use_id: "tu_b01".to_string(),
        is_error: false,
        content: "slug: new-idea\nstatus: in_progress\n".to_string(),
    })?;
    store.append(run_b, TelemetryEventType::RunResult {
        is_error: false,
        cost_usd: 0.023,
        turns: 3,
        duration_ms: 8100,
    })?;

    println!("Run B stored: 4 events");

    // ─── Validation: query and display ────────────────────────────────────

    println!();
    println!("=== Validation: Query run A events ===");
    let events_a = store.events_for_run(run_a)?;
    println!("Retrieved {} events for run A:", events_a.len());
    for e in &events_a {
        let kind = match &e.event_type {
            TelemetryEventType::Init { model, mcp_servers, .. } =>
                format!("Init(model={}, mcp_servers={:?})", model, mcp_servers),
            TelemetryEventType::ToolCall { name, tool_use_id, .. } =>
                format!("ToolCall(name={}, id={})", name, tool_use_id),
            TelemetryEventType::ToolResult { tool_use_id, is_error, content } =>
                format!("ToolResult(id={}, error={}, content={}...)", tool_use_id, is_error, &content[..content.len().min(40)]),
            TelemetryEventType::SubagentStarted { task_id, description, .. } =>
                format!("SubagentStarted(task={}, desc={}...)", task_id, &description[..description.len().min(40)]),
            TelemetryEventType::SubagentProgress { task_id, last_tool_name, total_tokens, .. } =>
                format!("SubagentProgress(task={}, last_tool={:?}, tokens={})", task_id, last_tool_name, total_tokens),
            TelemetryEventType::SubagentCompleted { task_id, status, summary, .. } =>
                format!("SubagentCompleted(task={}, status={}, summary={}...)", task_id, status, &summary[..summary.len().min(40)]),
            TelemetryEventType::RunResult { cost_usd, turns, .. } =>
                format!("RunResult(cost=${:.4}, turns={})", cost_usd, turns),
            TelemetryEventType::AssistantText { text } =>
                format!("AssistantText({}...)", &text[..text.len().min(30)]),
        };
        println!("  [seq={}] {} at {}", e.seq, kind, &e.timestamp[..19]);
    }

    println!();
    println!("=== Validation: Summary for run A ===");
    let summary_a = store.summary_for_run(run_a)?;
    println!("  Total events:      {}", summary_a.total_events);
    println!("  Tool calls:        {}", summary_a.tool_calls);
    println!("  Tool errors:       {}", summary_a.tool_errors);
    println!("  Tools used:        {:?}", summary_a.tools_used);
    println!("  Subagents spawned: {}", summary_a.subagents_spawned);
    println!("  Subagent tokens:   {}", summary_a.subagent_tokens);
    println!("  Subagent duration: {}ms", summary_a.subagent_duration_ms);
    println!("  Total cost:        ${:.4}", summary_a.total_cost_usd);
    println!("  Total turns:       {}", summary_a.total_turns);

    println!();
    println!("=== Validation: All run IDs in store ===");
    let all_ids = store.all_run_ids()?;
    for id in &all_ids {
        let events = store.events_for_run(id)?;
        println!("  {} → {} events", id, events.len());
    }

    println!();
    println!("=== Validation: Isolated query — run B only ===");
    let events_b = store.events_for_run(run_b)?;
    println!("Run B events: {} (should be 4)", events_b.len());
    assert_eq!(events_b.len(), 4, "Run B should have exactly 4 events");
    assert_eq!(all_ids.len(), 2, "Should have exactly 2 runs");

    println!();
    println!("=== Database file size ===");
    let meta = std::fs::metadata(db_path)?;
    println!("  {} bytes ({:.1} KB)", meta.len(), meta.len() as f64 / 1024.0);

    println!();
    println!("✅ All assertions passed. redb works for MCP run telemetry.");

    Ok(())
}
