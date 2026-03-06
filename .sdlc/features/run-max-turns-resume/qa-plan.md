# QA Plan: Paused (turn limit) UX and Resume action for investigation and ponder runs

## Scope

Verify that:
1. `ErrorMaxTurns` runs are classified as `paused` (not `failed`) in the RunRecord.
2. `session_id` and `stop_reason` are captured and stored.
3. Resume endpoints accept a paused run_id and start a new agent with `opts.resume` set.
4. The frontend RunCard displays the amber pause icon, label suffix, and Resume button.
5. Backward compat: old RunRecord JSON without new fields deserializes without panic.

## Test Cases

### TC-1: RunRecord status for ErrorMaxTurns

**Method:** Unit test in `runs.rs` `mod tests`

**Steps:**
1. Construct a `Message::Result(ResultMessage::ErrorMaxTurns(ResultError { session_id: "sid-123", stop_reason: Some("max_turns"), ... }))`.
2. Pass through the status determination logic.
3. Assert `status == "paused"`.
4. Assert `session_id == Some("sid-123")`.
5. Assert `stop_reason == Some("max_turns")`.
6. Assert `error == None` (not populated for max_turns).

**Pass criteria:** All assertions hold.

### TC-2: RunRecord status for normal completion

**Method:** Unit test

**Steps:** Same setup with `ResultMessage::Success`. Assert `status == "completed"`, `session_id` set, `stop_reason = Some("end_turn")` (or whatever the fixture provides).

### TC-3: RunRecord status for execution error

**Method:** Unit test

**Steps:** `ResultMessage::ErrorDuringExecution`. Assert `status == "failed"`, `error` is set.

### TC-4: Backward compat deserialization

**Method:** Unit test

**Steps:**
1. Deserialize a raw JSON string representing an old RunRecord without `session_id` or `stop_reason`.
2. Assert no panic/error.
3. Assert `session_id == None`, `stop_reason == None`.

### TC-5: Resume endpoint — happy path (ponder)

**Method:** Integration test with `build_router_for_test`

**Steps:**
1. Insert a RunRecord with `status = "paused"`, `session_id = Some("sid-abc")`, `key = "ponder:test-slug"` into run_history.
2. `POST /api/ponder/test-slug/chat/resume` with `{ "run_id": "<id>" }`.
3. Assert response is `200` with `{ "status": "started" }`.
4. Assert `agent_runs` now contains `"ponder:test-slug"` (agent was spawned).

**Pass criteria:** 200 response and run key present in active runs.

### TC-6: Resume endpoint — not paused (400)

**Method:** Integration test

**Steps:**
1. Insert a RunRecord with `status = "completed"`, `session_id = Some("sid-abc")`.
2. `POST /api/ponder/test-slug/chat/resume`.
3. Assert 400 response.

### TC-7: Resume endpoint — no session_id (400)

**Method:** Integration test

**Steps:**
1. Insert RunRecord with `status = "paused"`, `session_id = None`.
2. `POST /api/ponder/test-slug/chat/resume`.
3. Assert 400 response.

### TC-8: Resume endpoint — already running (409)

**Method:** Integration test

**Steps:**
1. Insert paused RunRecord with session_id.
2. Insert active run under `"ponder:test-slug"` key in agent_runs.
3. `POST /api/ponder/test-slug/chat/resume`.
4. Assert 409 (via spawn_agent_run conflict guard).

### TC-9: Frontend — paused status icon and label

**Method:** Vitest component test or manual QA

**Steps:**
1. Render `<RunCard>` with a run fixture where `status = "paused"`.
2. Assert `PauseCircle` icon rendered (or class `text-amber-400` present).
3. Assert label text includes `"paused (turn limit)"`.
4. Assert Resume button (Play icon) is visible.
5. Assert Stop button (Square icon) is NOT visible.

### TC-10: Frontend — Resume button calls correct endpoint

**Method:** Vitest component test with fetch mock

**Steps:**
1. Render `<RunCard>` with paused ponder run (`key = "ponder:my-idea"`, `target = "my-idea"`).
2. Click Resume button.
3. Assert `fetch` called with `POST /api/ponder/my-idea/chat/resume` and correct body.

### TC-11: Frontend — non-resumable run type hides Resume button

**Method:** Component test

**Steps:**
1. Render `<RunCard>` with paused run where `key = "sdlc-run:my-feature"`.
2. Assert Resume button is NOT rendered.

## Build Verification

After implementation:

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
cd frontend && npm run build
```

All three must pass with zero errors.
