# Spec: Orchestrator SSE Bridge

## Summary

The orchestrator daemon (`sdlc orchestrate`) and the SDLC server (`sdlc ui`) share the same redb `ActionDb` but currently have no real-time signaling channel. When an action transitions from Pending → Running → Completed/Failed, the frontend has no way to know until it refreshes. This feature closes that gap with three coordinated additions:

1. **`ActionStateChanged` SSE variant** — a new `SseMessage` variant the server broadcasts whenever an orchestrator action state changes; the frontend listens via `/api/events` and refreshes the orchestrator panel in real time.
2. **Daemon sentinel file** — the orchestrator daemon writes a lightweight JSON sentinel file (`.sdlc/.orchestrator.state`) after every tick; the server watches this file's mtime and emits `ActionStateChanged` when it changes.
3. **Server directory watcher** — `AppState::new_with_port` spawns a background task that polls the sentinel file mtime at 800ms intervals (matching the existing watcher pattern) and broadcasts `SseMessage::ActionStateChanged` when the mtime advances.

## Motivation

Currently the Orchestrator panel in the UI must be polled manually or page-refreshed to see action state changes. As of v08, webhooks land in redb and are processed on the next tick, but there is no push signal to the browser. Users watching an action in flight see a stale "Pending" until they reload. The SSE bridge eliminates that polling gap without adding a new transport.

## Requirements

### R1 — New SSE variant

`SseMessage::ActionStateChanged` is added to the enum in `crates/sdlc-server/src/state.rs`. It carries no payload — it is a generic invalidation signal telling the frontend "re-fetch actions". If specific action metadata is needed later it can be added as a follow-on enhancement.

### R2 — Sentinel file written by the daemon

After each tick completes (both phases: scheduled actions + webhook dispatch), `run_one_tick` in `crates/sdlc-cli/src/cmd/orchestrate.rs` writes or updates the sentinel file at `.sdlc/.orchestrator.state`. Contents:

```json
{
  "last_tick_at": "2026-03-02T07:00:00Z",
  "actions_dispatched": 2,
  "webhooks_dispatched": 1
}
```

The write is best-effort (errors are logged to stderr, not propagated). Atomic write is not required for the sentinel — a partial write from a crashed daemon would at most trigger an extra SSE ping, which is harmless.

### R3 — Server watches sentinel mtime

`AppState::new_with_port` spawns a new background watcher task (matching the existing `state.yaml` watcher pattern):

```rust
// Watch .sdlc/.orchestrator.state mtime — written by the orchestrator daemon
// after each tick. Changes indicate action state transitions.
let sentinel = state.root.join(".sdlc").join(".orchestrator.state");
let tx_orch = tx.clone();
tokio::spawn(async move {
    let mut last_mtime = None::<std::time::SystemTime>;
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        if let Ok(meta) = tokio::fs::metadata(&sentinel).await {
            if let Ok(mtime) = meta.modified() {
                if last_mtime != Some(mtime) {
                    last_mtime = Some(mtime);
                    let _ = tx_orch.send(SseMessage::ActionStateChanged);
                }
            }
        }
    }
});
```

### R4 — SSE event serialization

`ActionStateChanged` must be serialized into the `data:` payload of the SSE stream. The existing event handler (`routes/events.rs`) maps `SseMessage` variants to event names. The new variant maps to event name `"action_state_changed"` and data `{}`.

### R5 — Frontend consumption

The frontend's `useSSE` hook (or equivalent) subscribes to `action_state_changed` events and invalidates/refetches the actions list from `GET /api/orchestrator/webhooks/routes` (routes) and any future `GET /api/orchestrator/actions` endpoint. The exact fetch targets are determined by the orchestrator UI page (which is a separate feature); for this feature it is sufficient that the SSE event fires correctly.

## Out of Scope

- Adding `GET /api/orchestrator/actions` — that is part of the orchestrator-actions feature.
- Changing the frontend orchestrator panel layout — separate feature.
- WebSocket transport — SSE is sufficient and already wired.
- Sentinel file encryption or signing — not needed.

## File Impact

| File | Change |
|---|---|
| `crates/sdlc-server/src/state.rs` | Add `SseMessage::ActionStateChanged` variant; add sentinel file watcher in `AppState::new_with_port` |
| `crates/sdlc-server/src/routes/events.rs` | Serialize `ActionStateChanged` → event name `"action_state_changed"` |
| `crates/sdlc-cli/src/cmd/orchestrate.rs` | Write sentinel file after each `run_one_tick` call |

## Acceptance Criteria

1. After `run_one_tick` completes, `.sdlc/.orchestrator.state` exists and contains valid JSON with `last_tick_at`.
2. The server, running concurrently, emits an `action_state_changed` SSE event within ~1s of the sentinel file being updated.
3. `cargo test --all` passes with `SDLC_NO_NPM=1`.
4. No new `unwrap()` calls in library or server code.
