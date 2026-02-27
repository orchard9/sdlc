use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use crate::state::AppState;

/// GET /api/events — SSE stream that emits `update` whenever project state changes.
pub async fn sse_events(State(app): State<AppState>) -> impl axum::response::IntoResponse {
    let rx = app.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        msg.ok()
            .map(|_| Ok::<Event, Infallible>(Event::default().event("update").data("update")))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
