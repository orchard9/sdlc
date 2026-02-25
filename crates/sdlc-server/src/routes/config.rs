use axum::extract::State;
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/config/agents — agent backend routing config.
pub async fn get_agents_config(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::config::Config::load(&root)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "default": config.agents.default,
            "actions": config.agents.actions,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// PUT /api/config/agents — update agent backend routing config.
pub async fn put_agents_config(
    State(app): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        // Load the existing config so we preserve project/phases/platform settings
        let mut config = sdlc_core::config::Config::load(&root)?;

        // Deserialize the incoming JSON into the AgentsConfig struct
        let agents: sdlc_core::config::AgentsConfig =
            serde_json::from_value(body).map_err(sdlc_core::SdlcError::Json)?;
        config.agents = agents;

        // Write the full config back atomically
        config.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({ "ok": true }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
