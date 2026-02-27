use axum::extract::State;
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/config — read-only view of the project's `.sdlc/config.yaml`.
///
/// Returns the parsed Config struct as JSON. No PUT endpoint — config is a
/// git-committed YAML file; changes go through the normal edit-commit workflow.
pub async fn get_config(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::config::Config::load(&root)?;
        let json = serde_json::to_value(&config)?;
        Ok::<_, sdlc_core::SdlcError>(json)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;

    #[tokio::test]
    async fn get_config_returns_error_when_not_initialized() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = get_config(State(app)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_config_returns_project_config() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = sdlc_core::config::Config::new("test-project");
        sdlc_core::io::ensure_dir(&dir.path().join(".sdlc")).unwrap();
        config.save(dir.path()).unwrap();

        let app = AppState::new(dir.path().to_path_buf());
        let result = get_config(State(app)).await.unwrap();
        let json = result.0;
        assert_eq!(json["project"]["name"], "test-project");
        assert_eq!(json["version"], 1);
    }
}
