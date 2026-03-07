//! Orchestrator webhook route management REST handlers.
//!
//! Routes:
//! - `POST /api/orchestrator/webhooks/routes` — register a new webhook route
//! - `GET  /api/orchestrator/webhooks/routes` — list all registered webhook routes

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
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
    /// When true, payloads are stored but never dispatched to the tool.
    pub store_only: Option<bool>,
    /// Optional shared secret. If set, incoming requests must supply a matching
    /// `X-Webhook-Secret` header or the payload is rejected with 401.
    pub secret_token: Option<String>,
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

    let backend = app.orchestrator_backend()?;
    let path = body.path.clone();
    let tool_name = body.tool_name.clone();
    let input_template = body.input_template.clone();
    let store_only = body.store_only.unwrap_or(false);
    let secret_token = body.secret_token.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut route =
            sdlc_core::orchestrator::WebhookRoute::new(&path, &tool_name, &input_template);
        route.store_only = store_only;
        route.secret_token = secret_token;

        let created_at = route.created_at;
        let id = route.id;
        let route_path = route.path.clone();
        let route_tool = route.tool_name.clone();
        let route_template = route.input_template.clone();
        let route_store_only = route.store_only;
        let route_secret_set = route.secret_token.is_some();

        backend.insert_route(&route).map_err(|e| {
            let is_conflict = e.to_string().contains("duplicate webhook route path");
            (is_conflict, anyhow::anyhow!("{e}"))
        })?;

        Ok::<_, (bool, anyhow::Error)>(serde_json::json!({
            "id": id.to_string(),
            "path": route_path,
            "tool_name": route_tool,
            "input_template": route_template,
            "store_only": route_store_only,
            "secret_token_set": route_secret_set,
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
    let backend = app.orchestrator_backend()?;
    let result = tokio::task::spawn_blocking(move || {
        let routes = backend.list_routes().map_err(|e| anyhow::anyhow!("{e}"))?;
        let json: Vec<serde_json::Value> = routes
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id.to_string(),
                    "path": r.path,
                    "tool_name": r.tool_name,
                    "input_template": r.input_template,
                    "store_only": r.store_only,
                    "secret_token_set": r.secret_token.is_some(),
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

// ---------------------------------------------------------------------------
// Action CRUD helpers
// ---------------------------------------------------------------------------

/// Serialize an `Action` to the API response shape.
fn action_to_json(action: &sdlc_core::orchestrator::Action) -> serde_json::Value {
    use sdlc_core::orchestrator::action::{ActionStatus, ActionTrigger};

    let trigger = match &action.trigger {
        ActionTrigger::Scheduled { next_tick_at } => serde_json::json!({
            "type": "scheduled",
            "next_tick_at": next_tick_at.to_rfc3339(),
        }),
        ActionTrigger::Webhook { .. } => serde_json::json!({ "type": "webhook" }),
    };

    let status = match &action.status {
        ActionStatus::Pending => serde_json::json!({ "type": "pending" }),
        ActionStatus::Running => serde_json::json!({ "type": "running" }),
        ActionStatus::Completed { result } => {
            serde_json::json!({ "type": "completed", "result": result })
        }
        ActionStatus::Failed { reason } => {
            serde_json::json!({ "type": "failed", "reason": reason })
        }
    };

    let recurrence_secs: serde_json::Value = match action.recurrence {
        Some(dur) => serde_json::json!(dur.as_secs()),
        None => serde_json::Value::Null,
    };

    serde_json::json!({
        "id": action.id.to_string(),
        "label": action.label,
        "tool_name": action.tool_name,
        "tool_input": action.tool_input,
        "trigger": trigger,
        "status": status,
        "recurrence_secs": recurrence_secs,
        "created_at": action.created_at.to_rfc3339(),
        "updated_at": action.updated_at.to_rfc3339(),
    })
}

// ---------------------------------------------------------------------------
// GET /api/orchestrator/actions — list all actions
// ---------------------------------------------------------------------------

/// `GET /api/orchestrator/actions`
///
/// Returns all actions sorted by `created_at` descending (newest first).
pub async fn list_actions(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let backend = app.orchestrator_backend()?;
    let result = tokio::task::spawn_blocking(move || {
        let actions = backend.list_all().map_err(|e| anyhow::anyhow!("{e}"))?;
        let json: Vec<serde_json::Value> = actions.iter().map(action_to_json).collect();
        Ok::<_, anyhow::Error>(serde_json::json!(json))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
    .map_err(AppError)?;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/orchestrator/actions — create a scheduled action
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateActionBody {
    pub label: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub next_tick_at: chrono::DateTime<chrono::Utc>,
    pub recurrence_secs: Option<u64>,
}

/// `POST /api/orchestrator/actions`
///
/// Create a new scheduled action. Returns `201 Created` with the action object.
///
/// Errors:
/// - `400` if `label` is empty
/// - `400` if `tool_name` is empty or fails slug validation
/// - `400` if `tool_input` is not a JSON object
pub async fn create_action(
    State(app): State<AppState>,
    Json(body): Json<CreateActionBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Validate inputs
    if body.label.is_empty() {
        return Err(AppError::bad_request("label must be non-empty"));
    }
    if body.tool_name.is_empty() {
        return Err(AppError::bad_request("tool_name must be non-empty"));
    }
    if let Err(e) = sdlc_core::paths::validate_slug(&body.tool_name) {
        return Err(AppError::bad_request(format!("tool_name: {e}")));
    }
    if !body.tool_input.is_object() {
        return Err(AppError::bad_request("tool_input must be a JSON object"));
    }

    let backend = app.orchestrator_backend()?;
    let label = body.label.clone();
    let tool_name = body.tool_name.clone();
    let tool_input = body.tool_input.clone();
    let next_tick_at = body.next_tick_at;
    let recurrence = body.recurrence_secs.map(std::time::Duration::from_secs);

    let result = tokio::task::spawn_blocking(move || {
        let action = sdlc_core::orchestrator::Action::new_scheduled(
            label,
            tool_name,
            tool_input,
            next_tick_at,
            recurrence,
        );
        backend
            .insert(&action)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok::<_, anyhow::Error>(action_to_json(&action))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
    .map_err(AppError)?;

    Ok((StatusCode::CREATED, Json(result)))
}

// ---------------------------------------------------------------------------
// DELETE /api/orchestrator/actions/{id}
// ---------------------------------------------------------------------------

/// `DELETE /api/orchestrator/actions/{id}`
///
/// Delete an action by UUID. Returns `204 No Content` (idempotent — succeeds
/// even if the action does not exist).
///
/// Returns `400` if `id` is not a valid UUID.
pub async fn delete_action(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = id
        .parse::<uuid::Uuid>()
        .map_err(|_| AppError::bad_request(format!("'{id}' is not a valid UUID")))?;

    let backend = app.orchestrator_backend()?;
    tokio::task::spawn_blocking(move || backend.delete(uuid).map_err(|e| anyhow::anyhow!("{e}")))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
        .map_err(AppError)?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// PATCH /api/orchestrator/actions/{id}
// ---------------------------------------------------------------------------

/// Newtype wrapper that distinguishes "field absent" from "field = null".
///
/// - Absent field (key not in JSON object)  → `MaybeAbsent::Absent`
/// - `null` value                           → `MaybeAbsent::Present(None)`
/// - Non-null value                         → `MaybeAbsent::Present(Some(v))`
#[derive(Debug, Default)]
pub(crate) enum MaybeAbsent<T> {
    #[default]
    Absent,
    Present(Option<T>),
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for MaybeAbsent<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // When the key is present (even if null), this is called.
        // serde calls the inner Option::deserialize for us.
        Option::<T>::deserialize(d).map(MaybeAbsent::Present)
    }
}

#[derive(Deserialize)]
pub struct PatchActionBody {
    pub label: Option<String>,
    /// Use `MaybeAbsent` to distinguish absent (no change) from null (clear).
    #[serde(default)]
    pub(crate) recurrence_secs: MaybeAbsent<u64>,
}

/// `PATCH /api/orchestrator/actions/{id}`
///
/// Update `label` and/or `recurrence_secs` on an existing action.
///
/// - `label`: optional non-empty string
/// - `recurrence_secs`: `null` clears it; a positive integer sets it; absent → no change
///
/// Returns `200 OK` with the updated action, `404` if not found, `400` for
/// validation errors.
pub async fn patch_action(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<PatchActionBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uuid = id
        .parse::<uuid::Uuid>()
        .map_err(|_| AppError::bad_request(format!("'{id}' is not a valid UUID")))?;

    // Validate label if provided
    if let Some(ref lbl) = body.label {
        if lbl.is_empty() {
            return Err(AppError::bad_request("label must be non-empty"));
        }
    }

    // Parse recurrence_secs field:
    //   MaybeAbsent::Absent         → None           (no change)
    //   MaybeAbsent::Present(None)  → Some(None)      (clear recurrence)
    //   MaybeAbsent::Present(Some(n))→ Some(Some(dur)) (set recurrence)
    let recurrence: Option<Option<std::time::Duration>> = match body.recurrence_secs {
        MaybeAbsent::Absent => None,
        MaybeAbsent::Present(None) => Some(None),
        MaybeAbsent::Present(Some(secs)) => Some(Some(std::time::Duration::from_secs(secs))),
    };

    let backend = app.orchestrator_backend()?;
    let label = body.label.clone();

    let result = tokio::task::spawn_blocking(move || {
        backend
            .update_label_and_recurrence(uuid, label, recurrence)
            .map_err(|e| {
                let not_found = e.to_string().contains("not found");
                (not_found, anyhow::anyhow!("{e}"))
            })
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?;

    match result {
        Ok(action) => Ok(Json(action_to_json(&action))),
        Err((true, e)) => Err(AppError::not_found(e.to_string())),
        Err((false, e)) => Err(AppError(e)),
    }
}

// ---------------------------------------------------------------------------
// DELETE /api/orchestrator/webhooks/routes/{id}
// ---------------------------------------------------------------------------

/// `DELETE /api/orchestrator/webhooks/routes/{id}`
///
/// Remove a registered webhook route by UUID. Returns `204 No Content`
/// (idempotent — succeeds even if the route does not exist).
///
/// Returns `400` if `id` is not a valid UUID.
pub async fn delete_route(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let uuid = id
        .parse::<uuid::Uuid>()
        .map_err(|_| AppError::bad_request(format!("'{id}' is not a valid UUID")))?;

    let backend = app.orchestrator_backend()?;
    tokio::task::spawn_blocking(move || {
        backend
            .delete_route(uuid)
            .map_err(|e| anyhow::anyhow!("{e}"))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
    .map_err(AppError)?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// GET /api/orchestrator/webhooks/events — list webhook event audit log
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct WebhookEventsQuery {
    /// Maximum number of events to return (default: 20).
    #[serde(default = "default_events_limit")]
    pub limit: usize,
}

fn default_events_limit() -> usize {
    20
}

/// `GET /api/orchestrator/webhooks/events?limit=N`
///
/// Returns stored webhook events from the ring buffer, sorted by `received_at`
/// descending (most recent first). The optional `?limit` query parameter caps
/// the number of returned events (default 20).
pub async fn list_webhook_events(
    State(app): State<AppState>,
    Query(params): Query<WebhookEventsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = params.limit;
    let backend = app.orchestrator_backend()?;
    let result = tokio::task::spawn_blocking(move || {
        let events = backend
            .list_webhook_events()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let json: Vec<serde_json::Value> = events
            .into_iter()
            .take(limit)
            .map(|e| {
                serde_json::json!({
                    "id": e.id.to_string(),
                    "seq": e.seq,
                    "route_path": e.route_path,
                    "content_type": e.content_type,
                    "body_bytes": e.body_bytes,
                    "received_at": e.received_at.to_rfc3339(),
                    "outcome": e.outcome,
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
