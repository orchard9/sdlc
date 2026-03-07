//! Webhook ingestion and inspection routes for the orchestrator.
//!
//! `POST /webhooks/{route}` accepts any HTTP body, stores the raw bytes in the
//! orchestrator's redb `WEBHOOKS` table, and returns `202 Accepted` with the
//! assigned UUID. No payload transformation on ingress -- store exactly what
//! arrived.
//!
//! If the registered route has a `secret_token` set, the request must supply a
//! matching `X-Webhook-Secret` header or a `401 Unauthorized` is returned.
//!
//! After a successful `insert_webhook`, a `WebhookEvent` with
//! `outcome: Received` is written to the `WEBHOOK_EVENTS` ring buffer. Event
//! logging is best-effort -- a failure to write the event does not affect the
//! 202 response returned to the sender.
//!
//! `GET /api/webhooks/{route}/data` returns stored payloads for a route with
//! optional `since`, `until`, and `limit` query parameters.
//!
//! `POST /api/webhooks/{route}/replay/{id}` re-dispatches a stored payload
//! through the registered tool.

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::Engine as _;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use sdlc_core::orchestrator::{WebhookEvent, WebhookEventOutcome, WebhookPayload};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// POST /webhooks/{route}
// ---------------------------------------------------------------------------

/// `POST /webhooks/{route}`
///
/// Accepts a webhook from any external sender. Stores the raw body bytes,
/// the request `Content-Type` (if present), and the route path in the redb
/// `WEBHOOKS` table. Returns `202 Accepted` with `{ "id": "<uuid>" }`.
///
/// The `route_path` stored in the DB is normalized to always start with `/`
/// (e.g. URL `/webhooks/github` -> stored as `/github`). This matches the
/// format expected by `WebhookRoute.path` in the route registry.
///
/// If the registered route has a `secret_token`, the `X-Webhook-Secret` header
/// must match. Returns `401 Unauthorized` if missing or wrong.
///
/// Also writes a `WebhookEvent` with `outcome: Received` to the ring-buffered
/// `WEBHOOK_EVENTS` table for audit/history purposes.
pub async fn receive_webhook(
    State(app): State<AppState>,
    Path(route): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Normalize: always prefix with '/' to match WebhookRoute.path format.
    let route_path = if route.starts_with('/') {
        route
    } else {
        format!("/{route}")
    };

    let body_bytes = body.len();
    let body_vec = body.to_vec();

    let backend = match app.orchestrator_backend() {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "orchestrator not ready" })),
            )
                .into_response();
        }
    };

    // Look up route to check secret_token. If no route is registered we still
    // store the payload (dispatch will drop it later with NoRoute).
    let supplied_secret = headers
        .get("x-webhook-secret")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let rp_clone = route_path.clone();
    let ct_clone = content_type.clone();
    let result = tokio::task::spawn_blocking(move || {
        // Secret token check
        if let Some(registered_route) = backend
            .find_route_by_path(&rp_clone)
            .map_err(|e| {
                tracing::warn!(error = %e, "route lookup failed during secret check");
                e
            })
            .unwrap_or(None)
        {
            if let Some(expected) = &registered_route.secret_token {
                match &supplied_secret {
                    None => {
                        return Err(("missing X-Webhook-Secret header".to_string(), true));
                    }
                    Some(provided) if provided != expected => {
                        return Err(("invalid X-Webhook-Secret".to_string(), true));
                    }
                    _ => {}
                }
            }
        }

        let payload = WebhookPayload::new(rp_clone.clone(), body_vec, ct_clone.clone());
        let id = payload.id;

        backend
            .insert_webhook(&payload)
            .map_err(|e| (e.to_string(), false))?;

        // Best-effort: record a WebhookEvent for the arrival. Do not fail the
        // request if event logging fails -- the payload is already stored safely.
        let event = WebhookEvent::new(
            &rp_clone,
            ct_clone,
            body_bytes,
            WebhookEventOutcome::Received,
        );
        if let Err(e) = backend.insert_webhook_event(&event) {
            tracing::warn!(error = %e, "Failed to record webhook arrival event");
        }

        Ok::<_, (String, bool)>(id)
    })
    .await;

    match result {
        Ok(Ok(id)) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({ "id": id.to_string() })),
        )
            .into_response(),
        Ok(Err((msg, true))) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
        Ok(Err((msg, false))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("task join error: {e}") })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// GET /api/webhooks/{route}/data
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct WebhookQueryParams {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

fn default_query_limit() -> usize {
    100
}

/// `GET /api/webhooks/{route}/data`
///
/// Returns stored webhook payloads for the given route, newest first.
///
/// Query parameters:
/// - `since`: ISO-8601 lower bound on `received_at` (inclusive)
/// - `until`: ISO-8601 upper bound on `received_at` (inclusive)
/// - `limit`: max payloads to return (default 100)
///
/// Each payload in the response has `body` as a base64-encoded string.
pub async fn query_webhook_payloads(
    State(app): State<AppState>,
    Path(route): Path<String>,
    Query(params): Query<WebhookQueryParams>,
) -> impl IntoResponse {
    // Normalize route path
    let route_path = if route.starts_with('/') {
        route
    } else {
        format!("/{route}")
    };

    let limit = params.limit.unwrap_or_else(default_query_limit);

    let backend = match app.orchestrator_backend() {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "orchestrator not ready" })),
            )
                .into_response();
        }
    };

    let result = tokio::task::spawn_blocking(move || {
        backend
            .query_webhooks(&route_path, params.since, params.until, limit)
            .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(Ok(payloads)) => {
            let json: Vec<serde_json::Value> = payloads
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "id": p.id.to_string(),
                        "route_path": p.route_path,
                        "received_at": p.received_at.to_rfc3339(),
                        "content_type": p.content_type,
                        "body": base64::engine::general_purpose::STANDARD.encode(&p.raw_body),
                    })
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!(json))).into_response()
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("task join error: {e}") })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// POST /api/webhooks/{route}/replay/{id}
// ---------------------------------------------------------------------------

/// `POST /api/webhooks/{route}/replay/{id}`
///
/// Re-dispatch a stored payload for `route` with the given `id` through the
/// registered tool. The payload must still exist in storage (not yet deleted)
/// and must belong to `route`. The tool is run synchronously.
///
/// Returns `{ "ok": true }` on success, or an error JSON body.
pub async fn replay_webhook_payload(
    State(app): State<AppState>,
    Path((route, id)): Path<(String, Uuid)>,
) -> impl IntoResponse {
    // Normalize route path
    let route_path = if route.starts_with('/') {
        route
    } else {
        format!("/{route}")
    };

    let backend = match app.orchestrator_backend() {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "orchestrator not ready" })),
            )
                .into_response();
        }
    };

    let root = app.root.clone();

    let result = tokio::task::spawn_blocking(move || {
        // Find the payload via query (limit 1, exact id match via full scan).
        let payloads = backend
            .all_pending_webhooks()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let payload = payloads.into_iter().find(|p| p.id == id).ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("no stored payload with id {id}"),
            )
        })?;

        // Verify it belongs to the requested route
        if payload.route_path != route_path {
            return Err((
                StatusCode::NOT_FOUND,
                format!(
                    "payload {id} belongs to route '{}', not '{route_path}'",
                    payload.route_path
                ),
            ));
        }

        // Find the registered route
        let route = backend
            .find_route_by_path(&route_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    format!("no route registered for '{route_path}'"),
                )
            })?;

        // Render template
        let tool_input = route.render_input(&payload.raw_body).map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("template render error: {e}"),
            )
        })?;

        let script = sdlc_core::paths::tool_script(&root, &route.tool_name);
        if !script.exists() {
            return Err((
                StatusCode::NOT_FOUND,
                format!("tool script not found: {}", script.display()),
            ));
        }

        let input_json = serde_json::to_string(&tool_input)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sdlc_core::tool_runner::run_tool(&script, "--run", Some(&input_json), &root, None)
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))?;

        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Ok(Err((status, msg))) => {
            (status, Json(serde_json::json!({ "error": msg }))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("task join error: {e}") })),
        )
            .into_response(),
    }
}
