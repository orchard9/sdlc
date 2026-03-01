# Tasks: Server Routes for UAT Run History and MilestoneUatCompleted SSE Event

## Task Breakdown

### Task 1: Add MilestoneUatCompleted SSE variant to state.rs

Add `MilestoneUatCompleted { slug: String }` to the `SseMessage` enum in `crates/sdlc-server/src/state.rs`. This is the foundational change that all other SSE-related work depends on.

**Files:** `crates/sdlc-server/src/state.rs`

---

### Task 2: Wire MilestoneUatCompleted serialization in events.rs

Add the pattern-match arm in `crates/sdlc-server/src/routes/events.rs` to serialize `SseMessage::MilestoneUatCompleted { slug }` into `{ "type": "MilestoneUatCompleted", "slug": slug }`.

**Files:** `crates/sdlc-server/src/routes/events.rs`
**Depends on:** Task 1

---

### Task 3: Emit MilestoneUatCompleted in start_milestone_uat

Update `start_milestone_uat` in `crates/sdlc-server/src/routes/runs.rs` to pass `Some(SseMessage::MilestoneUatCompleted { slug: slug.clone() })` as the `completion_event` argument to `spawn_agent_run` instead of `None`.

**Files:** `crates/sdlc-server/src/routes/runs.rs`
**Depends on:** Task 1

---

### Task 4: Add HTTP route handlers for UAT run history

Add `list_milestone_uat_runs` and `get_latest_milestone_uat_run` handler functions to `crates/sdlc-server/src/routes/milestones.rs`. Both use `spawn_blocking` and call `sdlc_core::milestone::list_uat_runs` / `latest_uat_run`.

**Files:** `crates/sdlc-server/src/routes/milestones.rs`

---

### Task 5: Register new routes in lib.rs

Register `GET /api/milestones/{slug}/uat-runs` and `GET /api/milestones/{slug}/uat-runs/latest` in the `build_router_from_state` function in `crates/sdlc-server/src/lib.rs`.

**Files:** `crates/sdlc-server/src/lib.rs`
**Depends on:** Task 4

---

### Task 6: Add UatVerdict and UatRun TypeScript types

Add `UatVerdict` and `UatRun` to `frontend/src/lib/types.ts` matching the Rust struct's serde representation.

**Files:** `frontend/src/lib/types.ts`

---

### Task 7: Add API client methods for UAT run history

Add `listMilestoneUatRuns` and `getLatestMilestoneUatRun` to the `api` object in `frontend/src/api/client.ts`.

**Files:** `frontend/src/api/client.ts`
**Depends on:** Task 6

---

### Task 8: Verify build quality

Run `SDLC_NO_NPM=1 cargo build --all` to confirm Rust compiles clean. Run `cd frontend && npx tsc --noEmit` to confirm TypeScript compiles clean.

**Depends on:** Tasks 1-7
