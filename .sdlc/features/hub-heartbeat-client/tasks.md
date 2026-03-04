# Tasks: hub-heartbeat-client

## T1 — Create `crates/sdlc-server/src/heartbeat.rs`

Implement the `spawn_heartbeat_task` function and private `build_payload` helper.

- Read `SDLC_HUB_URL` env var; return `None` if absent.
- Read `SDLC_BASE_URL` env var; fall back to `http://localhost:{port}`.
- Spawn tokio loop: sleep 30s, build payload, POST with 5s timeout.
- `build_payload`: name from `root.file_name()`, active_milestone from
  `.sdlc/state.yaml`, feature_count from `.sdlc/features/` dir count,
  agent_running from `agent_runs` lock.
- Log `debug!` on success, `warn!` on any error.
- Return `AbortHandle`.

## T2 — Wire heartbeat module into `state.rs`

In `AppState::new_with_port`, after the watcher tasks:
- Call `crate::heartbeat::spawn_heartbeat_task(&state)`.
- If `Some(handle)`, push to `handles` vec.
- Ensures `WatcherGuard` aborts the task on drop.

## T3 — Register module in `lib.rs` / `main.rs`

Add `mod heartbeat;` to the sdlc-server module declarations.

## T4 — Unit test: heartbeat skipped when no hub URL

In `heartbeat.rs` `#[cfg(test)]` block:
- Verify `spawn_heartbeat_task` returns `None` when `SDLC_HUB_URL` is not set.
- Use a temp dir and `AppState::new_for_test` so no real tasks are spawned.

## T5 — Run tests and clippy

- `SDLC_NO_NPM=1 cargo test --all`
- `cargo clippy --all -- -D warnings`
- Fix any failures before marking complete.
