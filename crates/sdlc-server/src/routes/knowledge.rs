use axum::extract::{Path, Query, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Parameter types
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct SessionPath {
    pub slug: String,
    pub n: u32,
}

#[derive(serde::Deserialize, Default)]
pub struct ListKnowledgeQuery {
    pub code: Option<String>,
    pub tag: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/knowledge/catalog
// ---------------------------------------------------------------------------

pub async fn get_catalog(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let catalog = sdlc_core::knowledge::load_catalog(&root)?;
        let classes: Vec<serde_json::Value> = catalog
            .classes
            .iter()
            .map(|c| {
                let divisions: Vec<serde_json::Value> = c
                    .divisions
                    .iter()
                    .map(|d| {
                        serde_json::json!({
                            "code": d.code,
                            "name": d.name,
                            "description": d.description,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "code": c.code,
                    "name": c.name,
                    "description": c.description,
                    "divisions": divisions,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "classes": classes,
            "updated_at": catalog.updated_at,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/knowledge
// ---------------------------------------------------------------------------

pub async fn list_knowledge(
    State(app): State<AppState>,
    Query(params): Query<ListKnowledgeQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut entries = match params.code.as_deref() {
            Some(prefix) => sdlc_core::knowledge::list_by_code_prefix(&root, prefix)?,
            None => sdlc_core::knowledge::list(&root)?,
        };

        if let Some(tag) = params.tag.as_deref() {
            entries.retain(|e| e.tags.iter().any(|t| t == tag));
        }

        let list: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "slug": e.slug,
                    "title": e.title,
                    "code": e.code,
                    "status": e.status.to_string(),
                    "summary": e.summary,
                    "tags": e.tags,
                    "created_at": e.created_at,
                    "updated_at": e.updated_at,
                })
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/knowledge
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CreateKnowledgeBody {
    pub slug: Option<String>,
    pub title: String,
    #[serde(default = "default_code")]
    pub code: String,
    #[serde(default)]
    pub content: Option<String>,
}

fn default_code() -> String {
    "uncategorized".to_string()
}

pub async fn create_knowledge(
    State(app): State<AppState>,
    Json(body): Json<CreateKnowledgeBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let slug = body
            .slug
            .unwrap_or_else(|| slugify_title_server(&body.title));

        let entry = sdlc_core::knowledge::create(&root, &slug, &body.title, &body.code)?;

        if let Some(text) = body.content.as_deref() {
            sdlc_core::knowledge::append_content(&root, &slug, text)?;
        }

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "code": entry.code,
            "status": entry.status.to_string(),
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/knowledge/:slug
// ---------------------------------------------------------------------------

pub async fn get_knowledge(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entry = sdlc_core::knowledge::load(&root, &slug)?;
        let content = sdlc_core::knowledge::read_content(&root, &slug).unwrap_or_default();
        let artifacts = sdlc_core::knowledge::list_named_artifacts(&root, &slug)?;

        let artifact_list: Vec<serde_json::Value> = artifacts
            .iter()
            .map(|a| {
                serde_json::json!({
                    "filename": a.filename,
                    "size_bytes": a.size_bytes,
                    "modified_at": a.modified_at,
                })
            })
            .collect();

        let sources: Vec<serde_json::Value> = entry
            .sources
            .iter()
            .map(|s| {
                serde_json::json!({
                    "type": s.source_type.to_string(),
                    "url": s.url,
                    "path": s.path,
                    "workspace": s.workspace,
                    "captured_at": s.captured_at,
                })
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "code": entry.code,
            "status": entry.status.to_string(),
            "summary": entry.summary,
            "tags": entry.tags,
            "sources": sources,
            "related": entry.related,
            "origin": entry.origin.to_string(),
            "harvested_from": entry.harvested_from,
            "last_verified_at": entry.last_verified_at,
            "staleness_flags": entry.staleness_flags,
            "created_at": entry.created_at,
            "updated_at": entry.updated_at,
            "content": content,
            "artifacts": artifact_list,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// PUT /api/knowledge/:slug
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct UpdateKnowledgeBody {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub tags_add: Option<Vec<String>>,
    #[serde(default)]
    pub related_add: Option<Vec<String>>,
}

pub async fn update_knowledge(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<UpdateKnowledgeBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let status = body
            .status
            .as_deref()
            .map(|s| {
                s.parse::<sdlc_core::knowledge::KnowledgeStatus>()
                    .map_err(|_| sdlc_core::SdlcError::InvalidKnowledgeStatus(s.to_string()))
            })
            .transpose()?;

        let tags_add = body.tags_add.unwrap_or_default();
        let related_add = body.related_add.unwrap_or_default();

        let entry = sdlc_core::knowledge::update(
            &root,
            &slug,
            body.title.as_deref(),
            body.code.as_deref(),
            status,
            body.summary.as_deref(),
            &tags_add,
            &related_add,
        )?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "code": entry.code,
            "status": entry.status.to_string(),
            "updated_at": entry.updated_at,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/knowledge/:slug/capture
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CaptureKnowledgeBody {
    pub filename: String,
    pub content: String,
}

pub async fn capture_knowledge_artifact(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<CaptureKnowledgeBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::knowledge::capture_named_artifact(&root, &slug, &body.filename, &body.content)?;
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

// ---------------------------------------------------------------------------
// GET /api/knowledge/:slug/sessions
// ---------------------------------------------------------------------------

pub async fn list_knowledge_sessions(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let sessions = sdlc_core::knowledge::list_sessions(&root, &slug)?;
        let list: Vec<serde_json::Value> = sessions
            .iter()
            .map(|s| {
                serde_json::json!({
                    "session": s.session,
                    "timestamp": s.timestamp,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/knowledge/:slug/sessions/:n
// ---------------------------------------------------------------------------

pub async fn get_knowledge_session(
    State(app): State<AppState>,
    Path(SessionPath { slug, n }): Path<SessionPath>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let content = sdlc_core::knowledge::read_session(&root, &slug, n)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "session": n,
            "content": content,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn slugify_title_server(title: &str) -> String {
    let lower = title.to_lowercase();
    let mut result = String::new();
    let mut last_was_dash = false;
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c);
            last_was_dash = false;
        } else if !last_was_dash && !result.is_empty() {
            result.push('-');
            last_was_dash = true;
        }
    }
    while result.ends_with('-') {
        result.pop();
    }
    result.chars().take(40).collect()
}
