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

/// GET /api/investigations — list all investigations with artifact counts.
pub async fn list_investigations(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entries = sdlc_core::investigation::list(&root)?;
        let list: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                let artifact_count = sdlc_core::investigation::list_artifacts(&root, &e.slug)
                    .map(|a| a.len())
                    .unwrap_or(0);
                serde_json::json!({
                    "slug": e.slug,
                    "title": e.title,
                    "kind": e.kind.to_string(),
                    "phase": e.phase,
                    "status": e.status.to_string(),
                    "sessions": e.sessions,
                    "artifact_count": artifact_count,
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

/// GET /api/investigations/:slug — full detail: manifest + artifacts.
pub async fn get_investigation(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entry = sdlc_core::investigation::load(&root, &slug)?;
        let artifacts = sdlc_core::investigation::list_artifacts(&root, &slug)?;

        let artifact_list: Vec<serde_json::Value> = artifacts
            .iter()
            .map(|a| {
                let content =
                    sdlc_core::investigation::read_artifact(&root, &slug, &a.filename).ok();
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
            "kind": entry.kind.to_string(),
            "phase": entry.phase,
            "status": entry.status.to_string(),
            "context": entry.context,
            "sessions": entry.sessions,
            "orientation": orientation,
            "created_at": entry.created_at,
            "updated_at": entry.updated_at,
            "confidence": entry.confidence,
            "output_type": entry.output_type,
            "output_ref": entry.output_ref,
            "scope": entry.scope,
            "lens_scores": entry.lens_scores,
            "output_refs": entry.output_refs,
            "guideline_scope": entry.guideline_scope,
            "problem_statement": entry.problem_statement,
            "evidence_counts": entry.evidence_counts,
            "principles_count": entry.principles_count,
            "publish_path": entry.publish_path,
            "artifacts": artifact_list,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct CreateInvestigationBody {
    pub slug: String,
    pub title: String,
    pub kind: String,
    #[serde(default)]
    pub context: Option<String>,
}

/// POST /api/investigations — create a new investigation.
pub async fn create_investigation(
    State(app): State<AppState>,
    Json(body): Json<CreateInvestigationBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let kind: sdlc_core::investigation::InvestigationKind = body
            .kind
            .parse()
            .map_err(|_| sdlc_core::SdlcError::InvalidInvestigationKind(body.kind.clone()))?;

        let entry =
            sdlc_core::investigation::create(&root, body.slug, body.title, kind, body.context)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "kind": entry.kind.to_string(),
            "phase": entry.phase,
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

/// POST /api/investigations/:slug/capture — capture content into workspace.
pub async fn capture_artifact(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<CaptureArtifactBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::investigation::capture_content(&root, &slug, &body.filename, &body.content)?;
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
pub struct UpdateInvestigationBody {
    #[serde(default)]
    pub phase: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub confidence: Option<u32>,
    #[serde(default)]
    pub output_type: Option<String>,
    #[serde(default)]
    pub output_ref: Option<String>,
}

/// PUT /api/investigations/:slug — update phase/status/title/scope/confidence.
pub async fn update_investigation(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<UpdateInvestigationBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut entry = sdlc_core::investigation::load(&root, &slug)?;

        if let Some(phase) = body.phase {
            entry.phase = phase;
            entry.updated_at = chrono::Utc::now();
        }
        if let Some(status_str) = body.status {
            let status: sdlc_core::investigation::InvestigationStatus =
                status_str.parse().map_err(|_| {
                    sdlc_core::SdlcError::InvalidInvestigationStatus(status_str.clone())
                })?;
            entry.status = status;
            entry.updated_at = chrono::Utc::now();
        }
        if let Some(title) = body.title {
            entry.title = title;
            entry.updated_at = chrono::Utc::now();
        }
        if let Some(s) = body.scope {
            match entry.kind {
                sdlc_core::investigation::InvestigationKind::Evolve => {
                    entry.scope = Some(s);
                }
                sdlc_core::investigation::InvestigationKind::Guideline => {
                    entry.guideline_scope = Some(s);
                }
                _ => {
                    return Err(sdlc_core::SdlcError::InvalidInvestigationKind(
                        "scope is only valid for evolve and guideline".to_string(),
                    ));
                }
            }
            entry.updated_at = chrono::Utc::now();
        }
        if let Some(c) = body.confidence {
            if entry.kind != sdlc_core::investigation::InvestigationKind::RootCause {
                return Err(sdlc_core::SdlcError::InvalidInvestigationKind(
                    "confidence is only valid for root-cause".to_string(),
                ));
            }
            entry.confidence = Some(c);
            entry.updated_at = chrono::Utc::now();
        }
        if let Some(ot) = body.output_type {
            if entry.kind != sdlc_core::investigation::InvestigationKind::RootCause {
                return Err(sdlc_core::SdlcError::InvalidInvestigationKind(
                    "output_type is only valid for root-cause".to_string(),
                ));
            }
            entry.output_type = Some(ot);
            entry.updated_at = chrono::Utc::now();
        }
        if let Some(or_) = body.output_ref {
            if entry.kind != sdlc_core::investigation::InvestigationKind::RootCause {
                return Err(sdlc_core::SdlcError::InvalidInvestigationKind(
                    "output_ref is only valid for root-cause".to_string(),
                ));
            }
            entry.output_ref = Some(or_);
            entry.updated_at = chrono::Utc::now();
        }

        sdlc_core::investigation::save(&root, &entry)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "kind": entry.kind.to_string(),
            "phase": entry.phase,
            "status": entry.status.to_string(),
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/investigations/:slug/sessions — list session metadata.
pub async fn list_investigation_sessions(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let sessions = sdlc_core::investigation::list_sessions(&root, &slug)?;
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

/// GET /api/investigations/:slug/sessions/:n — full content of a single session.
pub async fn get_investigation_session(
    State(app): State<AppState>,
    Path(SessionPath { slug, n }): Path<SessionPath>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let content = sdlc_core::investigation::read_session(&root, &slug, n)?;
        let meta = sdlc_core::workspace::parse_session_meta(&content);
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
