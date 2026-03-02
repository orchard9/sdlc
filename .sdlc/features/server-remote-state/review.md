# Code Review: All state ops available over HTTP for remote consumers

## Summary

This feature adds two HTTP endpoints to the sdlc-server that close the gap between the CLI and the HTTP API:

- `POST /api/artifacts/:slug/:type/draft` — marks an artifact as draft status
- `POST /api/features/:slug/merge` — finalizes the merge phase, transitioning to released

## Files Changed

- `crates/sdlc-server/src/routes/artifacts.rs` — added `draft_artifact` handler
- `crates/sdlc-server/src/routes/features.rs` — added `merge_feature` handler + `use` import
- `crates/sdlc-server/src/lib.rs` — registered two new routes
- `crates/sdlc-server/tests/integration.rs` — added 6 new integration tests

## Review Findings

### Correctness

All handlers follow the same `spawn_blocking` + `?` propagation pattern used throughout the codebase. No `unwrap()` calls in library or production code.

The `draft_artifact` handler correctly:
1. Loads the feature by slug
2. Parses the artifact type string to `ArtifactType` (errors map to 404 via `ArtifactNotFound`)
3. Calls `feature.mark_artifact_draft(at)`
4. Saves
5. Runs `try_auto_transition` and includes the result if non-null

The `merge_feature` handler correctly:
1. Loads config and feature
2. Guards on `feature.phase == Phase::Merge`, returning `SdlcError::InvalidPhase` (→ 400) if not
3. Calls `feature.transition(Phase::Released, &config)` and saves
4. Loads state, calls `record_action` and `complete_directive`, saves state
5. Returns `{ slug, phase: "released", merged: true }`

This matches the CLI implementation in `crates/sdlc-cli/src/cmd/merge.rs` exactly.

### Error Handling

| Scenario | HTTP Status | Mechanism |
|---|---|---|
| Feature not found | 404 | `SdlcError::FeatureNotFound` |
| Unknown artifact type | 404 | `SdlcError::ArtifactNotFound` |
| Feature not in merge phase | 400 | `SdlcError::InvalidPhase` |
| Spawn join error | 500 | explicit `anyhow::anyhow!` |

All errors are properly propagated through the existing `AppError` mapping with no new error types introduced.

### Tests

6 new integration tests cover:
- `draft_artifact_ok` — happy path, 200 with correct body
- `draft_artifact_feature_not_found` — 404
- `draft_artifact_invalid_type` — 404 (ArtifactNotFound)
- `merge_feature_ok` — happy path, 200 with phase: released
- `merge_feature_sets_phase_to_released` — verifies disk state after merge
- `merge_feature_wrong_phase` — 400 (feature not in merge phase)
- `merge_feature_not_found` — 404

All 25 integration tests pass. All 92 unit tests pass. `cargo clippy -- -D warnings` clean.

### Code Quality

- No `unwrap()` in production code
- No new error types needed
- Consistent with existing handler style
- Routes registered in logical position alongside sibling routes
- The `use sdlc_core::types::{ActionType, Phase};` import in `features.rs` is necessary and clean

### No Issues Found

The implementation is minimal, correct, and consistent with codebase conventions. No changes needed.

## Verdict

APPROVED — implementation is complete and correct.
