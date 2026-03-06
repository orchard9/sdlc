# Spec: Fix Human UAT PassWithTasks Skips Release

## Problem

When a user submits a manual UAT with verdict "Pass with Tasks", the milestone stays in Verifying status instead of advancing to Released. The UI shows the milestone stuck — it doesn't archive or refresh. Only an exact "Pass" verdict triggers `milestone.release()`.

## Root Cause

In `submit_milestone_uat_human` (`crates/sdlc-server/src/routes/runs.rs:1235`), the release guard is:

```rust
if verdict == UatVerdict::Pass {
    milestone.release();
    milestone.save(&root)?;
}
```

This does not include `UatVerdict::PassWithTasks`. The `PassWithTasks` variant is a passing verdict (UAT succeeded, but follow-up tasks were created) and should also trigger milestone release.

Additionally, the notes-required validation at line 1165 treats `PassWithTasks` as a non-pass verdict, requiring notes — this is correct behavior since the notes should describe the tasks that need follow-up.

## Requirements

1. The release guard in `submit_milestone_uat_human` must treat both `UatVerdict::Pass` and `UatVerdict::PassWithTasks` as passing verdicts that trigger `milestone.release()`.
2. No changes to the notes validation — `PassWithTasks` should continue to require notes.
3. Add a test that submits a human UAT with `PassWithTasks` verdict and verifies the milestone gets `released_at` set.

## Scope

- **In scope**: Fix the conditional in `runs.rs`, add integration test
- **Out of scope**: Agent-driven UAT flow (uses a different code path), UI changes (already sends correct verdict values)
