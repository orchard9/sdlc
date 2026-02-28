use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

/// GET /api/escalations — list escalations (default: open only; ?status=all for all)
pub async fn list_escalations(
    State(app): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let status = q.status;
    let result = tokio::task::spawn_blocking(move || {
        let items = sdlc_core::escalation::list(&root, status.as_deref())?;
        let list: Vec<serde_json::Value> = items.iter().map(escalation_to_json).collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Show
// ---------------------------------------------------------------------------

/// GET /api/escalations/:id — single escalation detail
pub async fn get_escalation(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let item = sdlc_core::escalation::get(&root, &id)?;
        Ok::<_, sdlc_core::SdlcError>(escalation_to_json(&item))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CreateBody {
    pub kind: String,
    pub title: String,
    pub context: String,
    pub source_feature: Option<String>,
}

/// POST /api/escalations — create a new escalation
pub async fn create_escalation(
    State(app): State<AppState>,
    Json(body): Json<CreateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let kind: sdlc_core::escalation::EscalationKind = body.kind.parse()?;
        let item = sdlc_core::escalation::create(
            &root,
            kind,
            &body.title,
            &body.context,
            body.source_feature.as_deref(),
        )?;
        Ok::<_, sdlc_core::SdlcError>(escalation_to_json(&item))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Resolve
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct ResolveBody {
    pub resolution: String,
}

/// POST /api/escalations/:id/resolve — resolve an escalation
pub async fn resolve_escalation(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ResolveBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let item = sdlc_core::escalation::resolve(&root, &id, &body.resolution)?;
        Ok::<_, sdlc_core::SdlcError>(escalation_to_json(&item))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn escalation_to_json(e: &sdlc_core::escalation::EscalationItem) -> serde_json::Value {
    serde_json::json!({
        "id": e.id,
        "kind": e.kind.to_string(),
        "title": e.title,
        "context": e.context,
        "source_feature": e.source_feature,
        "linked_comment_id": e.linked_comment_id,
        "status": e.status.to_string(),
        "created_at": e.created_at,
        "resolved_at": e.resolved_at,
        "resolution": e.resolution,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn list_empty_when_no_escalations() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = list_escalations(State(app), Query(ListQuery { status: None }))
            .await
            .unwrap();
        let arr = result.0.as_array().unwrap();
        assert!(arr.is_empty());
    }

    #[tokio::test]
    async fn create_and_list() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());

        let body = CreateBody {
            kind: "question".to_string(),
            title: "Should we do X?".to_string(),
            context: "Affects milestone v1".to_string(),
            source_feature: None,
        };
        let _ = create_escalation(State(app.clone()), Json(body))
            .await
            .unwrap();

        let result = list_escalations(State(app), Query(ListQuery { status: None }))
            .await
            .unwrap();
        let arr = result.0.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "E1");
        assert_eq!(arr[0]["kind"], "question");
        assert_eq!(arr[0]["status"], "open");
    }

    #[tokio::test]
    async fn get_missing_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let err = get_escalation(State(app), Path("E99".to_string()))
            .await
            .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn resolve_and_list_resolved() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());

        let body = CreateBody {
            kind: "vision".to_string(),
            title: "Define product vision".to_string(),
            context: "No vision document yet".to_string(),
            source_feature: None,
        };
        let _ = create_escalation(State(app.clone()), Json(body))
            .await
            .unwrap();

        let _ = resolve_escalation(
            State(app.clone()),
            Path("E1".to_string()),
            Json(ResolveBody {
                resolution: "Vision doc added".to_string(),
            }),
        )
        .await
        .unwrap();

        let open = list_escalations(State(app.clone()), Query(ListQuery { status: None }))
            .await
            .unwrap();
        assert!(open.0.as_array().unwrap().is_empty());

        let resolved = list_escalations(
            State(app),
            Query(ListQuery {
                status: Some("resolved".to_string()),
            }),
        )
        .await
        .unwrap();
        assert_eq!(resolved.0.as_array().unwrap().len(), 1);
    }
}
