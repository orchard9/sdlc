# Code Review: hub-heartbeat-client

## Summary

The implementation adds `crates/sdlc-server/src/heartbeat.rs` — a new module
with a single public function `spawn_heartbeat_task` — and wires it into
`AppState::new_with_port` in `state.rs` and the module list in `lib.rs`.

---

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-server/src/heartbeat.rs` | New — heartbeat module |
| `crates/sdlc-server/src/lib.rs` | Added `pub mod heartbeat;` |
| `crates/sdlc-server/src/state.rs` | Call `spawn_heartbeat_task` in `new_with_port` |

---

## Findings

### Correctness

- `SDLC_HUB_URL` absent → returns `None`, no task spawned. Correct.
- Task is only spawned inside the `if tokio::runtime::Handle::try_current().is_ok()` guard, matching the pattern for all other watcher tasks. Correct.
- `AbortHandle` is pushed to `handles` and included in `WatcherGuard` → task aborted on drop. Correct.
- `reqwest::Client` reused from `AppState` (no new client allocation). Correct.
- 5-second timeout applied via `.timeout(Duration::from_secs(5))`. Correct.
- Non-2xx responses produce `warn!`, not panics. Correct.
- Network errors produce `warn!`, loop continues. Correct.
- `active_milestone` read from `.sdlc/state.yaml` with graceful `None` fallback. Correct.
- `feature_count` counts subdirectories of `.sdlc/features/` with graceful `None` fallback. Correct.
- `agent_running` derived from `agent_runs.lock().await.is_empty()`. Correct.

### Payload Shape

Matches `HeartbeatPayload` in `hub.rs` exactly:
- `name: String` — project root basename
- `url: String` — `SDLC_BASE_URL` or `http://localhost:{port}`
- `active_milestone: Option<String>`
- `feature_count: Option<u32>`
- `agent_running: Option<bool>` — always `Some(bool)` from the client

### Code Quality

- No `unwrap()` in library code — `?`-style chaining and `unwrap_or_default()` only.
- `&Path` used in function signatures, not `&PathBuf` (clippy-clean).
- Logging at appropriate levels: `debug!` on success, `warn!` on failure.
- Single `StateYaml` struct defined locally for minimal deserialization — no leaking of internal types.

### Test Coverage

5 unit tests all pass:
1. `spawn_returns_none_when_hub_url_unset` — env guard works
2. `count_features_none_for_missing_dir` — graceful `None`
3. `count_features_counts_subdirs` — correct count, ignores files
4. `read_active_milestone_returns_none_for_missing_file` — graceful `None`
5. `read_active_milestone_parses_yaml` — correct parse
6. `read_active_milestone_returns_none_when_field_absent` — field absent = `None`

(6 tests in `heartbeat` module; 2 more in `hub` module remain unchanged.)

Full suite: 49 integration tests pass. `cargo clippy --all -- -D warnings` clean.

### No Regressions

- `AppState::new` and `AppState::new_for_test` do not call `spawn_heartbeat_task` (no Tokio runtime guard).
- `AppState::new_with_port_hub` calls `new_with_port` internally, so heartbeat is available in hub mode too (correct — a hub-mode server may also want to register with another hub).
- No new dependencies added.
- No schema or file-format changes.

---

## Verdict: Approved

Implementation is minimal, correct, and consistent with the existing watcher-task pattern. All findings are resolved.
