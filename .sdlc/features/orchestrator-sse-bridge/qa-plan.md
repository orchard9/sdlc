# QA Plan: Orchestrator SSE Bridge

## Scope

Verify that:
1. The sentinel file is written correctly by `run_one_tick`.
2. The `ActionStateChanged` SSE variant serializes to the correct wire format.
3. The server's mtime watcher fires `ActionStateChanged` when the sentinel file changes.
4. All existing tests continue to pass (no regression).
5. No `unwrap()` introduced in non-test code.

---

## QC-1 — Sentinel file written by `run_one_tick`

**Method:** Automated unit test (T4 from tasks.md).

**Steps:**
1. Create `TempDir` with `.sdlc/` subdirectory.
2. Open an `ActionDb` in that temp dir (empty, no actions).
3. Call `run_one_tick(root, &db)`.
4. Assert `.sdlc/.orchestrator.state` exists.
5. Read and parse as `serde_json::Value`.
6. Assert `last_tick_at` is a non-empty string.
7. Assert `actions_dispatched` == 0 and `webhooks_dispatched` == 0.

**Pass criteria:** Test exits 0; all assertions hold.

---

## QC-2 — Sentinel content after dispatching actions

**Method:** Automated unit test (add to same test module).

**Steps:**
1. Create `TempDir`. Open `ActionDb`.
2. Insert one scheduled `Action` with `next_tick_at = Utc::now() - 1s` (already due).
3. Create a stub tool script at `.sdlc/tools/<name>/tool.ts` that exits 0 with `{}`.
4. Call `run_one_tick`.
5. Assert sentinel exists and `actions_dispatched` == 1.

**Pass criteria:** `actions_dispatched` field equals the number of dispatched actions.

---

## QC-3 — SSE event serialization

**Method:** Code inspection + compile-time exhaustiveness check.

**Steps:**
1. After adding the `ActionStateChanged` arm to `events.rs`, run `cargo build --all`.
2. Confirm the compiler reports no missing arms in the `filter_map` match.
3. Inspect the arm: event name must be `"orchestrator"`, data JSON must contain `"type": "action_state_changed"`.

**Pass criteria:** Build succeeds; arm matches spec.

---

## QC-4 — Watcher fires on sentinel file change (integration)

**Method:** Automated integration test in `crates/sdlc-server/tests/integration.rs`.

**Steps:**
1. Build a test router with `build_router` (temp dir).
2. Subscribe to `app.event_tx` before writing the sentinel.
3. Write `.sdlc/.orchestrator.state` directly (simulating daemon).
4. Wait up to 2s for an `SseMessage::ActionStateChanged` on the receiver.
5. Assert the message was received within the timeout.

**Pass criteria:** `ActionStateChanged` received within 2s of sentinel write.

**Note:** This test only runs inside a tokio runtime (use `#[tokio::test]`). The watcher is guarded by `tokio::runtime::Handle::try_current().is_ok()` so it will be active.

---

## QC-5 — No regression: full test suite

**Method:** `SDLC_NO_NPM=1 cargo test --all`

**Pass criteria:** All tests pass; no new failures.

---

## QC-6 — Clippy clean

**Method:** `cargo clippy --all -- -D warnings`

**Pass criteria:** Zero warnings. Pay particular attention to:
- Unused variable warnings on `actions_dispatched`/`webhooks_dispatched` if counts are collected but not used.
- Dead code warnings on `write_tick_sentinel` (it should be used by `run_one_tick`).

---

## QC-7 — No `unwrap()` in non-test code

**Method:** Code grep.

```bash
grep -n "unwrap()" \
  crates/sdlc-server/src/state.rs \
  crates/sdlc-server/src/routes/events.rs \
  crates/sdlc-cli/src/cmd/orchestrate.rs
```

**Pass criteria:** Any `unwrap()` hits must be pre-existing (from before this feature). New code introduced by this feature must use `?`, `if let`, or `let _ =`.

---

## QC-8 — Sentinel file absent before first tick (server handles gracefully)

**Method:** Code inspection.

**Steps:**
1. Confirm the watcher uses `if let Ok(meta) = tokio::fs::metadata(...).await` — i.e., missing file is silently skipped.
2. Confirm no panic or error log when sentinel does not exist.

**Pass criteria:** Watcher loop continues without crashing when sentinel is absent.

---

## Run Order

QC-1 → QC-2 → QC-3 → QC-4 → QC-5 → QC-6 → QC-7 → QC-8

QC-5 (full test suite) subsumes QC-1, QC-2, and QC-4 if those tests are wired into the cargo test harness, which they must be.
