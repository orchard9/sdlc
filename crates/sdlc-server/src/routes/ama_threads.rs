use axum::extract::{Path, Query, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;
use sdlc_core::ama_thread::{self, AmaThread, AmaTurn};

// ---------------------------------------------------------------------------
// GET /api/tools/ama/threads — list threads
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, Default)]
pub struct ThreadListParams {
    limit: Option<usize>,
}

pub async fn list_ama_threads(
    State(app): State<AppState>,
    Query(params): Query<ThreadListParams>,
) -> Result<Json<Vec<AmaThread>>, AppError> {
    let root = app.root.clone();
    let limit = params.limit.unwrap_or(50);
    let result = tokio::task::spawn_blocking(move || {
        ama_thread::list_threads(&root, limit).map_err(|e| AppError(e.into()))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/tools/ama/threads — create thread
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CreateThreadBody {
    pub id: String,
    pub title: String,
}

pub async fn create_ama_thread(
    State(app): State<AppState>,
    Json(body): Json<CreateThreadBody>,
) -> Result<Json<AmaThread>, AppError> {
    if body.id.is_empty() || body.title.is_empty() {
        return Err(AppError::bad_request("id and title must not be empty"));
    }
    let root = app.root.clone();
    let id = body.id.clone();
    let title = body.title.clone();
    let result = tokio::task::spawn_blocking(move || {
        ama_thread::create_thread(&root, &id, &title).map_err(|e| AppError(e.into()))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/tools/ama/threads/:id — thread detail with turns
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
pub struct AmaThreadDetail {
    #[serde(flatten)]
    pub thread: AmaThread,
    pub turns: Vec<AmaTurn>,
}

pub async fn get_ama_thread(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AmaThreadDetail>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<AmaThreadDetail, AppError> {
        let thread = ama_thread::load_thread(&root, &id)
            .map_err(|_| AppError::not_found(format!("AMA thread '{id}' not found")))?;
        let turns = ama_thread::list_turns(&root, &id).map_err(|e| AppError(e.into()))?;
        Ok(AmaThreadDetail { thread, turns })
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// PATCH /api/tools/ama/threads/:id — update title / tags / committed_to
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct UpdateThreadBody {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub committed_to: Option<String>,
}

pub async fn update_ama_thread(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateThreadBody>,
) -> Result<Json<AmaThread>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<AmaThread, AppError> {
        let mut thread = ama_thread::load_thread(&root, &id)
            .map_err(|_| AppError::not_found(format!("AMA thread '{id}' not found")))?;
        if let Some(title) = body.title {
            thread.title = title;
        }
        if let Some(tags) = body.tags {
            thread.tags = tags;
        }
        if let Some(committed_to) = body.committed_to {
            thread.committed_to = Some(committed_to);
        }
        thread.updated_at = chrono::Utc::now().to_rfc3339();
        ama_thread::save_thread(&root, &thread).map_err(|e| AppError(e.into()))?;
        Ok(thread)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// DELETE /api/tools/ama/threads/:id
// ---------------------------------------------------------------------------

pub async fn delete_ama_thread(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    tokio::task::spawn_blocking(move || {
        ama_thread::delete_thread(&root, &id)
            .map_err(|_| AppError::not_found(format!("AMA thread '{id}' not found")))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------------------------
// PATCH /api/tools/ama/threads/:id/turns/:n — update synthesis
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct UpdateTurnBody {
    pub synthesis: String,
}

pub async fn update_ama_turn_synthesis(
    State(app): State<AppState>,
    Path((id, n)): Path<(String, u32)>,
    Json(body): Json<UpdateTurnBody>,
) -> Result<Json<AmaTurn>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<AmaTurn, AppError> {
        ama_thread::update_turn_synthesis(&root, &id, n, &body.synthesis)
            .map_err(|e| AppError(e.into()))?;
        // Also bump thread updated_at
        if let Ok(mut thread) = ama_thread::load_thread(&root, &id) {
            thread.updated_at = chrono::Utc::now().to_rfc3339();
            let _ = ama_thread::save_thread(&root, &thread);
        }
        ama_thread::load_turn(&root, &id, n).map_err(|e| AppError(e.into()))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/tools/ama/threads/:id/turns — add a new turn (question + sources)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct AddTurnBody {
    pub question: String,
    pub sources: Vec<serde_json::Value>,
    pub run_id: Option<String>,
}

pub async fn add_ama_turn(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AddTurnBody>,
) -> Result<Json<AmaTurn>, AppError> {
    if body.question.trim().is_empty() {
        return Err(AppError::bad_request("question must not be empty"));
    }
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<AmaTurn, AppError> {
        let mut thread = ama_thread::load_thread(&root, &id)
            .map_err(|_| AppError::not_found(format!("AMA thread '{id}' not found")))?;
        let turn_index = thread.turn_count;
        let turn = AmaTurn {
            turn_index,
            question: body.question.clone(),
            sources: body.sources.clone(),
            synthesis: None,
            run_id: body.run_id.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        };
        ama_thread::save_turn(&root, &id, &turn).map_err(|e| AppError(e.into()))?;
        thread.turn_count += 1;
        thread.updated_at = chrono::Utc::now().to_rfc3339();
        ama_thread::save_thread(&root, &thread).map_err(|e| AppError(e.into()))?;
        Ok(turn)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}
