# Tasks: UAT Fail Endpoint and Skill Template for Explicit Failure Signaling

## T1 — Add `MilestoneUatFailed` SSE variant to state.rs

**File:** `crates/sdlc-server/src/state.rs`

Add `MilestoneUatFailed { slug: String }` to the `SseMessage` enum, directly after
`MilestoneUatCompleted`.

---

## T2 — Serialize `MilestoneUatFailed` in events.rs

**File:** `crates/sdlc-server/src/routes/events.rs`

Add match arm for `SseMessage::MilestoneUatFailed { slug }` that emits:
```
event: milestone_uat
data: {"type":"milestone_uat_failed","slug":"<slug>"}
```

---

## T3 — Implement `fail_milestone_uat` handler in runs.rs

**File:** `crates/sdlc-server/src/routes/runs.rs`

Add `pub async fn fail_milestone_uat` that:
1. Validates the slug.
2. Loads the milestone (blocking) to verify existence.
3. Broadcasts `SseMessage::MilestoneUatFailed { slug }`.
4. Returns `Json({ "slug": slug, "status": "failed" })`.

---

## T4 — Register `POST /api/milestone/:slug/uat/fail` route

**File:** `crates/sdlc-server/src/lib.rs`

Add route alongside the existing milestone UAT start/stop/events routes.

---

## T5 — Update `MilestoneUatSseEvent` TypeScript type

**File:** `frontend/src/lib/types.ts`

Extend the `type` union:
```typescript
type: 'milestone_uat_completed' | 'milestone_uat_failed'
```

---

## T6 — Update Claude UAT skill template (`SDLC_MILESTONE_UAT_COMMAND`)

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`

Update the "On Failed" branch in Step 5 to include:
```bash
curl -s -X POST http://localhost:7777/api/milestone/<slug>/uat/fail
```

---

## T7 — Update Gemini/OpenCode playbook (`SDLC_MILESTONE_UAT_PLAYBOOK`)

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`

Update step 6 of the playbook (On Failed path) with the fail endpoint curl call.

---

## T8 — Update Agents SKILL.md template (`SDLC_MILESTONE_UAT_SKILL`)

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_milestone_uat.rs`

Update step 6 of the SKILL.md workflow with the fail endpoint curl call.

---

## T9 — Verify build and tests pass

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```
