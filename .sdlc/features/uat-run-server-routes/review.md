# Code Review: Server Routes for UAT Run History and MilestoneUatCompleted SSE Event

## Summary

All 7 files changed as specified in the design. The implementation adds two HTTP GET endpoints for UAT run history, a new SSE event variant with its serialization arm, the completion event emission in the UAT agent spawn path, and the corresponding TypeScript types and API client methods. Both Rust and TypeScript builds pass cleanly.

## Changes Reviewed

### `crates/sdlc-server/src/state.rs`

Added `MilestoneUatCompleted { slug: String }` to the `SseMessage` enum, following the exact same pattern as `PonderRunCompleted`, `InvestigationRunCompleted`, `ToolBuildCompleted`, and other existing variants. The variant is documented with a clear doc comment.

**Verdict: Correct.** No issues.

### `crates/sdlc-server/src/routes/events.rs`

Added a match arm for `SseMessage::MilestoneUatCompleted { slug }` that serializes to:
```json
{ "type": "milestone_uat_completed", "slug": "<slug>" }
```
Events are emitted on the `"milestone_uat"` SSE event channel (consistent with how ponder events use `"ponder"`, investigation events use `"investigation"`, and advisory events use `"advisory"`). The pattern is an exact copy of existing arms with only the type string and field names changed.

**Verdict: Correct.** No issues.

### `crates/sdlc-server/src/routes/runs.rs`

`start_milestone_uat` now passes `Some(SseMessage::MilestoneUatCompleted { slug: slug.clone() })` as the `completion_event` argument to `spawn_agent_run`. The slug is cloned before the `format!` call consumes it into the prompt string. The completion_event variable is bound on its own line for readability, consistent with other callers that name their completion events.

**Verdict: Correct.** No issues.

### `crates/sdlc-server/src/routes/milestones.rs`

Two new handler functions added:

1. `list_milestone_uat_runs`: Returns `Json<Vec<sdlc_core::milestone::UatRun>>`. Uses `spawn_blocking`, propagates errors through `AppError`. Returns `200` with `[]` when no runs exist (the core function returns `Ok(Vec::new())` in that case).

2. `get_latest_milestone_uat_run`: Returns `Json<sdlc_core::milestone::UatRun>`. Uses `spawn_blocking`. Returns `404` via `AppError::not_found` when `latest_uat_run` returns `Ok(None)`. Uses pattern matching to distinguish `Some` / `None`.

Both functions follow the exact same structure as `get_milestone`, `review_milestone`, etc. No `unwrap()` calls. All error paths propagate through `?` or `AppError`.

**Verdict: Correct.** No issues.

### `crates/sdlc-server/src/lib.rs`

Two new routes registered in `build_router_from_state`:
- `GET /api/milestones/{slug}/uat-runs`
- `GET /api/milestones/{slug}/uat-runs/latest`

Both are placed after the existing milestone feature-reorder route and before the Roadmap routes. The `latest` route does not conflict with any wildcard patterns since axum matches static segments before dynamic ones.

**Verdict: Correct.** No issues.

### `frontend/src/lib/types.ts`

`UatVerdict` and `UatRun` added after the existing status types (`ArtifactStatus`, `TaskStatus`, `MilestoneStatus`), keeping all status-related types grouped together. The types exactly mirror the Rust struct's serde representation:
- `verdict` values match `#[serde(rename_all = "snake_case")]` on `UatVerdict`
- `completed_at` and `playwright_report_path` are `string | null` matching the Rust `Option<T>`
- `started_at` / `completed_at` are `string` (ISO 8601 from `DateTime<Utc>` serde)

**Verdict: Correct.** No issues.

### `frontend/src/api/client.ts`

`listMilestoneUatRuns` and `getLatestMilestoneUatRun` added immediately after the existing UAT control methods (`startMilestoneUat`, `stopMilestoneUat`), keeping UAT-related methods grouped. Both use `encodeURIComponent` for the slug. Return types use inline `import('@/lib/types')` consistent with all other methods in the file.

`getLatestMilestoneUatRun` is typed as `UatRun | null` — the `request` function throws on non-2xx responses (including 404), so callers must either catch the error or the union type signals "handle the null case". This matches the pattern used for other optional-result endpoints.

**Verdict: Correct.** No issues.

## Build Verification

- `SDLC_NO_NPM=1 cargo build --all` → `Finished \`dev\` profile` with no warnings or errors.
- `npx tsc --noEmit` → clean exit with no output.

## Code Quality

- No `unwrap()` in any new code.
- All fallible Rust calls use `?` with `AppError` propagation.
- All new functions have doc comments.
- TypeScript types are minimal, correct, and consistent with existing patterns.
- No logic was added to the Rust layer that belongs in a skill instruction (state machine rule is respected).

## Conclusion

The implementation is complete, correct, and clean. Ready to advance to audit.
