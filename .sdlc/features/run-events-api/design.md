# Design: run-events-api

## Architecture

Three integration points + two new files.

### `TelemetryStore` (`crates/sdlc-server/src/telemetry.rs`)

Wraps a `redb::Database`. Single table `EVENTS` with composite key `(run_id: &str, seq: u64)` → JSON string.

```rust
const EVENTS: TableDefinition<(&str, u64), &str> = TableDefinition::new("events");
```

Per-run sequence counter lives in a `DashMap<String, u64>` (in-memory). On startup, the counter for each run is initialized by reading the max seq from the DB (one scan per run on first access). This avoids a DB round-trip on every append after the first.

Key operations:
- `append_raw(run_id, event: serde_json::Value)` — serialize to string, write in a single write transaction
- `events_for_run(run_id)` — prefix-range scan `(run_id, 0)..(next_string(run_id), 0)`, deserialize each value
- `summary_for_run(run_id)` — same scan, aggregate into `RunSummary` struct

`next_string(s)` — increment the last byte of the string to get the exclusive upper bound. Handles the edge case where `run_id` ends in `0xFF` by appending a null byte.

### `AppState` integration (`state.rs`)

Add `pub telemetry: Option<Arc<TelemetryStore>>`. Initialize in `AppState::new_with_port()` after the root path is resolved. `Option` so the server degrades gracefully if the `.sdlc/` directory is not writable.

### Write hook (`runs.rs:spawn_agent_run`)

After the existing `accumulated_events.push(event.clone())` line:

```rust
if let Some(store) = &app.telemetry {
    let store = store.clone();
    let run_id2 = run_id.clone();
    let ev = event.clone();
    tokio::task::spawn_blocking(move || { let _ = store.append_raw(&run_id2, ev); });
}
```

`spawn_blocking` because `redb` write transactions are synchronous. Non-blocking from the perspective of the streaming loop — errors are silently dropped (telemetry must not affect agent execution).

### HTTP routes (`routes/telemetry.rs`)

Both handlers extract `app.telemetry`, return `500` if not initialized, dispatch to `spawn_blocking` for the DB read, serialize the result to JSON.

`GET /api/runs/:id/telemetry` — `{ run_id, events: [...] }`
`GET /api/runs/:id/telemetry/summary` — `{ run_id, tool_calls, tool_errors, tools_used, subagents_spawned, subagent_tokens, total_cost_usd, total_turns }`

## Concurrency

`redb` serializes write transactions (one writer at a time). Concurrent agent runs each fire `spawn_blocking` — these queue naturally through redb's write lock. Read transactions are concurrent. No additional synchronization needed.

## File growth

Each event serialized to ~100-500 bytes. redb allocates in pages (~4KB). Worst case: 1000 runs × 50 events × 500 bytes = 25MB raw, ~50-100MB with redb page overhead. Acceptable for a local dev tool. Retention (drop events for runs older than N days) can be added later if needed.
