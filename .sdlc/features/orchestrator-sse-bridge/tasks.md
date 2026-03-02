# Tasks: Orchestrator SSE Bridge

## T1 — Add `ActionStateChanged` SSE variant to `state.rs`

**File:** `crates/sdlc-server/src/state.rs`

Add the new variant to the `SseMessage` enum:

```rust
/// The orchestrator daemon completed a tick — action states may have changed.
/// Frontend should re-fetch the orchestrator actions list.
ActionStateChanged,
```

Add the sentinel file watcher task inside the `if tokio::runtime::Handle::try_current().is_ok()` block in `AppState::new_with_port`, after the existing tools watcher:

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

**Acceptance:** `SseMessage::ActionStateChanged` compiles; watcher task is inside the tokio guard.

---

## T2 — Serialize `ActionStateChanged` in `events.rs`

**File:** `crates/sdlc-server/src/routes/events.rs`

Add a new arm to the `filter_map` match block:

```rust
Ok(SseMessage::ActionStateChanged) => {
    let data = serde_json::json!({ "type": "action_state_changed" }).to_string();
    Some(Ok(Event::default().event("orchestrator").data(data)))
}
```

Place it after the `MilestoneUatCompleted` arm, before `Err(_) => None`.

**Acceptance:** The match is exhaustive (compiler confirms); event name is `"orchestrator"`, type field is `"action_state_changed"`.

---

## T3 — Write sentinel file in orchestrator daemon

**File:** `crates/sdlc-cli/src/cmd/orchestrate.rs`

Add `write_tick_sentinel` function (best-effort, logs to stderr):

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

Update `run_one_tick` to track dispatch counts and call the sentinel writer at the end:

- Capture `due.len()` as `actions_dispatched` before the dispatch loop.
- Capture `webhooks.len()` as `webhooks_dispatched` before the webhook loop.
- Call `write_tick_sentinel(root, actions_dispatched, webhooks_dispatched)` after both loops complete.

**Acceptance:** Calling `run_one_tick` against a temp dir creates `.sdlc/.orchestrator.state` with valid JSON containing `last_tick_at`.

---

## T4 — Unit test: sentinel file creation

**File:** `crates/sdlc-cli/tests/integration.rs` (or a new `orchestrate_tests.rs` module)

Add a test that:
1. Creates a `TempDir`.
2. Calls `run_one_tick` with an empty `ActionDb` (no actions, no webhooks).
3. Asserts `.sdlc/.orchestrator.state` exists.
4. Parses the file as JSON and asserts `last_tick_at` is a non-empty string.
5. Asserts `actions_dispatched == 0` and `webhooks_dispatched == 0`.

**Acceptance:** Test passes with `SDLC_NO_NPM=1 cargo test --all`.

---

## T5 — Verify exhaustiveness and build clean

Run:
```bash
SDLC_NO_NPM=1 cargo build --all 2>&1
SDLC_NO_NPM=1 cargo test --all 2>&1
cargo clippy --all -- -D warnings 2>&1
```

Fix any compilation errors from the exhaustive match on `SseMessage` in `events.rs` (the compiler will catch any missing arms).

**Acceptance:** All three commands exit 0 with no warnings or errors.
