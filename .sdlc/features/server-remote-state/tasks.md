# Tasks: All state ops available over HTTP for remote consumers

## T1: Add `draft_artifact` handler to `artifacts.rs`

Add `pub async fn draft_artifact(...)` to `crates/sdlc-server/src/routes/artifacts.rs`.

- Accept `Path((slug, artifact_type)): Path<(String, String)>` and `State(app): State<AppState>`.
- In `spawn_blocking`: load feature, parse artifact type, call `feature.mark_artifact_draft(at)`, save feature, call `try_auto_transition`.
- Return `{ "slug", "artifact_type", "status": "draft", "transitioned_to"? }`.
- No request body needed.

## T2: Add `merge_feature` handler to `features.rs`

Add `pub async fn merge_feature(...)` to `crates/sdlc-server/src/routes/features.rs`.

- Accept `Path(slug): Path<String>` and `State(app): State<AppState>`.
- In `spawn_blocking`: load config, load feature, check `feature.phase == Phase::Merge` (return `SdlcError::InvalidPhase(...)` if not), transition to `Released`, save feature, load state, call `state.record_action(...)` and `state.complete_directive(...)`, save state.
- Return `{ "slug", "phase": "released", "merged": true }`.
- No request body needed.

## T3: Register both routes in `lib.rs`

In `crates/sdlc-server/src/lib.rs`, add route registrations:

- `POST /api/artifacts/{slug}/{artifact_type}/draft` → `routes::artifacts::draft_artifact`
- `POST /api/features/{slug}/merge` → `routes::features::merge_feature`

Place the draft route alongside the other artifact routes (approve/reject/waive).
Place the merge route alongside the transition route.

## T4: Add integration tests for `draft_artifact`

Add tests in the sdlc-server integration test module:

- `draft_artifact_ok` — create feature, POST draft on spec artifact, assert 200 and `status: draft`.
- `draft_artifact_feature_not_found` — POST draft for nonexistent slug, assert 404.
- `draft_artifact_invalid_type` — POST draft for `bogus-type`, assert 404 (ArtifactNotFound).

## T5: Add integration tests for `merge_feature`

Add tests in the sdlc-server integration test module:

- `merge_feature_ok` — create feature, transition to merge phase, POST merge, assert 200 and `phase: released`.
- `merge_feature_wrong_phase` — create feature (in draft phase), POST merge, assert 400.
- `merge_feature_not_found` — POST merge for nonexistent slug, assert 404.
