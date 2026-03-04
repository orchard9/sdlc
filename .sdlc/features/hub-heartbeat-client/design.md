# Design: hub-heartbeat-client

## Summary

A single new source file `crates/sdlc-server/src/heartbeat.rs` exposes one
public function:

```rust
pub fn spawn_heartbeat_task(state: &AppState) -> Option<tokio::task::AbortHandle>
```

`AppState::new_with_port` calls this after the watcher tasks are spawned and
includes the returned `AbortHandle` in the `WatcherGuard` vector.

No new files, no new dependencies, no schema changes.

---

## Module: `crates/sdlc-server/src/heartbeat.rs`

### Public API

```rust
/// Spawn the hub heartbeat background task.
///
/// Returns `None` if `SDLC_HUB_URL` is not set — no task is spawned.
/// The returned `AbortHandle` must be held in `WatcherGuard` so the task
/// is cancelled when `AppState` is dropped.
pub fn spawn_heartbeat_task(state: &AppState) -> Option<tokio::task::AbortHandle>
```

### Internal helpers (private)

```rust
/// Collect the heartbeat payload from current AppState.
/// All reads are best-effort; missing files yield None fields.
async fn build_payload(state: &AppState, base_url: &str) -> HeartbeatPayload
```

### Data flow

```
new_with_port()
  └─ spawn_heartbeat_task(&state)
       │
       ├─ read SDLC_HUB_URL   → None? return None (no task)
       ├─ read SDLC_BASE_URL  → fallback http://localhost:{port}
       │
       └─ tokio::spawn(loop)
            │
            ├─ sleep 30s
            ├─ build_payload()
            │    ├─ name:              root.file_name()
            │    ├─ url:               base_url (captured at spawn)
            │    ├─ active_milestone:  parse .sdlc/state.yaml → active_milestone
            │    ├─ feature_count:     count entries in .sdlc/features/
            │    └─ agent_running:     agent_runs.lock().await.is_empty() == false
            │
            └─ http_client
                 .post(hub_url + "/api/hub/heartbeat")
                 .timeout(5s)
                 .json(&payload)
                 .send()
                 │
                 ├─ Ok(resp) if resp.status().is_success() → debug!("heartbeat ok")
                 └─ Err(_) | non-2xx → warn!("heartbeat failed: {err}")
```

### State.yaml shape (read for active_milestone)

The file lives at `{root}/.sdlc/state.yaml`. We only need one field:

```yaml
active_milestone: v37-project-hub   # or absent
```

We deserialize minimally:

```rust
#[derive(serde::Deserialize, Default)]
struct StateYaml {
    active_milestone: Option<String>,
}
```

### feature_count

Count immediate subdirectories of `{root}/.sdlc/features/` using
`std::fs::read_dir`. If the directory doesn't exist, return `None`.

### agent_running

```rust
let running = !state.agent_runs.lock().await.is_empty();
```

---

## Integration point: `state.rs`

In `AppState::new_with_port`, after the existing 8 watcher tasks are pushed to
`handles`:

```rust
// Spawn hub heartbeat task (no-op if SDLC_HUB_URL is not set).
if let Some(hb_handle) = crate::heartbeat::spawn_heartbeat_task(&state) {
    handles.push(hb_handle);
}
```

The `WatcherGuard` will then abort the heartbeat task alongside the file watchers
when `AppState` is dropped.

---

## mod.rs / lib.rs wiring

Add to `crates/sdlc-server/src/main.rs` (or wherever modules are declared):

```rust
mod heartbeat;
```

---

## No UI changes

This is a pure backend feature. No frontend changes are required.

---

## Error handling

| Scenario | Behaviour |
|---|---|
| `SDLC_HUB_URL` not set | `spawn_heartbeat_task` returns `None`, no task |
| Tokio runtime not present | guard in `new_with_port` prevents spawn |
| Network error / timeout | `warn!`, loop continues |
| Non-2xx response | `warn!`, loop continues |
| `.sdlc/state.yaml` missing | `active_milestone: None`, continue |
| `.sdlc/features/` missing | `feature_count: None`, continue |
| `agent_runs` lock fails | Cannot fail (Tokio Mutex, no poison) |
