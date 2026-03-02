use axum::{
    extract::{Path, State},
    Json,
};

use crate::{error::AppError, state::AppState};

/// GET /api/runs/:id/telemetry
///
/// Returns all events captured for the given run in sequence order.
/// Response: `{ "run_id": "...", "events": [...] }`
pub async fn get_run_telemetry(
    Path(run_id): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let store = app
        .telemetry
        .as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("Telemetry store not available")))?
        .clone();

    let run_id_clone = run_id.clone();
    let events = tokio::task::spawn_blocking(move || store.events_for_run(&run_id_clone))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("spawn_blocking error: {e}")))?
        .map_err(|e| AppError(anyhow::anyhow!("Telemetry read error: {e}")))?;

    Ok(Json(serde_json::json!({
        "run_id": run_id,
        "events": events,
    })))
}

/// GET /api/runs/:id/telemetry/summary
///
/// Returns aggregated stats for the given run.
/// Response: `{ "run_id": "...", "tool_calls": N, "tool_errors": N, ... }`
pub async fn get_run_telemetry_summary(
    Path(run_id): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let store = app
        .telemetry
        .as_ref()
        .ok_or_else(|| AppError(anyhow::anyhow!("Telemetry store not available")))?
        .clone();

    let run_id_clone = run_id.clone();
    let summary = tokio::task::spawn_blocking(move || store.summary_for_run(&run_id_clone))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("spawn_blocking error: {e}")))?
        .map_err(|e| AppError(anyhow::anyhow!("Telemetry read error: {e}")))?;

    Ok(Json(serde_json::json!({
        "run_id": run_id,
        "tool_calls": summary.tool_calls,
        "tool_errors": summary.tool_errors,
        "tools_used": summary.tools_used,
        "subagents_spawned": summary.subagents_spawned,
        "subagent_tokens": summary.subagent_tokens,
        "total_cost_usd": summary.total_cost_usd,
        "total_turns": summary.total_turns,
    })))
}
