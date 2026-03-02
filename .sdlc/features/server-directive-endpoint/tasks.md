# Tasks: server-directive-endpoint

## Task List

- [ ] Add `get_feature_directive` handler to `crates/sdlc-server/src/routes/features.rs` — classifies feature and returns `Json(classification)` (full serde serialization)
- [ ] Register `GET /api/features/{slug}/directive` route in `crates/sdlc-server/src/lib.rs`
- [ ] Add integration test verifying the endpoint returns all Classification fields including `description`
