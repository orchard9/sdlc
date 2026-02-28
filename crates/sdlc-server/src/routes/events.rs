use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue};
use axum::response::sse::{Event, KeepAlive, Sse};
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use crate::state::{AppState, SseMessage};

/// GET /api/events — SSE stream that emits typed events whenever state changes.
///
/// Event types:
/// - `update`  data: "update"               — generic state change, re-fetch everything
/// - `ponder`  data: JSON `{ type, slug, session? }` — ponder run lifecycle
pub async fn sse_events(State(app): State<AppState>) -> impl axum::response::IntoResponse {
    let rx = app.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(SseMessage::Update) => Some(Ok::<Event, Infallible>(
            Event::default().event("update").data("update"),
        )),
        Ok(SseMessage::PonderRunStarted { slug, session }) => {
            let data = serde_json::json!({
                "type": "ponder_run_started",
                "slug": slug,
                "session": session,
            })
            .to_string();
            Some(Ok(Event::default().event("ponder").data(data)))
        }
        Ok(SseMessage::PonderRunCompleted { slug, session }) => {
            let data = serde_json::json!({
                "type": "ponder_run_completed",
                "slug": slug,
                "session": session,
            })
            .to_string();
            Some(Ok(Event::default().event("ponder").data(data)))
        }
        Ok(SseMessage::PonderRunStopped { slug }) => {
            let data = serde_json::json!({
                "type": "ponder_run_stopped",
                "slug": slug,
            })
            .to_string();
            Some(Ok(Event::default().event("ponder").data(data)))
        }
        Ok(SseMessage::InvestigationRunStarted { slug, session }) => {
            let data = serde_json::json!({
                "type": "investigation_run_started",
                "slug": slug,
                "session": session,
            })
            .to_string();
            Some(Ok(Event::default().event("investigation").data(data)))
        }
        Ok(SseMessage::InvestigationRunCompleted { slug, session }) => {
            let data = serde_json::json!({
                "type": "investigation_run_completed",
                "slug": slug,
                "session": session,
            })
            .to_string();
            Some(Ok(Event::default().event("investigation").data(data)))
        }
        Ok(SseMessage::InvestigationRunStopped { slug }) => {
            let data = serde_json::json!({
                "type": "investigation_run_stopped",
                "slug": slug,
            })
            .to_string();
            Some(Ok(Event::default().event("investigation").data(data)))
        }
        Ok(SseMessage::RunStarted { id, key, label }) => {
            let data = serde_json::json!({
                "type": "run_started",
                "id": id,
                "key": key,
                "label": label,
            })
            .to_string();
            Some(Ok(Event::default().event("run").data(data)))
        }
        Ok(SseMessage::RunFinished { id, key, status }) => {
            let data = serde_json::json!({
                "type": "run_finished",
                "id": id,
                "key": key,
                "status": status,
            })
            .to_string();
            Some(Ok(Event::default().event("run").data(data)))
        }
        Ok(SseMessage::VisionAlignCompleted) => {
            let data = serde_json::json!({ "type": "vision_align_completed" }).to_string();
            Some(Ok(Event::default().event("docs").data(data)))
        }
        Ok(SseMessage::ArchitectureAlignCompleted) => {
            let data = serde_json::json!({ "type": "architecture_align_completed" }).to_string();
            Some(Ok(Event::default().event("docs").data(data)))
        }
        Err(_) => None,
    });
    // Prepend a ~2KB padding comment so the response body exceeds Cloudflare's
    // initial buffer threshold on first flush. Without this, small SSE events
    // (100–200 bytes) sit in Cloudflare's buffer and are never forwarded.
    // x-accel-buffering disables nginx buffering; Cache-Control covers other
    // proxy layers.
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
    (headers, Sse::new(padded).keep_alive(KeepAlive::default()))
}
