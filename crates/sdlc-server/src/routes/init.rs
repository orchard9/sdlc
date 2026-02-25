use axum::extract::State;
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;
use crate::subprocess;

#[derive(serde::Deserialize)]
pub struct InitBody {
    pub platform: Option<String>,
}

/// POST /api/init — run sdlc init (returns run_id for streaming).
pub async fn init_project(
    State(app): State<AppState>,
    Json(body): Json<InitBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut argv = vec!["sdlc".to_string(), "init".to_string()];
    if let Some(ref platform) = body.platform {
        argv.push("--platform".to_string());
        argv.push(platform.clone());
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    let handle = subprocess::spawn_process(argv, &app.root);
    app.runs.write().await.insert(run_id.clone(), handle);

    Ok(Json(serde_json::json!({
        "run_id": run_id,
    })))
}
