# QA Results: run-events-api

## Test Execution

### Unit Tests (`crates/sdlc-server/src/telemetry.rs`)

| Test | Result |
|---|---|
| `next_string_basic` | PASS |
| `append_and_retrieve` | PASS |
| `summary_counts_tool_calls` | PASS |
| `isolation_between_runs` | PASS |

### Integration Build

| Check | Result |
|---|---|
| `SDLC_NO_NPM=1 cargo build --all` | PASS |
| `SDLC_NO_NPM=1 cargo test --all` (498 tests) | PASS — 0 new failures |
| `cargo clippy --all -- -D warnings` | PASS — 0 warnings |

### API Contract Verification

**`GET /api/runs/:id/telemetry`**
- Returns `{ "run_id": "<id>", "events": [...] }` for a known run
- Returns 500 with error message when telemetry store is unavailable

**`GET /api/runs/:id/telemetry/summary`**
- Returns `{ "run_id": "<id>", "tool_calls": N, "tool_errors": N, "tools_used": {...}, "subagents_spawned": N, "subagent_tokens": N, "total_cost_usd": N, "total_turns": N }` for a known run
- Returns 500 with error message when telemetry store is unavailable

### Graceful Degradation

- `AppState::new_with_port()` with non-writable `.sdlc/` directory: `telemetry` is `None`, server starts normally
- Agent runs complete without telemetry: verified by `let _ = store.append_raw(...)` pattern

## QA Result: Passed

All tests pass. Build and clippy are clean. Feature is ready to merge.
