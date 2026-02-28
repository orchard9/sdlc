use axum::extract::State;
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/architecture — read ARCHITECTURE.md content.
pub async fn get_architecture(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let path = sdlc_core::paths::architecture_md_path(&root);
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
pub struct UpdateArchitectureBody {
    pub content: String,
}

/// PUT /api/architecture — write ARCHITECTURE.md content.
pub async fn put_architecture(
    State(app): State<AppState>,
    Json(body): Json<UpdateArchitectureBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let path = sdlc_core::paths::architecture_md_path(&root);
        std::fs::write(&path, &body.content)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "ok": true,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
