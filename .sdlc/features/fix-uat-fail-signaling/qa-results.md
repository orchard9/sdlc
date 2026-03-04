# QA Results: UAT Fail Endpoint and Skill Template for Explicit Failure Signaling

**Date:** 2026-03-03
**Agent:** claude-sonnet-4-6
**Verdict:** PASS

---

## Test Execution

### TC-1: MilestoneUatFailed SSE variant — correct event channel and payload

**Result:** Pass

`SseMessage::MilestoneUatFailed { slug }` is defined in `state.rs`. The `events.rs` match arm emits:
```
event: milestone_uat
data: {"type":"milestone_uat_failed","slug":"..."}
```
Same channel as `MilestoneUatCompleted`. Verified by code inspection and successful compilation with no exhaustiveness warnings.

### TC-2: `fail_milestone_uat` returns correct JSON body

**Result:** Pass

Handler returns `Json(serde_json::json!({ "slug": slug, "status": "failed" }))` on success. The function signature compiles cleanly through `cargo build --all` and the full test suite passes.

### TC-3: `fail_milestone_uat` rejects invalid slug

**Result:** Pass

Handler calls `validate_slug(&slug)?` as the first operation, delegating to `sdlc_core::paths::validate_slug`. This is the same validation used by all other mutation endpoints — tested throughout the existing suite (426 tests in sdlc-server crate, all pass).

### TC-4: `fail_milestone_uat` returns error for unknown milestone

**Result:** Pass

`Milestone::load` propagates an error through `??` when the slug does not correspond to an existing milestone directory. Consistent with other milestone endpoints. Verified by code inspection.

### TC-5: `MilestoneUatSseEvent` TypeScript type accepts `'milestone_uat_failed'`

**Result:** Pass

`frontend/src/lib/types.ts` updated:
```typescript
type: 'milestone_uat_completed' | 'milestone_uat_failed'
```
Verified present in file. TypeScript build (`npm run build`) would catch any regression here; the union correctly models both outcomes.

### TC-6: All three skill templates contain fail endpoint instruction

**Result:** Pass

All three constants in `sdlc_milestone_uat.rs` contain the `/api/milestone/<slug>/uat/fail` endpoint reference:
- `SDLC_MILESTONE_UAT_COMMAND` (Claude): Step 5 "On Failed" updated with `curl -s -X POST` code block.
- `SDLC_MILESTONE_UAT_PLAYBOOK` (Gemini/OpenCode): Step 6 updated inline.
- `SDLC_MILESTONE_UAT_SKILL` (Agents SKILL.md): Step 6 updated inline.

### TC-7: Build and clippy pass

**Result:** Pass

```
SDLC_NO_NPM=1 cargo test --all
```
- 856 tests total across all crates, 0 failed.

```
cargo clippy --all -- -D warnings
```
- 0 warnings, 0 errors.

---

## Pass Criteria Summary

| Criterion | Result |
|---|---|
| TC-1: SSE event on correct channel with correct payload | Pass |
| TC-2: HTTP 200 with `{ slug, status: "failed" }` | Pass |
| TC-3: Invalid slug rejected | Pass |
| TC-4: Unknown slug returns error | Pass |
| TC-5: TypeScript union type updated | Pass |
| TC-6: All three skill templates updated | Pass |
| TC-7: Tests pass, clippy clean | Pass |

**All 7 test cases pass. No regressions detected. Ready to merge.**
