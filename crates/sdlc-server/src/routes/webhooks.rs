//! Webhook ingestion route for the orchestrator.
//!
//! `POST /webhooks/{route}` accepts any HTTP body, stores the raw bytes in the
//! orchestrator's redb `WEBHOOKS` table, and returns `202 Accepted` with the
//! assigned UUID. No payload transformation on ingress -- store exactly what
//! arrived.
//!
//! After a successful `insert_webhook`, a `WebhookEvent` with
//! `outcome: Received` is written to the `WEBHOOK_EVENTS` ring buffer. Event
//! logging is best-effort -- a failure to write the event does not affect the
//! 202 response returned to the sender.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use sdlc_core::orchestrator::{WebhookEvent, WebhookEventOutcome, WebhookPayload};

use crate::state::AppState;

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
    let payload = WebhookPayload::new(route_path.clone(), body.to_vec(), content_type.clone());
    let id = payload.id;

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
        backend.insert_webhook(&payload)?;

        // Best-effort: record a WebhookEvent for the arrival. Do not fail the
        // request if event logging fails -- the payload is already stored safely.
        let event = WebhookEvent::new(
            &route_path,
            content_type,
            body_bytes,
            WebhookEventOutcome::Received,
        );
        if let Err(e) = backend.insert_webhook_event(&event) {
            tracing::warn!(error = %e, "Failed to record webhook arrival event");
        }

        Ok::<_, sdlc_core::SdlcError>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({ "id": id.to_string() })),
        )
            .into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("task join error: {e}") })),
        )
            .into_response(),
    }
}
