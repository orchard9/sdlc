# Spec: run-events-api

## Problem

Once `run-event-capture` fixes the stream, we have complete events in the in-memory accumulated array and the `.sdlc/.runs/{id}.events.json` sidecar. But:

1. **No persistence across server restarts** — the in-memory array is gone on restart; the JSON sidecar exists but is not served via API
2. **No cross-run queryability** — the JSON array is a flat file with no index; "show all runs that called mcp__sdlc__sdlc_approve_artifact" requires reading every file
3. **No typed API** — the frontend has no endpoint to fetch structured events for a specific run

## Solution

Add a `redb`-backed `TelemetryStore` that writes every event as it fires and exposes two query endpoints.

### Storage: `TelemetryStore`

New file: `crates/sdlc-server/src/telemetry.rs`

```rust
const EVENTS: TableDefinition<(&str, u64), &str> = TableDefinition::new("events");

pub struct TelemetryStore {
    db: Arc<Database>,
    counters: DashMap<String, AtomicU64>,
}

impl TelemetryStore {
    pub fn open(path: &Path) -> Result<Self>
    pub fn append_raw(&self, run_id: &str, event: serde_json::Value) -> Result<()>
    pub fn events_for_run(&self, run_id: &str) -> Result<Vec<serde_json::Value>>
    pub fn summary_for_run(&self, run_id: &str) -> Result<RunSummary>
}
```

Composite key `(run_id: &str, seq: u64)` — one write transaction per event. Prefix-range scan for `events_for_run`: `(run_id, 0)..(next_string(run_id), 0)`. O(k log n) where k = events for that run.

Database file: `.sdlc/telemetry.redb` (gitignored).

### Integration: `AppState`

Add `pub telemetry: Option<Arc<TelemetryStore>>` to `AppState`. Initialize in `AppState::new_with_port()`:

```rust
let telemetry = TelemetryStore::open(&root.join(".sdlc").join("telemetry.redb"))
    .ok()
    .map(Arc::new);
```

### Capture hook: `spawn_agent_run()`

After `accumulated_events.push(event.clone())`, fire non-blocking write:

```rust
if let Some(store) = &app.telemetry {
    let store = store.clone();
    let run_id2 = run_id.clone();
    let ev = event.clone();
    tokio::task::spawn_blocking(move || { let _ = store.append_raw(&run_id2, ev); });
}
```

Best-effort — telemetry failure never blocks the agent run.

### API: two new endpoints

New file: `crates/sdlc-server/src/routes/telemetry.rs`

- `GET /api/runs/:id/telemetry` — returns `{ run_id, events: [...] }` — all events in sequence order
- `GET /api/runs/:id/telemetry/summary` — returns aggregated stats:
  ```json
  {
    "run_id": "...",
    "tool_calls": 12,
    "tool_errors": 1,
    "tools_used": { "Bash": 4, "Edit": 3, "mcp__sdlc__...": 5 },
    "subagents_spawned": 2,
    "subagent_tokens": 45000,
    "total_cost_usd": 0.147,
    "total_turns": 8
  }
  ```

## Scope

- New file: `crates/sdlc-server/src/telemetry.rs`
- New file: `crates/sdlc-server/src/routes/telemetry.rs`
- Modified: `crates/sdlc-server/src/state.rs` — add `telemetry` field to `AppState`
- Modified: `crates/sdlc-server/src/routes/runs.rs` — wire `store.append_raw()` after event push
- Modified: `crates/sdlc-server/Cargo.toml` — add `redb = { workspace = true }`
- Modified: router registration file — add two new routes
- Modified: `.gitignore` — add `.sdlc/telemetry.redb`

## Dependencies

`redb = "2"` is already in `[workspace.dependencies]`. Only need to add it to `crates/sdlc-server/Cargo.toml`.

## Out of Scope

- Cross-run aggregation queries (global tool usage stats across all runs)
- Retention / compaction (tracked as known risk — add later if file growth is problematic)
- Frontend rendering (Feature: run-activity-ui)
