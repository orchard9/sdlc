# Spike: MCP Run Telemetry

**Slug:** mcp-run-telemetry
**Date:** 2026-03-01
**Verdict:** ADOPT

## The Question

Can we capture all agent and subagent activity (tool calls, tool results, MCP calls, subagent lifecycle) per run into a local embedded database so users can see the full execution graph?

## Success Criteria

- Given a `run_id`, retrieve every tool call (name, input, output, duration) for the parent agent AND any subagents it spawned
- Stored locally — no external service, embedded in `sdlc-server` binary
- Rust-native, single file, no C dependency
- UI can query "show all MCP calls for run X" or "cost breakdown by tool"

## Candidates Evaluated

| Candidate | Verdict | Reason |
|---|---|---|
| `claude_telemetry` (TechNickAI) | **Eliminated** | Python-only. Requires external OTEL backend (Logfire, Sentry, Honeycomb). Zero value for our Rust binary. Does not solve subagent visibility — it wraps the Python SDK hooks which don't exist in our Rust crate. |
| `sled` | **Eliminated** | Still in beta ("champagne of beta embedded databases"). No production stability guarantees. No SQL. Worse than redb on every axis. |
| `redb` | **Winner** | Already a workspace dependency. Pure Rust (no C). ACID. B-tree based. 2.x stable. Single file. Covers all our access patterns. |
| `rusqlite` | **Not prototyped** | Would work, but adds a C/FFI dependency. Overkill — we don't need SQL joins. redb's prefix-range scan covers everything we need. |

## Winner: `redb` + stream event extension

### Why It Won

1. **Already in the workspace** — `redb = "2"` is in `Cargo.toml [workspace.dependencies]`. Zero new dependencies.
2. **Pure Rust** — no system libraries, no C bindings. Binary stays self-contained.
3. **Single file** — one `.redb` file alongside the existing `.sdlc/.runs/` directory.
4. **ACID transactions** — consistent even if the server crashes mid-run.
5. **B-tree key ordering** — composite `(run_id, seq)` key allows efficient prefix-range scans: all events for a run in one scan.

### The Real Discovery: The Data Is Already In The Stream

The critical finding is that `claude-agent`'s message stream already emits all the data we need — **we are currently discarding it**:

| Stream message | Data | Current handling |
|---|---|---|
| `Message::User` → `UserContentBlock::ToolResult` | Tool outputs (responses) | `{"type": "user"}` — **content dropped** |
| `SystemPayload::TaskStarted` | Subagent spawned (task_id, description) | Falls into `Unknown` — **dropped** |
| `SystemPayload::TaskProgress` | Subagent progress (last_tool_name, tokens, duration) | Falls into `Unknown` — **dropped** |
| `SystemPayload::TaskNotification` | Subagent completed (status, summary, usage) | Falls into `Unknown` — **dropped** |
| `Message::ToolProgress` | Tool timing (elapsed_time_seconds, task_id) | Captured but `task_id` linkage lost |

Tool inputs (ToolUse) are captured. Everything else is lost.

### How It Works

**Storage:** One `redb` database at `.sdlc/telemetry.redb`. Table `events` with composite key `(run_id: &str, seq: u64)` → JSON event. Prefix-range scan retrieves all events for a run_id in O(k log n) where k = events for that run.

**Capture:** Extend `message_to_event` in `runs.rs` to extract `ToolResult` content from `Message::User` and emit `TaskStarted/TaskProgress/TaskNotification` from `SystemPayload`.

**API:** Add `GET /api/runs/{id}/telemetry` → returns all events for a run, typed and sequenced.

### Working Prototype

Location: `/tmp/spike-mcp-run-telemetry/prototype/` (ephemeral)
Preserved: `.sdlc/spikes/mcp-run-telemetry/prototype-src/main.rs`

Prototype proves: two concurrent runs store 9 and 4 events respectively. Isolated query returns exactly the events for the queried run. Summary aggregation (tool_calls, subagent_tokens, cost) works. Database persists across process restarts.

### Validation Evidence

```
=== Validation: Query run A events ===
Retrieved 9 events for run A:
  [seq=1] Init(model=claude-sonnet-4-6, mcp_servers=["sdlc"])
  [seq=2] ToolCall(name=Bash, id=tu_001)
  [seq=3] ToolResult(id=tu_001, error=false, content={"action":"implement_task"...})
  [seq=4] SubagentStarted(task=task_abc123, desc=Implement JWT validation middleware...)
  [seq=5] SubagentProgress(task=task_abc123, last_tool=Some("Edit"), tokens=8420)
  [seq=6] SubagentCompleted(task=task_abc123, status=success, summary=Implemented JWT...)
  [seq=7] ToolCall(name=mcp__sdlc__sdlc_approve_artifact, id=tu_003)
  [seq=8] ToolResult(id=tu_003, error=false, content={"status":"approved"...})
  [seq=9] RunResult(cost=$0.1470, turns=8)

=== Validation: Summary for run A ===
  Total events:      9
  Tool calls:        2
  Tool errors:       0
  Tools used:        {"Bash": 1, "mcp__sdlc__sdlc_approve_artifact": 1}
  Subagents spawned: 1
  Subagent tokens:   11200
  Subagent duration: 18300ms
  Total cost:        $0.1470
  Total turns:       8

=== Validation: Isolated query — run B only ===
Run B events: 4 (should be 4)  ✅

✅ All assertions passed. redb works for MCP run telemetry.
```

## Implementation Plan

### Files to add

- `crates/sdlc-server/src/telemetry.rs` — `TelemetryStore` struct: open/append/events_for_run/summary_for_run. Wraps `redb`. ~150 lines. Copied/adapted directly from prototype.

### Files to modify

**`crates/sdlc-server/src/routes/runs.rs`:**

1. `fn message_to_event()` — extend to capture discarded data:

```rust
Message::User(user) => {
    // Capture tool results (currently discarded as `{"type":"user"}`)
    let tool_results: Vec<serde_json::Value> = user.message.content.iter()
        .filter_map(|c| {
            if let UserContentBlock::ToolResult { tool_use_id, content, is_error } = c {
                let text = content.as_ref()
                    .and_then(|blocks| blocks.iter().find_map(|b| {
                        if let ToolResultContent::Text { text } = b { Some(text.as_str()) } else { None }
                    }))
                    .unwrap_or("");
                Some(serde_json::json!({
                    "tool_use_id": tool_use_id,
                    "is_error": is_error.unwrap_or(false),
                    "content": &text[..text.len().min(2000)]
                }))
            } else {
                None
            }
        })
        .collect();
    serde_json::json!({"type": "user", "tool_results": tool_results})
}
```

2. `SystemPayload::TaskStarted/TaskProgress/TaskNotification` arms in the `System` match:

```rust
SystemPayload::TaskStarted(t) => serde_json::json!({
    "type": "subagent_started",
    "task_id": t.task_id,
    "tool_use_id": t.tool_use_id,
    "description": t.description,
}),
SystemPayload::TaskProgress(t) => serde_json::json!({
    "type": "subagent_progress",
    "task_id": t.task_id,
    "last_tool_name": t.last_tool_name,
    "total_tokens": t.usage.total_tokens,
    "tool_uses": t.usage.tool_uses,
    "duration_ms": t.usage.duration_ms,
}),
SystemPayload::TaskNotification(t) => serde_json::json!({
    "type": "subagent_completed",
    "task_id": t.task_id,
    "status": t.status,
    "summary": t.summary,
    "total_tokens": t.usage.as_ref().map(|u| u.total_tokens),
    "duration_ms": t.usage.as_ref().map(|u| u.duration_ms),
}),
```

3. In `spawn_agent_run`, after `accumulated_events.push(event.clone())`, also append to telemetry store:

```rust
// Append to telemetry store (non-blocking, best-effort)
if let Some(store) = &app.telemetry {
    let store = store.clone();
    let run_id2 = run_id.clone();
    let ev = event.clone();
    tokio::task::spawn_blocking(move || {
        let _ = store.append_raw(&run_id2, ev);
    });
}
```

**`crates/sdlc-server/src/state.rs`:**

Add `telemetry: Option<Arc<TelemetryStore>>` to `AppState`:

```rust
pub struct AppState {
    // ... existing fields ...
    pub telemetry: Option<Arc<TelemetryStore>>,
}
```

Initialize in `AppState::new_with_port`:

```rust
let telemetry = TelemetryStore::open(&root.join(".sdlc").join("telemetry.redb"))
    .ok()
    .map(Arc::new);
```

**`crates/sdlc-server/src/routes/` — new file `telemetry.rs`:**

```rust
// GET /api/runs/:id/telemetry — all events for a run
pub async fn get_run_telemetry(
    Path(id): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let store = app.telemetry.as_ref()
        .ok_or_else(|| AppError::internal("Telemetry not initialized"))?;
    let events = tokio::task::spawn_blocking({
        let store = store.clone();
        let id = id.clone();
        move || store.events_for_run(&id)
    }).await??;
    Ok(Json(serde_json::json!({"run_id": id, "events": events})))
}

// GET /api/runs/:id/telemetry/summary — aggregated stats
pub async fn get_run_telemetry_summary(
    Path(id): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let store = app.telemetry.as_ref()
        .ok_or_else(|| AppError::internal("Telemetry not initialized"))?;
    let summary = tokio::task::spawn_blocking({
        let store = store.clone();
        let id = id.clone();
        move || store.summary_for_run(&id)
    }).await??;
    Ok(Json(serde_json::json!({
        "run_id": id,
        "tool_calls": summary.tool_calls,
        "tool_errors": summary.tool_errors,
        "tools_used": summary.tools_used,
        "subagents_spawned": summary.subagents_spawned,
        "subagent_tokens": summary.subagent_tokens,
        "total_cost_usd": summary.total_cost_usd,
        "total_turns": summary.total_turns,
    })))
}
```

**`crates/sdlc-server/src/main.rs` or router:**

```rust
.route("/api/runs/:id/telemetry", get(routes::telemetry::get_run_telemetry))
.route("/api/runs/:id/telemetry/summary", get(routes::telemetry::get_run_telemetry_summary))
```

### Dependencies to add

None. `redb = "2"` is already in `[workspace.dependencies]`. Add to `crates/sdlc-server/Cargo.toml`:

```toml
redb = { workspace = true }
```

### Configuration

No configuration needed. Database opens at `.sdlc/telemetry.redb` automatically. It is gitignored (alongside `.sdlc/.runs/`).

Add to `.gitignore`:
```
.sdlc/telemetry.redb
```

### Integration points

| Location | What changes |
|---|---|
| `runs.rs:message_to_event()` | Extend match arms for `Message::User` and `SystemPayload::Task*` |
| `runs.rs:spawn_agent_run()` | After `accumulated_events.push()`, fire async `store.append_raw()` |
| `state.rs:AppState` | Add `telemetry: Option<Arc<TelemetryStore>>` field |
| `state.rs:AppState::new_with_port()` | Initialize telemetry store |
| New `routes/telemetry.rs` | `GET /api/runs/:id/telemetry` and `/telemetry/summary` |
| Router registration | Register the two new routes |

### Code patterns

The `TelemetryStore` uses a composite key `(run_id: &str, seq: u64)` in redb:

```rust
const EVENTS: TableDefinition<(&str, u64), &str> = TableDefinition::new("events");
```

Prefix-range scan for all events of a run:
```rust
let prefix_end = next_string(run_id); // "abc" → "abd"
let range = table.range((run_id, 0)..(prefix_end.as_str(), 0))?;
```

This is O(k log n) where k = events in that run. No full table scan needed.

The `append_raw` method takes a `serde_json::Value` directly (the already-computed event from `message_to_event`), serializes it, and inserts — no double-conversion.

## Risks and Open Questions

- **DB file growth:** 1.5 MB for 13 events (redb allocates pages). For 1000 runs × 50 events = ~100 MB. Acceptable for local dev tool. Mitigation: add retention (delete events for runs older than N days, matching `enforce_retention` pattern for JSON files).
- **Concurrent writers:** `spawn_agent_run` runs many parallel tasks. `redb` supports one writer at a time (serialized by the write transaction). The `spawn_blocking` pattern serializes writes naturally. Benchmark if needed.
- **Tool result content size:** Some tool results are large (file reads, bash output). The prototype truncates at 2000 chars. Fine for the UI — full content already in the JSON events sidecar.
- **Subagent output_file path:** `TaskNotification.output_file` is a path to the subagent's output. Not currently captured. Could be added as a field to retrieve the full subagent result if needed.

## What Was Not Tried

- **rusqlite:** Would provide SQL queries (GROUP BY tool, date ranges). Not needed — redb prefix scans cover all our UI needs. Also adds C FFI dependency.
- **sled:** Beta status is disqualifying for a persistent store users rely on.
- **OpenTelemetry OTEL export:** Would require an external backend. Against our "no external service" constraint. claude_telemetry shows how — but for local UI, overkill.
- **In-memory only (Vec<> in AppState):** Fast but lost on server restart. Users want to see activity for past runs.
