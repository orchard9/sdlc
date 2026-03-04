# Spec: hub-heartbeat-client

## Overview

Add a background heartbeat task to `sdlc-server` that, when `SDLC_HUB_URL` is
set, POSTs to `{SDLC_HUB_URL}/api/hub/heartbeat` every 30 seconds so the hub
server can track project instances in its registry.

## Goals

- Project instances self-register with a hub server automatically on startup.
- Hub receives fresh liveness data (milestone, feature count, agent activity)
  without polling.
- Failure to reach the hub is silent: no crash, no degraded server behaviour.

## Non-Goals

- No configuration beyond two environment variables.
- No retry logic beyond "try again on the next tick".
- No UI changes in this feature (hub UI is handled elsewhere).

## Environment Variables

| Variable | Required | Purpose |
|---|---|---|
| `SDLC_HUB_URL` | No | Base URL of the hub server. If unset, heartbeats are disabled entirely. |
| `SDLC_BASE_URL` | No | Public URL of this project instance, sent as `url` in each heartbeat payload. Falls back to `http://localhost:{port}` if unset. |

## Heartbeat Payload

Matches `HeartbeatPayload` in `crates/sdlc-server/src/hub.rs`:

```json
{
  "name": "<project folder name>",
  "url": "<SDLC_BASE_URL or http://localhost:{port}>",
  "active_milestone": "<slug or null>",
  "feature_count": 42,
  "agent_running": false
}
```

Fields:
- `name` — basename of the project root directory.
- `url` — derived from `SDLC_BASE_URL` env var, or constructed from local port.
- `active_milestone` — read from `.sdlc/state.yaml` (`active_milestone` field). `None` if absent or file missing.
- `feature_count` — count of entries in `.sdlc/features/` directory. `None` if directory missing.
- `agent_running` — `true` if `AppState::agent_runs` has any active entries.

## Behaviour

1. Task is spawned inside `new_with_port` only (production path).
2. If no Tokio runtime is present (sync unit tests), the spawn is skipped.
3. `SDLC_HUB_URL` is read once at task-spawn time. If absent, the task exits immediately.
4. `SDLC_BASE_URL` is read once at task-spawn time; falls back to `http://localhost:{port}`.
5. Heartbeat loop:
   a. Sleep 30 seconds.
   b. Build payload by reading state.
   c. POST with a 5-second timeout using the existing `reqwest::Client` on `AppState`.
   d. On error (network, timeout, non-2xx): log `warn!`, continue looping.
   e. On success: log `debug!`.
6. The task is registered in `WatcherGuard` so it is aborted cleanly when
   `AppState` is dropped (test isolation, graceful shutdown).

## Implementation Location

New file: `crates/sdlc-server/src/heartbeat.rs`

```rust
pub fn spawn_heartbeat_task(state: &AppState) -> Option<tokio::task::AbortHandle>
```

Called from `AppState::new_with_port` after the watcher tasks are spawned.
Returns `None` if `SDLC_HUB_URL` is not set (no task spawned).

## Dependencies

- `reqwest` — already in `sdlc-server/Cargo.toml` with `rustls-tls` feature.
- No new dependencies needed.

## Testing

- Unit test: `heartbeat_skipped_when_no_hub_url` — verifies the function returns
  `None` when `SDLC_HUB_URL` is not set (or the env var is cleared for the test).
- Integration: manual verification against a running hub instance is sufficient;
  the heartbeat is best-effort and its payload shape is already tested via hub.rs.
