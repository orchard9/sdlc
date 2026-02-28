use axum::extract::State;
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
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
