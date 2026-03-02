//! Orchestrator webhook route management REST handlers.
//!
//! Routes:
//! - `POST /api/orchestrator/webhooks/routes` — register a new webhook route
//! - `GET  /api/orchestrator/webhooks/routes` — list all registered webhook routes

use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// POST /api/orchestrator/webhooks/routes — register a route
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct RegisterRouteBody {
    /// URL path this route handles. Must start with `/` (e.g. `/hooks/github`).
    pub path: String,
    /// Tool slug matching a directory under `.sdlc/tools/<name>/`.
    pub tool_name: String,
    /// JSON template for the tool input. Use `{{payload}}` as the placeholder
    /// for the JSON-escaped webhook body.
    pub input_template: String,
}

/// `POST /api/orchestrator/webhooks/routes`
///
/// Register a new webhook route. Returns `201 Created` with the created route.
///
/// Errors:
/// - `400` if `path` is empty or does not start with `/`
/// - `400` if `tool_name` or `input_template` is empty
/// - `409` if a route with the same `path` already exists
pub async fn register_route(
    State(app): State<AppState>,
    Json(body): Json<RegisterRouteBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Validate inputs
    if body.path.is_empty() || !body.path.starts_with('/') {
        return Err(AppError::bad_request(
            "path must be non-empty and start with '/'",
        ));
    }
    if body.tool_name.is_empty() {
        return Err(AppError::bad_request("tool_name must be non-empty"));
    }
    // Validate tool_name as a slug to prevent path traversal (e.g. "../evil")
    if let Err(e) = sdlc_core::paths::validate_slug(&body.tool_name) {
        return Err(AppError::bad_request(format!("tool_name: {e}")));
    }
    if body.input_template.is_empty() {
        return Err(AppError::bad_request("input_template must be non-empty"));
    }

    let root = app.root.clone();
    let path = body.path.clone();
    let tool_name = body.tool_name.clone();
    let input_template = body.input_template.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db_path = sdlc_core::paths::orchestrator_db_path(&root);
        let db = sdlc_core::orchestrator::ActionDb::open(&db_path).map_err(|e| {
            (
                false,
                anyhow::anyhow!("failed to open orchestrator DB: {e}"),
            )
        })?;

        let route = sdlc_core::orchestrator::WebhookRoute::new(&path, &tool_name, &input_template);
        let created_at = route.created_at;
        let id = route.id;
        let route_path = route.path.clone();
        let route_tool = route.tool_name.clone();
        let route_template = route.input_template.clone();

        db.insert_route(&route).map_err(|e| {
            let is_conflict = e.to_string().contains("duplicate webhook route path");
            (is_conflict, anyhow::anyhow!("{e}"))
        })?;

        Ok::<_, (bool, anyhow::Error)>(serde_json::json!({
            "id": id.to_string(),
            "path": route_path,
            "tool_name": route_tool,
            "input_template": route_template,
            "created_at": created_at.to_rfc3339(),
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?;

    match result {
        Ok(json) => Ok((StatusCode::CREATED, Json(json))),
        Err((true, e)) => Err(AppError::conflict(e.to_string())),
        Err((false, e)) => Err(AppError(e)),
    }
}

// ---------------------------------------------------------------------------
// GET /api/orchestrator/webhooks/routes — list all routes
// ---------------------------------------------------------------------------

/// `GET /api/orchestrator/webhooks/routes`
///
/// Returns all registered webhook routes as a JSON array, sorted by
/// `created_at` ascending.
pub async fn list_routes(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_path = sdlc_core::paths::orchestrator_db_path(&root);
        let db = sdlc_core::orchestrator::ActionDb::open(&db_path)
            .map_err(|e| anyhow::anyhow!("failed to open orchestrator DB: {e}"))?;
        let routes = db.list_routes().map_err(|e| anyhow::anyhow!("{e}"))?;
        let json: Vec<serde_json::Value> = routes
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id.to_string(),
                    "path": r.path,
                    "tool_name": r.tool_name,
                    "input_template": r.input_template,
                    "created_at": r.created_at.to_rfc3339(),
                })
            })
            .collect();
        Ok::<_, anyhow::Error>(serde_json::json!(json))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
    .map_err(AppError)?;

    Ok(Json(result))
}
