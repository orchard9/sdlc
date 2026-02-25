use axum::extract::State;
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/vision — read VISION.md content.
pub async fn get_vision(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let path = sdlc_core::paths::vision_md_path(&root);
        let content = if path.exists() {
            std::fs::read_to_string(&path).unwrap_or_default()
        } else {
            String::new()
        };
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "content": content,
            "exists": path.exists(),
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct UpdateVisionBody {
    pub content: String,
}

/// PUT /api/vision — write VISION.md content.
pub async fn put_vision(
    State(app): State<AppState>,
    Json(body): Json<UpdateVisionBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let path = sdlc_core::paths::vision_md_path(&root);
        std::fs::write(&path, &body.content)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "ok": true,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
