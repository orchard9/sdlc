# Design: All state ops available over HTTP for remote consumers

## Overview

Two endpoints are added to close the gap between the CLI and the HTTP API for remote consumers. Both follow exact existing patterns in the codebase.

## Endpoint 1: POST /api/artifacts/:slug/:type/draft

### Handler Location

`crates/sdlc-server/src/routes/artifacts.rs` — new `draft_artifact` function alongside the existing `approve_artifact`, `reject_artifact`, and `waive_artifact` functions.

### Implementation

```rust
/// POST /api/artifacts/:slug/:type/draft — mark an artifact as draft.
pub async fn draft_artifact(
    State(app): State<AppState>,
    Path((slug, artifact_type)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        let at: sdlc_core::types::ArtifactType =
            artifact_type.parse().map_err(|e: sdlc_core::SdlcError| e)?;

        feature.mark_artifact_draft(at)?;
        feature.save(&root)?;

        let transitioned_to = sdlc_core::classifier::try_auto_transition(&root, &slug);

        let mut val = serde_json::json!({
            "slug": slug,
            "artifact_type": at,
            "status": "draft",
        });
        if let Some(phase) = transitioned_to {
            val["transitioned_to"] = serde_json::Value::String(phase);
        }
        Ok::<_, sdlc_core::SdlcError>(val)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
```

No request body is needed — the path parameters carry all the necessary information.

### Route Registration

In `crates/sdlc-server/src/lib.rs`, add alongside the artifact approve/reject/waive routes:

```rust
.route(
    "/api/artifacts/{slug}/{artifact_type}/draft",
    post(routes::artifacts::draft_artifact),
)
```

## Endpoint 2: POST /api/features/:slug/merge

### Handler Location

`crates/sdlc-server/src/routes/features.rs` — new `merge_feature` function.

### Implementation

```rust
/// POST /api/features/:slug/merge — finalize the merge phase.
pub async fn merge_feature(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::config::Config::load(&root)?;
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;

        if feature.phase != sdlc_core::types::Phase::Merge {
            return Err(AppError::bad_request(format!(
                "cannot finalize merge for '{slug}' from phase '{}'; move it to 'merge' first",
                feature.phase
            )).0.into());
        }

        feature
            .transition(sdlc_core::types::Phase::Released, &config)?;
        feature.save(&root)?;

        let mut state = sdlc_core::state::State::load(&root)?;
        state.record_action(
            &slug,
            sdlc_core::types::ActionType::Merge,
            sdlc_core::types::Phase::Released,
            "merged",
        );
        state.complete_directive(&slug);
        state.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": slug,
            "phase": "released",
            "merged": true,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
```

**Note on 400 vs 500 for phase precondition failure:** The phase check happens inside `spawn_blocking`. When the feature is not in `merge` phase, we return an `SdlcError`-based error. The existing `AppError` mapping for `SdlcError::InvalidSlug` already maps to 400. However, we need to emit a descriptive error without misusing `InvalidSlug`. The cleanest approach is to use `anyhow::bail!` with the message and rely on the 500 fallback, or better: use `AppError::bad_request()` by converting it to an `anyhow::Error` directly in the spawn_blocking closure and returning `Err(anyhow::anyhow!(...))`.

The revised approach for the phase check:

```rust
if feature.phase != sdlc_core::types::Phase::Merge {
    return Err(sdlc_core::SdlcError::InvalidPhase(format!(
        "cannot finalize merge for '{}' from phase '{}'; move it to 'merge' first",
        slug, feature.phase
    )));
}
```

`SdlcError::InvalidPhase` maps to `StatusCode::BAD_REQUEST` in `AppError::into_response`, so this cleanly returns a 400 with no new error types needed.

### Route Registration

In `crates/sdlc-server/src/lib.rs`, add alongside the transition route:

```rust
.route(
    "/api/features/{slug}/merge",
    post(routes::features::merge_feature),
)
```

## File Change Summary

| File | Change |
|---|---|
| `crates/sdlc-server/src/routes/artifacts.rs` | Add `draft_artifact` function |
| `crates/sdlc-server/src/routes/features.rs` | Add `merge_feature` function |
| `crates/sdlc-server/src/lib.rs` | Register both new routes |

## Error Handling

| Scenario | Status Code | Mechanism |
|---|---|---|
| Feature not found | 404 | `SdlcError::FeatureNotFound` → existing mapping |
| Unknown artifact type | 404 | `SdlcError::ArtifactNotFound` → existing mapping |
| Feature not in merge phase | 400 | `SdlcError::InvalidPhase` → existing mapping |
| I/O failure | 500 | `SdlcError::Io` → existing mapping |
| Task join error | 500 | explicit `anyhow::anyhow!` → 500 fallback |

No new error types or `AppError` helpers are needed.

## Tests

Integration tests are added to `crates/sdlc-server/tests/` (or `crates/sdlc-server/src/` integration test module) covering:

1. `draft_artifact` — happy path (200 + correct body)
2. `draft_artifact` — feature not found (404)
3. `draft_artifact` — invalid artifact type (404)
4. `merge_feature` — happy path (200 + `phase: released`)
5. `merge_feature` — feature not in merge phase (400)
6. `merge_feature` — feature not found (404)
