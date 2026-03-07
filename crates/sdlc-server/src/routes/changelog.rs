use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ChangelogQuery {
    /// ISO 8601 UTC timestamp — return only events at or after this time.
    pub since: Option<String>,
    /// Maximum number of events to return. Defaults to 100.
    pub limit: Option<usize>,
}

/// GET /api/changelog
///
/// Returns changelog events filtered by `since` (optional ISO 8601 UTC timestamp)
/// and capped at `limit` (optional, default 100).
///
/// Response: `{ "events": [...], "total": N }`
///
/// When `.sdlc/changelog.yaml` does not exist yet, returns `{ "events": [], "total": 0 }`.
pub async fn get_changelog(
    Query(params): Query<ChangelogQuery>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Parse `since` on the async thread before entering spawn_blocking so we
    // can return 400 quickly without occupying a blocking thread.
    let since: Option<DateTime<Utc>> = match params.since {
        None => None,
        Some(ref s) => Some(
            s.parse::<DateTime<Utc>>()
                .map_err(|_| AppError::bad_request("invalid since timestamp"))?,
        ),
    };
    let limit = params.limit.unwrap_or(100);
    let root = app.root.clone();

    let events = tokio::task::spawn_blocking(move || {
        sdlc_core::event_log::query_events(&root, since, limit)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    let total = events.len();
    Ok(Json(
        serde_json::json!({ "events": events, "total": total }),
    ))
}
