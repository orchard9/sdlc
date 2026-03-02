# Code Review: run-events-api

## Summary

Implementation adds a `redb`-backed `TelemetryStore` that persists raw agent events across server restarts and exposes two new HTTP endpoints for querying per-run telemetry.

## Changes Reviewed

### `crates/sdlc-server/src/telemetry.rs` (new file)

- `TelemetryStore` wraps `Arc<redb::Database>` with a `Mutex<HashMap<String, u64>>` for in-memory sequence counters — correct and safe
- `open()` creates the table on first open via a write transaction — correct initialization pattern
- `next_seq()` lazily initializes the per-run counter from the DB on first call — avoids unnecessary DB scans on every append
- `append_raw()` uses a single write transaction per event — correct for redb's exclusive write lock model
- `events_for_run()` uses prefix-range scan `(run_id, 0)..(next_string(run_id), 0)` — correct composite key ordering
- `next_string()` handles the 0xFF edge case — correct
- All methods return `anyhow::Result` — no bare `redb::Error` in return types, satisfying clippy `result-large-err`
- No `unwrap()` in library code — uses `?` with `.context()` throughout
- `RunSummary` correctly aggregates tool calls, errors, tools_used, subagent stats, cost, and turns from the event stream
- 5 unit tests covering: basic store, append/retrieve, summary counting, run isolation

### `crates/sdlc-server/src/state.rs`

- `pub telemetry: Option<Arc<TelemetryStore>>` added to `AppState` — correct `Option<Arc<_>>` pattern for graceful degradation
- Initialized via `TelemetryStore::open(...).ok().map(Arc::new)` — if `.sdlc/` is not writable the server still starts
- Import of `TelemetryStore` added — correct

### `crates/sdlc-server/src/routes/runs.rs`

- `telemetry_store` cloned from `app.telemetry` before spawn — correct, `AppState` is `Clone`
- Non-blocking write using `tokio::task::spawn_blocking` — correct for synchronous redb writes
- Error dropped with `let _ =` — intentional, telemetry failure must not affect agent execution
- Single hook point after `accumulated_events.push(event.clone())` — events captured for every message type

### `crates/sdlc-server/src/routes/telemetry.rs` (new file)

- `get_run_telemetry`: dispatches to `spawn_blocking`, returns `{ run_id, events: [...] }`
- `get_run_telemetry_summary`: dispatches to `spawn_blocking`, returns full summary JSON
- Both handlers return 500 if `app.telemetry` is `None` — correct graceful degradation
- Uses `anyhow::anyhow!` for error construction — consistent with existing patterns in the codebase

### `crates/sdlc-server/src/lib.rs`

- `pub mod telemetry` added
- Two routes registered: `GET /api/runs/{id}/telemetry` and `GET /api/runs/{id}/telemetry/summary`
- Placement: immediately after `GET /api/runs/{id}` — logical grouping

### `crates/sdlc-server/src/routes/mod.rs`

- `pub mod telemetry` added alphabetically in the correct position

### `crates/sdlc-server/Cargo.toml`

- `redb = { workspace = true }` added — uses workspace version, no version pin duplication

### `.gitignore`

- `.sdlc/telemetry.redb` added — database file correctly excluded from version control

## Quality Checks

- `SDLC_NO_NPM=1 cargo build --all` — passes
- `SDLC_NO_NPM=1 cargo test --all` — all suites pass (498 tests pass, 0 new failures)
- `cargo clippy --all -- -D warnings` — passes with no warnings

## Verdict: Approved

All acceptance criteria from the spec are met. The implementation is correct, safe, and consistent with project conventions.
