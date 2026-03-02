# QA Plan: All state ops available over HTTP for remote consumers

## Scope

Verify that the two new HTTP endpoints — `POST /api/artifacts/:slug/:type/draft` and `POST /api/features/:slug/merge` — are correctly implemented, properly registered, and behave identically to their CLI counterparts.

## Build Verification

- [ ] `SDLC_NO_NPM=1 cargo build --all` succeeds with no errors.
- [ ] `cargo clippy --all -- -D warnings` produces no warnings.

## Integration Tests (cargo test)

Run: `SDLC_NO_NPM=1 cargo test --all`

### draft_artifact endpoint

- [ ] `draft_artifact_ok` — POST to `/api/artifacts/{slug}/spec/draft` on a newly created feature returns 200 with body `{ "slug": ..., "artifact_type": "spec", "status": "draft" }`.
- [ ] `draft_artifact_sets_phase_transition` — after drafting, if that makes all required artifacts for the phase present, `transitioned_to` appears in the response body.
- [ ] `draft_artifact_feature_not_found` — POST for a non-existent slug returns 404.
- [ ] `draft_artifact_invalid_artifact_type` — POST with `artifact_type = "bogus"` returns 404.

### merge_feature endpoint

- [ ] `merge_feature_ok` — feature transitioned manually to merge phase, POST `/api/features/{slug}/merge` returns 200 with `{ "slug": ..., "phase": "released", "merged": true }`.
- [ ] `merge_feature_sets_state_released` — after merge, GET `/api/features/{slug}` returns `phase: "released"`.
- [ ] `merge_feature_wrong_phase` — feature not in merge phase, POST returns 400.
- [ ] `merge_feature_feature_not_found` — POST for non-existent slug returns 404.

## Manual Smoke Test (optional)

With `sdlc ui` running:

1. Create a test feature via `sdlc feature create qa-smoke-test "QA smoke"`.
2. `curl -X POST http://localhost:3141/api/artifacts/qa-smoke-test/spec/draft` → expect 200, status: draft.
3. Transition feature to merge phase: `sdlc feature transition qa-smoke-test merge`.
4. `curl -X POST http://localhost:3141/api/features/qa-smoke-test/merge` → expect 200, phase: released.
5. `curl http://localhost:3141/api/features/qa-smoke-test` → confirm `phase: "released"`.

## Acceptance Criteria

All `cargo test --all` tests pass, `cargo clippy` clean, and both endpoints work end-to-end as described above.
