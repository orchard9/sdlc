use anyhow::Context as _;
use redb::{Database, ReadableTable, TableDefinition};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Table: composite key (run_id, seq) → JSON string
const EVENTS: TableDefinition<(&str, u64), &str> = TableDefinition::new("events");

/// Aggregated stats for a single run.
#[derive(Debug, serde::Serialize)]
pub struct RunSummary {
    pub tool_calls: u64,
    pub tool_errors: u64,
    pub tools_used: HashMap<String, u64>,
    pub subagents_spawned: u64,
    pub subagent_tokens: u64,
    pub total_cost_usd: Option<f64>,
    pub total_turns: Option<u64>,
}

/// `TelemetryStore` persists raw agent events to a `redb` database so they
/// survive server restarts and support per-run queries.
///
/// The composite key `(run_id, seq)` allows O(k log n) prefix-range scans
/// where k is the number of events for a single run.
pub struct TelemetryStore {
    db: Arc<Database>,
    /// In-memory per-run sequence counters. Populated lazily on first write to
    /// a run by scanning the DB for the current max sequence.
    counters: Mutex<HashMap<String, u64>>,
}

impl TelemetryStore {
    /// Open (or create) the `redb` database at `path`.
    /// Returns an error if the database cannot be opened.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let db = Database::create(path).context("open telemetry db")?;
        // Ensure the table exists.
        {
            let wtx = db.begin_write().context("begin write")?;
            wtx.open_table(EVENTS).context("open events table")?;
            wtx.commit().context("commit init")?;
        }
        Ok(Self {
            db: Arc::new(db),
            counters: Mutex::new(HashMap::new()),
        })
    }

    /// Return the next sequence number for `run_id`, initializing from the DB
    /// if this is the first call for this run in the current process.
    fn next_seq(&self, run_id: &str) -> anyhow::Result<u64> {
        let mut counters = self.counters.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(seq) = counters.get_mut(run_id) {
            let current = *seq;
            *seq += 1;
            return Ok(current);
        }
        // Initialize from DB: scan for max existing seq.
        let rtx = self.db.begin_read().context("begin read")?;
        let table = rtx.open_table(EVENTS).context("open events table")?;
        let upper = next_string(run_id);
        let range = (run_id, 0u64)..(upper.as_str(), 0u64);
        let mut max_seq: u64 = 0;
        let mut found = false;
        for entry in table.range(range).context("range scan")? {
            let (key, _) = entry.context("read entry")?;
            let seq = key.value().1;
            if !found || seq >= max_seq {
                max_seq = seq;
                found = true;
            }
        }
        let next = if found { max_seq + 1 } else { 0 };
        counters.insert(run_id.to_string(), next + 1);
        Ok(next)
    }

    /// Append a raw JSON event for `run_id`. Synchronous — call from
    /// `tokio::task::spawn_blocking` in async contexts.
    pub fn append_raw(&self, run_id: &str, event: Value) -> anyhow::Result<()> {
        let seq = self.next_seq(run_id)?;
        let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
        let wtx = self.db.begin_write().context("begin write")?;
        {
            let mut table = wtx.open_table(EVENTS).context("open events table")?;
            table
                .insert((run_id, seq), json.as_str())
                .context("insert event")?;
        }
        wtx.commit().context("commit event")?;
        Ok(())
    }

    /// Delete all events whose run_id is NOT in `keep_ids`.
    ///
    /// Called after `enforce_retention` to keep the redb file from growing
    /// without bound. Safe to call with an empty `keep_ids` — it will delete
    /// everything. Synchronous — call from `tokio::task::spawn_blocking`.
    pub fn prune_runs_not_in(
        &self,
        keep_ids: &std::collections::HashSet<String>,
    ) -> anyhow::Result<()> {
        let wtx = self.db.begin_write().context("begin write for prune")?;
        {
            let mut table = wtx
                .open_table(EVENTS)
                .context("open events table for prune")?;
            // Collect keys to delete first (can't mutate while iterating).
            let to_delete: Vec<(String, u64)> = table
                .iter()
                .context("full scan for prune")?
                .filter_map(|entry| entry.ok())
                .filter_map(|(key, _)| {
                    let (run_id, seq) = key.value();
                    let run_id_str: &str = run_id;
                    if !keep_ids.contains(run_id_str) {
                        Some((run_id_str.to_string(), seq))
                    } else {
                        None
                    }
                })
                .collect();
            for (run_id, seq) in &to_delete {
                let _ = table.remove((run_id.as_str(), *seq));
            }
            if !to_delete.is_empty() {
                tracing::debug!(deleted = to_delete.len(), "pruned telemetry events");
            }
        }
        wtx.commit().context("commit prune")?;
        // Drop counters for pruned runs so the in-memory map stays bounded.
        let mut counters = self.counters.lock().unwrap_or_else(|e| e.into_inner());
        counters.retain(|k, _| keep_ids.contains(k));
        Ok(())
    }

    /// Return all events for `run_id` in sequence order.
    pub fn events_for_run(&self, run_id: &str) -> anyhow::Result<Vec<Value>> {
        let rtx = self.db.begin_read().context("begin read")?;
        let table = rtx.open_table(EVENTS).context("open events table")?;
        let upper = next_string(run_id);
        let range = (run_id, 0u64)..(upper.as_str(), 0u64);
        let mut events = Vec::new();
        for entry in table.range(range).context("range scan")? {
            let (_, val) = entry.context("read entry")?;
            let parsed: Value = serde_json::from_str(val.value()).unwrap_or(Value::Null);
            events.push(parsed);
        }
        Ok(events)
    }

    /// Aggregate all events for `run_id` into a `RunSummary`.
    pub fn summary_for_run(&self, run_id: &str) -> anyhow::Result<RunSummary> {
        let events = self.events_for_run(run_id)?;
        let mut summary = RunSummary {
            tool_calls: 0,
            tool_errors: 0,
            tools_used: HashMap::new(),
            subagents_spawned: 0,
            subagent_tokens: 0,
            total_cost_usd: None,
            total_turns: None,
        };
        for ev in &events {
            let ty = ev.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match ty {
                "assistant" => {
                    if let Some(tools) = ev.get("tools").and_then(|v| v.as_array()) {
                        for tool in tools {
                            summary.tool_calls += 1;
                            if let Some(name) = tool.get("name").and_then(|v| v.as_str()) {
                                *summary.tools_used.entry(name.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                "user" => {
                    if let Some(results) = ev.get("tool_results").and_then(|v| v.as_array()) {
                        for result in results {
                            if result
                                .get("is_error")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false)
                            {
                                summary.tool_errors += 1;
                            }
                        }
                    }
                }
                "subagent_started" => {
                    summary.subagents_spawned += 1;
                }
                "subagent_completed" | "subagent_progress" => {
                    if let Some(tokens) = ev.get("total_tokens").and_then(|v| v.as_u64()) {
                        summary.subagent_tokens = summary.subagent_tokens.saturating_add(tokens);
                    }
                }
                "result" => {
                    if let Some(cost) = ev.get("cost_usd").and_then(|v| v.as_f64()) {
                        summary.total_cost_usd = Some(cost);
                    }
                    if let Some(turns) = ev.get("turns").and_then(|v| v.as_u64()) {
                        summary.total_turns = Some(turns);
                    }
                }
                _ => {}
            }
        }
        Ok(summary)
    }
}

/// Return the exclusive upper bound for a prefix-range scan.
/// Increments the last byte of the string; appends `\x00` if all bytes are
/// `0xFF` (extremely unlikely for run IDs, but correct).
fn next_string(s: &str) -> String {
    let mut bytes = s.as_bytes().to_vec();
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        if bytes[i] < 0xFF {
            bytes[i] += 1;
            return String::from_utf8(bytes).unwrap_or_else(|_| format!("{s}\x00"));
        }
        bytes[i] = 0;
    }
    // All bytes were 0xFF — append a null byte as fallback.
    let mut result = s.to_string();
    result.push('\x00');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn next_string_basic() {
        assert_eq!(next_string("abc"), "abd");
        assert_eq!(next_string("a"), "b");
    }

    #[test]
    fn append_and_retrieve() {
        let dir = TempDir::new().unwrap();
        let store = TelemetryStore::open(&dir.path().join("test.redb")).unwrap();
        let ev = serde_json::json!({"type": "assistant", "tools": [{"name": "Bash"}]});
        store.append_raw("run-1", ev.clone()).unwrap();
        store.append_raw("run-1", ev.clone()).unwrap();
        let events = store.events_for_run("run-1").unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn summary_counts_tool_calls() {
        let dir = TempDir::new().unwrap();
        let store = TelemetryStore::open(&dir.path().join("test.redb")).unwrap();
        let ev = serde_json::json!({
            "type": "assistant",
            "tools": [{"name": "Bash"}, {"name": "Edit"}]
        });
        store.append_raw("run-1", ev).unwrap();
        let summary = store.summary_for_run("run-1").unwrap();
        assert_eq!(summary.tool_calls, 2);
        assert_eq!(summary.tools_used["Bash"], 1);
        assert_eq!(summary.tools_used["Edit"], 1);
    }

    #[test]
    fn isolation_between_runs() {
        let dir = TempDir::new().unwrap();
        let store = TelemetryStore::open(&dir.path().join("test.redb")).unwrap();
        store
            .append_raw("run-a", serde_json::json!({"type": "init"}))
            .unwrap();
        store
            .append_raw("run-b", serde_json::json!({"type": "result"}))
            .unwrap();
        assert_eq!(store.events_for_run("run-a").unwrap().len(), 1);
        assert_eq!(store.events_for_run("run-b").unwrap().len(), 1);
    }
}
