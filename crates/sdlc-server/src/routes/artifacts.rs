use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/artifacts/:slug/:type — artifact markdown content + status.
pub async fn get_artifact(
    State(app): State<AppState>,
    Path((slug, artifact_type)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        let at: sdlc_core::types::ArtifactType =
            artifact_type.parse().map_err(|e: sdlc_core::SdlcError| e)?;

        let artifact = feature
            .artifact(at)
            .ok_or_else(|| sdlc_core::SdlcError::ArtifactNotFound(artifact_type.clone()))?;

        // Read content from disk if it exists
        let content = if artifact.exists_on_disk(&root) {
            std::fs::read_to_string(root.join(&artifact.path)).ok()
        } else {
            None
        };

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "artifact_type": at,
            "status": artifact.status,
            "path": artifact.path,
            "content": content,
            "approved_at": artifact.approved_at,
            "approved_by": artifact.approved_by,
            "rejected_at": artifact.rejected_at,
            "rejection_reason": artifact.rejection_reason,
            "waived_at": artifact.waived_at,
            "waive_reason": artifact.waive_reason,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct ApproveBody {
    pub by: Option<String>,
}

/// POST /api/artifacts/:slug/:type/approve — approve an artifact.
pub async fn approve_artifact(
    State(app): State<AppState>,
    Path((slug, artifact_type)): Path<(String, String)>,
    Json(body): Json<ApproveBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        let at: sdlc_core::types::ArtifactType =
            artifact_type.parse().map_err(|e: sdlc_core::SdlcError| e)?;

        feature.approve_artifact(at, body.by)?;
        feature.save(&root)?;

        let transitioned_to = sdlc_core::classifier::try_auto_transition(&root, &slug);

        let mut val = serde_json::json!({
            "slug": slug,
            "artifact_type": at,
            "status": "approved",
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

#[derive(serde::Deserialize)]
pub struct RejectBody {
    pub reason: Option<String>,
}

/// POST /api/artifacts/:slug/:type/reject — reject an artifact.
pub async fn reject_artifact(
    State(app): State<AppState>,
    Path((slug, artifact_type)): Path<(String, String)>,
    Json(body): Json<RejectBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        let at: sdlc_core::types::ArtifactType =
            artifact_type.parse().map_err(|e: sdlc_core::SdlcError| e)?;

        feature.reject_artifact(at, body.reason)?;
        feature.save(&root)?;

        let transitioned_to = sdlc_core::classifier::try_auto_transition(&root, &slug);

        let mut val = serde_json::json!({
            "slug": slug,
            "artifact_type": at,
            "status": "rejected",
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

#[derive(serde::Deserialize)]
pub struct WaiveBody {
    pub reason: Option<String>,
}

/// POST /api/artifacts/:slug/:type/waive — waive an artifact.
pub async fn waive_artifact(
    State(app): State<AppState>,
    Path((slug, artifact_type)): Path<(String, String)>,
    Json(body): Json<WaiveBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        let at: sdlc_core::types::ArtifactType =
            artifact_type.parse().map_err(|e: sdlc_core::SdlcError| e)?;

        feature.waive_artifact(at, body.reason)?;
        feature.save(&root)?;

        let transitioned_to = sdlc_core::classifier::try_auto_transition(&root, &slug);

        let mut val = serde_json::json!({
            "slug": slug,
            "artifact_type": at,
            "status": "waived",
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
