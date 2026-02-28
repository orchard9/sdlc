use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Session route parameter types
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct SessionPath {
    pub slug: String,
    pub n: u32,
}

/// GET /api/roadmap — list all ponder entries with artifact counts and team size.
pub async fn list_ponders(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entries = sdlc_core::ponder::PonderEntry::list(&root)?;
        let list: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                let artifact_count = sdlc_core::ponder::list_artifacts(&root, &e.slug)
                    .map(|a| a.len())
                    .unwrap_or(0);
                let team_size = sdlc_core::ponder::load_team(&root, &e.slug)
                    .map(|t| t.partners.len())
                    .unwrap_or(0);
                serde_json::json!({
                    "slug": e.slug,
                    "title": e.title,
                    "status": e.status.to_string(),
                    "tags": e.tags,
                    "sessions": e.sessions,
                    "artifact_count": artifact_count,
                    "team_size": team_size,
                    "created_at": e.created_at,
                    "updated_at": e.updated_at,
                    "committed_at": e.committed_at,
                    "committed_to": e.committed_to,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/roadmap/:slug — full detail: manifest + team + artifacts.
pub async fn get_ponder(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entry = sdlc_core::ponder::PonderEntry::load(&root, &slug)?;
        let team = sdlc_core::ponder::load_team(&root, &slug)?;
        let artifacts = sdlc_core::ponder::list_artifacts(&root, &slug)?;

        let artifact_list: Vec<serde_json::Value> = artifacts
            .iter()
            .map(|a| {
                let content = sdlc_core::ponder::read_artifact(&root, &slug, &a.filename).ok();
                serde_json::json!({
                    "filename": a.filename,
                    "size_bytes": a.size_bytes,
                    "modified_at": a.modified_at,
                    "content": content,
                })
            })
            .collect();

        let orientation = entry.orientation.as_ref().map(|o| {
            serde_json::json!({
                "current": o.current,
                "next": o.next,
                "commit": o.commit,
            })
        });

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "status": entry.status.to_string(),
            "tags": entry.tags,
            "sessions": entry.sessions,
            "orientation": orientation,
            "committed_at": entry.committed_at,
            "committed_to": entry.committed_to,
            "created_at": entry.created_at,
            "updated_at": entry.updated_at,
            "team": team.partners,
            "artifacts": artifact_list,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct CreatePonderBody {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub brief: Option<String>,
}

/// POST /api/roadmap — create a new ponder entry.
pub async fn create_ponder(
    State(app): State<AppState>,
    Json(body): Json<CreatePonderBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entry = sdlc_core::ponder::PonderEntry::create(&root, body.slug, body.title)?;

        if let Some(brief) = body.brief {
            sdlc_core::ponder::capture_content(&root, &entry.slug, "brief.md", &brief)?;
        }

        if let Ok(mut state) = sdlc_core::state::State::load(&root) {
            state.add_ponder(&entry.slug);
            let _ = state.save(&root);
        }

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "status": entry.status.to_string(),
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct CaptureArtifactBody {
    pub filename: String,
    pub content: String,
}

/// POST /api/roadmap/:slug/capture — capture content into scrapbook.
pub async fn capture_artifact(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<CaptureArtifactBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::ponder::capture_content(&root, &slug, &body.filename, &body.content)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": slug,
            "filename": body.filename,
            "captured": true,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct UpdatePonderBody {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// PUT /api/roadmap/:slug — update status/title/tags.
pub async fn update_ponder(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<UpdatePonderBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut entry = sdlc_core::ponder::PonderEntry::load(&root, &slug)?;

        if let Some(status_str) = body.status {
            let status: sdlc_core::ponder::PonderStatus = status_str.parse()?;
            entry.update_status(status);
        }
        if let Some(title) = body.title {
            entry.update_title(title);
        }
        if let Some(tags) = body.tags {
            entry.set_tags(tags);
        }

        entry.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "status": entry.status.to_string(),
            "tags": entry.tags,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/roadmap/:slug/sessions — list session metadata, sorted by session number.
pub async fn list_ponder_sessions(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let sessions = sdlc_core::ponder::list_sessions(&root, &slug)?;
        let list: Vec<serde_json::Value> = sessions
            .iter()
            .map(|s| {
                let orientation = s.orientation.as_ref().map(|o| {
                    serde_json::json!({
                        "current": o.current,
                        "next": o.next,
                        "commit": o.commit,
                    })
                });
                serde_json::json!({
                    "session": s.session,
                    "timestamp": s.timestamp,
                    "orientation": orientation,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/roadmap/:slug/sessions/:n — full content of a single session.
pub async fn get_ponder_session(
    State(app): State<AppState>,
    Path(SessionPath { slug, n }): Path<SessionPath>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let content = sdlc_core::ponder::read_session(&root, &slug, n)?;
        let meta = sdlc_core::ponder::parse_session_meta(&content);
        let orientation = meta.as_ref().and_then(|m| m.orientation.as_ref()).map(|o| {
            serde_json::json!({
                "current": o.current,
                "next": o.next,
                "commit": o.commit,
            })
        });
        let timestamp = meta.as_ref().map(|m| m.timestamp);
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "session": n,
            "timestamp": timestamp,
            "orientation": orientation,
            "content": content,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
