# Design: Orchestrator SSE Bridge

## Architecture Overview

Three interacting components form the bridge:

```
Orchestrator daemon (CLI thread)
  └─ run_one_tick() completes
       └─ writes .sdlc/.orchestrator.state  ←── sentinel file

Server process (tokio runtime)
  └─ mtime watcher (800ms poll)
       └─ detects file change
            └─ broadcasts SseMessage::ActionStateChanged

Frontend (browser)
  └─ /api/events SSE stream
       └─ receives "action_state_changed" event
            └─ invalidates/refetches actions list
```

The sentinel file is the decoupling point. The daemon and server share the same redb DB, but broadcasting directly into the server's SSE channel would require the daemon to hold a reference to `AppState` — a coupling that crosses crate boundaries and complicates testing. The file-based signal is simpler, already proven by the `state.yaml` watcher, and requires no IPC mechanism.

## Component Designs

### 1. Sentinel File

**Path:** `.sdlc/.orchestrator.state`

**Format:**
```json
{
  "last_tick_at": "2026-03-02T07:00:00Z",
  "actions_dispatched": 2,
  "webhooks_dispatched": 1
}
```

The file is written after every call to `run_one_tick`, regardless of whether any actions were actually dispatched. This ensures the watcher fires even on idle ticks, which would tell the frontend there is no new work — useful for clearing a "pending" indicator.

`std::fs::write` is used directly (not `sdlc_core::io::atomic_write`) because:
- The sentinel is not a user-facing artifact — partial content is harmless.
- The watcher only cares about mtime change, not content integrity.
- Avoiding the temp-file rename simplifies error handling.

### 2. SseMessage Variant

**Location:** `crates/sdlc-server/src/state.rs`

```rust
/// The orchestrator daemon completed a tick — action states may have changed.
/// The frontend should re-fetch the actions list.
ActionStateChanged,
```

No payload is needed. The frontend re-fetches the full list anyway.

### 3. SSE Serialization

**Location:** `crates/sdlc-server/src/routes/events.rs`

New arm added to the `filter_map` match:

```rust
Ok(SseMessage::ActionStateChanged) => {
    let data = serde_json::json!({ "type": "action_state_changed" }).to_string();
    Some(Ok(Event::default().event("orchestrator").data(data)))
},
```

Event channel name: `"orchestrator"`. Using a dedicated channel name (rather than `"update"`) lets the frontend subscribe selectively without triggering a full app reload.

### 4. Server Watcher

**Location:** `crates/sdlc-server/src/state.rs`, inside `AppState::new_with_port`

The new task follows the existing watcher pattern exactly — same polling interval, same mtime comparison, same broadcast call:

```rust
// Watch .sdlc/.orchestrator.state — written by the orchestrator daemon
// after each tick. Fires ActionStateChanged so the frontend can
// refresh the actions list without polling.
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

The watcher is guarded by the existing `if tokio::runtime::Handle::try_current().is_ok()` block, so it is skipped in synchronous unit tests just like the other watchers.

### 5. Daemon Write

**Location:** `crates/sdlc-cli/src/cmd/orchestrate.rs`, end of `run_one_tick`

```rust
// Write sentinel after every tick so the server's mtime watcher
// can broadcast ActionStateChanged to connected SSE clients.
write_tick_sentinel(root, actions_dispatched, webhooks_dispatched);
```

`write_tick_sentinel` is a standalone free function that does the write and logs errors to stderr. It does NOT propagate errors to the caller — a failed write should never crash the daemon.

```rust
fn write_tick_sentinel(root: &Path, actions: usize, webhooks: usize) {
    let path = root.join(".sdlc").join(".orchestrator.state");
    let data = serde_json::json!({
        "last_tick_at": chrono::Utc::now().to_rfc3339(),
        "actions_dispatched": actions,
        "webhooks_dispatched": webhooks,
    });
    if let Err(e) = std::fs::write(&path, data.to_string()) {
        eprintln!("orchestrate: failed to write sentinel: {e}");
    }
}
```

`run_one_tick` is updated to count dispatched actions and webhooks before returning:

```rust
pub fn run_one_tick(root: &Path, db: &Mutex<ActionDb>) -> Result<()> {
    // Phase 1: scheduled actions
    let due = { /* ... existing code ... */ };
    let actions_dispatched = due.len();
    for action in due { dispatch(root, db, action)?; }

    // Phase 2: webhook payloads
    let webhooks = { /* ... existing code ... */ };
    let webhooks_dispatched = webhooks.len();
    for payload in webhooks { dispatch_webhook(root, db, payload)?; }

    // Phase 3: signal server that action states may have changed
    write_tick_sentinel(root, actions_dispatched, webhooks_dispatched);

    Ok(())
}
```

## Data Flow Sequence

```
t=0    daemon: run_one_tick() starts
t=0.1  daemon: dispatches 2 pending actions (Pending→Running→Completed)
t=0.2  daemon: writes .sdlc/.orchestrator.state (mtime updated)
t=0.2  daemon: returns to sleep loop

t=0.8  server watcher: polls .orchestrator.state metadata
t=0.8  server watcher: mtime changed → sends SseMessage::ActionStateChanged
t=0.8  server SSE handler: maps to Event { event:"orchestrator", data:"{...}" }
t=0.8  browser: receives "action_state_changed" event
t=0.8  browser: invalidates actions list, triggers refetch
t=0.9  browser: fetches GET /api/orchestrator/actions → sees Completed
```

Maximum observed latency from state change to browser notification: ~1.6s (daemon writes at t=0.2, watcher fires at t=0.8+0.8=1.6 in worst case). Acceptable for orchestrator tooling.

## Error Handling

| Scenario | Behavior |
|---|---|
| Daemon cannot write sentinel (disk full, etc.) | `eprintln!` to stderr; daemon continues normally; frontend does not receive SSE event for that tick |
| Server watcher crashes | tokio task is dropped silently; sentinel changes no longer trigger SSE; server continues serving other routes |
| Sentinel exists before daemon runs (leftover from previous session) | First mtime read stores the old mtime; on next daemon tick the mtime changes and SSE fires normally |
| Multiple simultaneous tick completions (should not happen in normal use) | Each write updates mtime; watcher fires once per 800ms; some firings may coalesce — acceptable |

## Testing Strategy

- Unit test for `write_tick_sentinel`: verify the sentinel file is created with valid JSON after a call.
- Integration test in `crates/sdlc-server/tests/integration.rs`: start a server with a temp directory, write the sentinel file directly, wait ~2s, verify that the SSE event stream emitted `action_state_changed`. This tests the full watcher → broadcast → serialization chain without needing a real daemon.
- Existing tests must continue to pass.

## Files Changed

| File | Type of Change |
|---|---|
| `crates/sdlc-server/src/state.rs` | Add `ActionStateChanged` variant; add sentinel watcher task |
| `crates/sdlc-server/src/routes/events.rs` | Add `ActionStateChanged` arm in filter_map |
| `crates/sdlc-cli/src/cmd/orchestrate.rs` | Add `write_tick_sentinel`; update `run_one_tick` to call it |
