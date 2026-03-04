# Code Review: Hub Server Mode

## Summary

Implemented hub mode for `sdlc-server` — a project navigator that accepts heartbeats from
project instances, maintains an in-memory registry with automatic sweep, emits SSE events,
and persists state to `~/.sdlc/hub-state.yaml`.

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-server/Cargo.toml` | Added `dirs = "5"` dependency |
| `crates/sdlc-server/src/hub.rs` | New: HubRegistry, ProjectEntry, sweep, persistence, unit tests |
| `crates/sdlc-server/src/routes/hub.rs` | New: heartbeat, list_projects, hub_sse_events handlers |
| `crates/sdlc-server/src/routes/mod.rs` | Added `pub mod hub;` |
| `crates/sdlc-server/src/state.rs` | Added `hub_registry` field, `new_with_port_hub()`, sweep task spawn |
| `crates/sdlc-server/src/lib.rs` | Added `pub mod hub;`, hub routes registration, `serve_on_hub()`, `build_hub_router()` |
| `crates/sdlc-cli/src/cmd/ui.rs` | Added `--hub` flag, `run_start_hub()` function |

## Findings

### F1: Route registration is always-on (acceptable)
Hub routes (`/api/hub/*`) are registered for both hub and project modes. In project mode,
they return 503 when `hub_registry` is `None`. This avoids route-registration complexity
and is consistent with the existing pattern where some routes are no-ops without certain
conditions.
**Decision: Accept** — clean, no runtime cost, consistent.

### F2: Sweep task is unguarded in `new_with_port_hub`
The hub sweep task spawned in `new_with_port_hub` is not tracked in `WatcherGuard`, so it
won't be aborted when AppState is dropped. In production this is harmless (process exits).
In tests, `new_with_port_hub` is not called, so no leaked tasks.
**Decision: Accept for now, track as improvement task** — adding to watcher guard would
require refactoring the guard construction. Low risk for production code; tests don't use
hub mode.

### F3: No `unwrap()` in library code
Verified: all error handling uses `?`, `ok()`, `unwrap_or_default()`, or logs warnings.
No `unwrap()` calls in hub.rs or routes/hub.rs.
**Finding: Clean.**

### F4: Atomic writes via sdlc_core::io::atomic_write
`HubRegistry::persist()` calls `sdlc_core::io::atomic_write` for the state file.
**Finding: Correct.**

### F5: hub_registry is properly `Clone`
`Arc<Mutex<HubRegistry>>` is `Clone`, so `AppState`'s `#[derive(Clone)]` works correctly.
HubRegistry itself is not Clone (intentionally — only the Arc wrapper is shared).
**Finding: Correct.**

### F6: SSE stream subscribes after locking hub
`hub_sse_events` acquires the lock to subscribe to `event_tx`, then drops the lock before
streaming. This is correct — `broadcast::Sender::subscribe()` doesn't hold the lock during
streaming.
**Finding: Correct.**

### F7: CLI hub mode skips update scaffolding
`run_start_hub` skips the `update::run(root)` call that normal mode performs. This is
intentional — hub mode has no project and update scaffolding would fail.
**Finding: Correct.**

### F8: Unit tests cover all status transitions
7 unit tests in `hub.rs` cover: registration, update, stale after 30s, offline after 90s,
removal after 5min, load from persisted state (all offline), and sort order.
**Finding: Complete coverage of core logic.**

## Build and Test Status

- `SDLC_NO_NPM=1 cargo build --all`: PASS (0 errors, 0 warnings)
- `SDLC_NO_NPM=1 cargo test --all`: PASS (49 tests, 0 failures)
- `cargo clippy --all -- -D warnings`: PASS (0 warnings)

## Verdict: Approved

Implementation is clean, follows project conventions, all tests pass.
F2 (sweep task not in WatcherGuard) is tracked below as a follow-up task.
