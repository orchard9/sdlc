use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use crate::hub::{HeartbeatPayload, HubSseMessage};
use crate::state::AppState;

/// POST /api/hub/heartbeat
///
/// Accepts a heartbeat payload from a project instance. First call from a given
/// URL registers the project; subsequent calls update `last_seen`. Returns 503
/// if the server is not running in hub mode.
pub async fn heartbeat(
    State(app): State<AppState>,
    Json(payload): Json<HeartbeatPayload>,
) -> axum::response::Response {
    let Some(hub) = &app.hub_registry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "not running in hub mode"})),
        )
            .into_response();
    };
    let mut registry = hub.lock().await;
    let _entry = registry.apply_heartbeat(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({"registered": true})),
    )
        .into_response()
}

/// GET /api/hub/projects
///
/// Returns the current project registry sorted by last_seen descending.
/// Returns 503 if not in hub mode.
pub async fn list_projects(State(app): State<AppState>) -> impl axum::response::IntoResponse {
    let Some(hub) = &app.hub_registry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({"error": "not running in hub mode"})),
        )
            .into_response();
    };
    let registry = hub.lock().await;
    let projects = registry.projects_sorted();
    axum::Json(serde_json::json!(projects)).into_response()
}

/// GET /api/hub/events
///
/// SSE stream for hub UI clients. Emits `ProjectUpdated` and `ProjectRemoved` events.
/// Returns 503 if not in hub mode.
pub async fn hub_sse_events(State(app): State<AppState>) -> impl axum::response::IntoResponse {
    let Some(hub) = &app.hub_registry else {
        return (StatusCode::SERVICE_UNAVAILABLE, "not running in hub mode").into_response();
    };
    let rx = hub.lock().await.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(HubSseMessage::ProjectUpdated(entry)) => {
            let data = serde_json::json!({
                "type": "project_updated",
                "project": entry,
            })
            .to_string();
            Some(Ok::<Event, Infallible>(
                Event::default().event("hub").data(data),
            ))
        }
        Ok(HubSseMessage::ProjectRemoved { url }) => {
            let data = serde_json::json!({
                "type": "project_removed",
                "url": url,
            })
            .to_string();
            Some(Ok(Event::default().event("hub").data(data)))
        }
        Err(_) => None,
    });

    // 2KB padding comment so Cloudflare/nginx don't buffer the initial flush.
    let padding = Ok::<Event, Infallible>(Event::default().comment(" ".repeat(2048)));
    let padded = tokio_stream::iter(std::iter::once(padding)).chain(stream);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store"),
    );
    headers.insert(
        header::HeaderName::from_static("x-accel-buffering"),
        HeaderValue::from_static("no"),
    );
    (headers, Sse::new(padded).keep_alive(KeepAlive::default())).into_response()
}
