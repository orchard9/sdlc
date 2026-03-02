# Code Review: server-directive-endpoint

## Changes

**`crates/sdlc-server/src/routes/features.rs`**
- Added `get_feature_directive` handler — classifies the feature and returns `Json(classification)` using axum's typed JSON extractor directly on `Classification`
- This eliminates the manual `serde_json::json!{}` construction used in `get_feature_next`, ensuring all struct fields are automatically serialized by serde

**`crates/sdlc-server/src/lib.rs`**
- Registered `GET /api/features/{slug}/directive` route alongside the existing `/next` route

**`crates/sdlc-server/tests/integration.rs`**
- Added `get_feature_directive_returns_full_classification` — verifies all expected fields including `description`
- Added `get_feature_directive_returns_error_for_missing_feature` — verifies non-200 on unknown slug

## Verdict: Approved

- Implementation is minimal and correct
- `Classification` derives `Serialize` so `Json(classification)` works without any manual field wiring
- The existing `/next` endpoint is unchanged (backward compatible)
- Both new tests pass; full test suite passes clean
- No `unwrap()` in library code; errors propagate via `AppError`

## No Issues Found
