use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct AddTaskBody {
    pub title: String,
}

/// POST /api/features/:slug/tasks — add a task to a feature.
pub async fn add_task(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<AddTaskBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        let id = sdlc_core::task::add_task(&mut feature.tasks, body.title);
        feature.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": slug,
            "task_id": id,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// POST /api/features/:slug/tasks/:id/start — start a task.
pub async fn start_task(
    State(app): State<AppState>,
    Path((slug, task_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        sdlc_core::task::start_task(&mut feature.tasks, &task_id)?;
        feature.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": slug,
            "task_id": task_id,
            "status": "in_progress",
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// POST /api/features/:slug/tasks/:id/complete — complete a task.
pub async fn complete_task(
    State(app): State<AppState>,
    Path((slug, task_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        sdlc_core::task::complete_task(&mut feature.tasks, &task_id)?;
        feature.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": slug,
            "task_id": task_id,
            "status": "completed",
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
