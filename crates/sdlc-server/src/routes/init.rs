use axum::extract::State;
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct InitBody {
    pub platform: Option<String>,
}

/// POST /api/init — initialize the sdlc project (runs sdlc_core init logic directly).
pub async fn init_project(
    State(app): State<AppState>,
    Json(_body): Json<InitBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();

    tokio::task::spawn_blocking(move || {
        let project_name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "project".to_string());

        // Create .sdlc directory structure
        let dirs = [sdlc_core::paths::SDLC_DIR, sdlc_core::paths::FEATURES_DIR];
        for dir in dirs {
            let p = root.join(dir);
            sdlc_core::io::ensure_dir(&p)?;
        }

        // Write config.yaml if missing
        let config_path = sdlc_core::paths::config_path(&root);
        if !config_path.exists() {
            let cfg = sdlc_core::config::Config::new(&project_name);
            cfg.save(&root)?;
        }

        // Write state.yaml if missing
        let state_path = sdlc_core::paths::state_path(&root);
        if !state_path.exists() {
            let state = sdlc_core::state::State::new(&project_name);
            state.save(&root)?;
        }

        Ok::<_, sdlc_core::SdlcError>(())
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(serde_json::json!({ "ok": true })))
}
