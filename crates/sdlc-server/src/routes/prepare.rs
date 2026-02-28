use axum::extract::{Query, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/project/phase
pub async fn get_project_phase(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let phase = sdlc_core::prepare::project_phase(&root)?;
        serde_json::to_value(&phase).map_err(sdlc_core::SdlcError::Json)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct PrepareParams {
    pub milestone: Option<String>,
}

/// GET /api/project/prepare?milestone=x
pub async fn get_prepare(
    State(app): State<AppState>,
    Query(params): Query<PrepareParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let prepare_result = sdlc_core::prepare::prepare(&root, params.milestone.as_deref())?;
        serde_json::to_value(&prepare_result).map_err(sdlc_core::SdlcError::Json)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
