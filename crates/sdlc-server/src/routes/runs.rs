use axum::{
    extract::{Path, State},
    Json,
};
use sdlc_core::{
    classifier::{Classifier, EvalContext},
    config::Config,
    directive::build_directive,
    feature::Feature,
    paths,
    rules::default_rules,
    state::State as SdlcState,
    types::ActionType,
};

use crate::{error::AppError, state::AppState};

/// POST /api/run/:slug — generate and write a directive for a feature.
pub async fn run_feature(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    tokio::task::spawn_blocking(move || {
        let config = Config::load(&root)?;
        let state = SdlcState::load(&root)?;
        let feature = Feature::load(&root, &slug)?;

        let ctx = EvalContext {
            feature: &feature,
            state: &state,
            config: &config,
            root: &root,
        };
        let classification = Classifier::new(default_rules()).classify(&ctx);

        if classification.action == ActionType::Done {
            return Ok(Json(serde_json::json!({
                "status": "done",
                "message": format!("Feature '{}' is complete — no pending actions.", slug)
            })));
        }

        let directive = build_directive(&classification, &slug, &root);
        let directive_path = paths::directive_md_path(&root, &slug);

        if let Some(p) = directive_path.parent() {
            std::fs::create_dir_all(p).map_err(sdlc_core::SdlcError::Io)?;
        }
        std::fs::write(&directive_path, &directive).map_err(sdlc_core::SdlcError::Io)?;

        Ok(Json(serde_json::json!({
            "status": "directive_written",
            "action": classification.action.as_str(),
            "directive_path": directive_path.to_string_lossy(),
            "message": classification.message
        })))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
}
