# Code Review: human-uat-backend

## Summary

Implementation adds `UatRunMode` enum, a `mode` field to `UatRun`, two new REST endpoints (`POST /api/milestone/{slug}/uat/human` and `POST /api/features/{slug}/human-qa`), and 4 integration tests + 1 unit test.

## Files Changed

| File | Change Type |
|---|---|
| `crates/sdlc-core/src/milestone.rs` | New `UatRunMode` enum, `mode` field on `UatRun`, backward-compat test |
| `crates/sdlc-server/src/routes/runs.rs` | New `HumanUatBody` struct, `submit_milestone_uat_human` handler |
| `crates/sdlc-server/src/routes/features.rs` | New `HumanQaBody` struct, `submit_human_qa` handler |
| `crates/sdlc-server/src/lib.rs` | Two new route registrations |
| `crates/sdlc-server/tests/integration.rs` | 4 new integration tests |

## Review Findings

### Correctness

- `serde(default)` on `UatRunMode` ensures all existing `run.yaml` files load correctly without a `mode` field. Unit test `uat_run_mode_backward_compat` verifies this.
- `skip_serializing_if = "UatRunMode::is_agent"` prevents writing `mode: agent` to existing agent-generated files, keeping them clean.
- Validation logic for empty notes on non-pass verdicts is correct and returns 422.
- `milestone.release()` is called correctly on `Pass` verdict only.
- All I/O goes through `sdlc_core::io::atomic_write` — no direct file writes.

### No `unwrap()` in New Code

Verified: no `unwrap()` calls in the new production code. Test code uses `unwrap()` as is standard.

### API Design

- `POST /api/milestone/{slug}/uat/human` route is placed alongside existing UAT routes, consistent with the existing pattern.
- `POST /api/features/{slug}/human-qa` route follows the same structure as other feature action routes.
- Both handlers use `spawn_blocking` for all `sdlc-core` sync calls.

### SSE Events

- `submit_milestone_uat_human` emits `MilestoneUatCompleted` — same event as the agent UAT path, so the frontend refresh logic works identically for both paths.
- `submit_human_qa` emits `SseMessage::Update` — triggers project-wide state refresh so the feature card updates.

### Test Coverage

All 4 integration tests pass. The `human_uat_submit_pass` test verifies:
- 200 response
- `run.yaml` written at correct path
- `mode` field is `human`
- `milestone.released_at` is set

The `human_qa_submit_drafts_artifact` test verifies the full write-then-draft lifecycle.

### Build and Lint

- `SDLC_NO_NPM=1 cargo test --all` — all tests pass (0 failures)
- `cargo clippy --all -- -D warnings` — zero warnings

## Verdict

APPROVED. Implementation is clean, backward-compatible, fully tested, and consistent with existing patterns.
