# Code Review: UAT Fail Endpoint and Skill Template for Explicit Failure Signaling

## Summary

The implementation is complete, minimal, and correct. All six targeted files were changed with surgical precision — no incidental sprawl. The build is clean: `SDLC_NO_NPM=1 cargo test --all` passes, `cargo clippy --all -- -D warnings` passes with zero warnings.

## File-by-File Review

### `crates/sdlc-server/src/state.rs`

**Change:** Added `MilestoneUatFailed { slug: String }` variant to `SseMessage` enum, placed logically after `MilestoneUatCompleted`.

**Assessment:** Correct placement, correct doc-comment style (matches surrounding variants), no issues.

### `crates/sdlc-server/src/routes/events.rs`

**Change:** Added match arm for `SseMessage::MilestoneUatFailed` that emits on the `milestone_uat` SSE channel with `type: "milestone_uat_failed"`.

**Assessment:** Uses the same `milestone_uat` event name as `MilestoneUatCompleted` — this is the intended design, keeping the frontend handler consolidated. Payload shape `{ type, slug }` mirrors the completed variant. No exhaustiveness warning (the new variant is covered). Correct.

### `crates/sdlc-server/src/routes/runs.rs`

**Change:** Added `pub async fn fail_milestone_uat` handler.

**Assessment:**
- Validates slug via `validate_slug` — early error on invalid input. Good.
- Loads milestone in `spawn_blocking` — correctly avoids blocking the async executor. Good.
- Propagates milestone load error with `??` — surfaces 404-equivalent on unknown slug. Good.
- Broadcasts `SseMessage::MilestoneUatFailed` with `let _ = ...` — correctly ignores the send error when no subscribers are connected (standard pattern in this codebase). Good.
- Returns `{ slug, status: "failed" }` as specified. Good.
- Handler is `pub` so lib.rs can reference it. Good.
- Idempotent by design — calling twice just emits two SSE events. Documented in the doc-comment. Good.

No issues.

### `crates/sdlc-server/src/lib.rs`

**Change:** Registered `POST /api/milestone/{slug}/uat/fail` route alongside the existing UAT start/stop/events routes.

**Assessment:** Correct placement (grouped with the other UAT endpoints). Uses `{slug}` path parameter syntax consistent with all other milestone routes. No issues.

### `frontend/src/lib/types.ts`

**Change:** Extended `MilestoneUatSseEvent.type` union from `'milestone_uat_completed'` to `'milestone_uat_completed' | 'milestone_uat_failed'`.

**Assessment:** Minimal, correct. Consumers can now discriminate on `type === 'milestone_uat_failed'` without TypeScript errors. No other frontend changes needed per spec. No issues.

### `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`

**Changes:** All three skill template constants updated:
- `SDLC_MILESTONE_UAT_COMMAND` (Claude): Step 5 "On Failed" now includes the `curl -X POST .../uat/fail` call with a code block.
- `SDLC_MILESTONE_UAT_PLAYBOOK` (Gemini/OpenCode): Step 6 updated inline.
- `SDLC_MILESTONE_UAT_SKILL` (Agents SKILL.md): Step 6 updated inline.

**Assessment:** All three templates consistently instruct agents to call the fail endpoint on a Failed verdict. The instruction is unambiguous: call the endpoint, leave milestone in Verifying, fix tasks and re-run. No template was missed. No issues.

## Acceptance Criteria Check

| Criterion | Status |
|---|---|
| `POST /api/milestone/:slug/uat/fail` returns 200 with `{ slug, status: "failed" }` | Done — handler returns exactly this |
| Calling the endpoint emits SSE on `milestone_uat` channel with `{ type: "milestone_uat_failed", slug }` | Done — events.rs match arm confirmed |
| `MilestoneUatSseEvent` TypeScript type accepts `'milestone_uat_failed'` | Done — union type updated |
| All three UAT skill templates instruct agent to call endpoint on Failed | Done — all three constants updated |
| `SDLC_NO_NPM=1 cargo test --all` passes | Pass |
| `cargo clippy --all -- -D warnings` passes | Pass — zero warnings |

## Verdict

**Approved.** Implementation matches spec and design exactly. No findings require action. The change is ready for audit.
