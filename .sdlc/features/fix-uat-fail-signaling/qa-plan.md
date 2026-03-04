# QA Plan: UAT Fail Endpoint and Skill Template for Explicit Failure Signaling

## Test Strategy

All verification is done through the Rust test suite and manual build checks.
No external services or running server needed — SSE serialization and endpoint
logic are unit-testable. Skill template content is verified by string assertions.

---

## TC-1: New `MilestoneUatFailed` SSE variant — correct event channel and payload

**What to verify:**
The `events.rs` serialization emits `event: milestone_uat` (same channel as
`MilestoneUatCompleted`) with `data: {"type":"milestone_uat_failed","slug":"<slug>"}`.

**Test:** Unit test in `runs.rs` or `events.rs` that constructs a `SseMessage::MilestoneUatFailed`
and asserts the JSON payload round-trips correctly.

---

## TC-2: `fail_milestone_uat` returns correct JSON body

**What to verify:**
`POST /api/milestone/:slug/uat/fail` returns HTTP 200 with:
```json
{ "slug": "<slug>", "status": "failed" }
```

**Test:** Unit test in `runs.rs` (follow existing pattern for similar endpoint tests).

---

## TC-3: `fail_milestone_uat` rejects invalid slug

**What to verify:**
Calling the endpoint with an invalid slug (e.g. containing spaces or special chars)
returns a 400 Bad Request from `validate_slug`.

**Test:** Integration assertion or unit test for slug validation.

---

## TC-4: `fail_milestone_uat` returns 404 for unknown milestone

**What to verify:**
Calling the endpoint with a syntactically valid but non-existent slug returns an
error response (milestone load fails).

**Note:** This is tested implicitly because `Milestone::load` returns an error
and the handler propagates it via `AppError`.

---

## TC-5: `MilestoneUatSseEvent` TypeScript type accepts `'milestone_uat_failed'`

**What to verify:**
The TypeScript type union includes `'milestone_uat_failed'`, so consumers can
write `if (event.type === 'milestone_uat_failed')` without a type error.

**Test:** Verified via TypeScript compilation (`npm run build` in `frontend/`).

---

## TC-6: Skill template instructs agent to call fail endpoint on Failed verdict

**What to verify:**
All three skill template constants (`SDLC_MILESTONE_UAT_COMMAND`,
`SDLC_MILESTONE_UAT_PLAYBOOK`, `SDLC_MILESTONE_UAT_SKILL`) contain the string
`/api/milestone/` and reference the fail endpoint in the "On Failed" section.

**Test:** String assertion in `sdlc_milestone_uat.rs` unit tests (existing file
already has a `start_milestone_uat_prompt_contains_screenshot_instructions` test —
add analogous coverage).

---

## TC-7: Build and clippy pass

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Both commands must exit 0.

---

## Pass Criteria

- TC-1 through TC-6 pass.
- TC-7: zero test failures, zero clippy warnings.
- No regression in existing milestone UAT tests (complete path still works).
