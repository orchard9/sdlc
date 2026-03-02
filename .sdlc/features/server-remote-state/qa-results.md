# QA Results: All state ops available over HTTP for remote consumers

## Summary

All QA checks pass. The two new HTTP endpoints are fully functional and covered by integration tests.

## Test Run

Command: `SDLC_NO_NPM=1 cargo test -p sdlc-server --test integration`

Result: **25 passed, 0 failed**

### New tests added for this feature

| Test | Result |
|---|---|
| `draft_artifact_ok` | PASS |
| `draft_artifact_feature_not_found` | PASS |
| `draft_artifact_invalid_type` | PASS |
| `merge_feature_ok` | PASS |
| `merge_feature_sets_phase_to_released` | PASS |
| `merge_feature_wrong_phase` | PASS |
| `merge_feature_not_found` | PASS |

All 7 new tests pass. All 18 pre-existing tests continue to pass.

## Lint

Command: `SDLC_NO_NPM=1 cargo clippy --all -- -D warnings`

Result: **No warnings or errors**

## Endpoint Verification

### `POST /api/artifacts/:slug/:type/draft`

- Returns `{"slug": ..., "artifact_type": ..., "status": "draft"}` on success
- Returns 404 when feature slug does not exist
- Returns 404 when artifact type is not a valid variant
- Calls `try_auto_transition` and includes `transitioned_to` in response when a phase transition occurs

### `POST /api/features/:slug/merge`

- Returns `{"slug": ..., "phase": "released", "merged": true}` on success
- Returns 400 when feature is not in `merge` phase
- Returns 404 when feature slug does not exist
- Persists `phase: released` in the feature manifest
- Records action in `state.yaml` and clears active directive

## Verdict

APPROVED — all QA checks pass, no regressions introduced.
