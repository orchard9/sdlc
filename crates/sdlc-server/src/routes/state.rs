use axum::extract::State;
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/state — project state with milestones grouped with features.
pub async fn get_state(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let state = sdlc_core::state::State::load(&root)?;
        let features = sdlc_core::feature::Feature::list(&root)?;
        let milestones = sdlc_core::milestone::Milestone::list(&root)?;

        let config = sdlc_core::config::Config::load(&root)?;
        let classifier = sdlc_core::classifier::Classifier::new(sdlc_core::rules::default_rules());

        let feature_summaries: Vec<serde_json::Value> = features
            .iter()
            .map(|f| {
                let ctx = sdlc_core::classifier::EvalContext {
                    feature: f,
                    state: &state,
                    config: &config,
                    root: &root,
                };
                let classification = classifier.classify(&ctx);
                serde_json::json!({
                    "slug": f.slug,
                    "title": f.title,
                    "description": f.description,
                    "phase": f.phase,
                    "archived": f.archived,
                    "blocked": f.is_blocked(),
                    "next_action": classification.action,
                    "next_message": classification.message,
                    "task_summary": sdlc_core::task::summarize(&f.tasks),
                    "updated_at": f.updated_at,
                })
            })
            .collect();

        let milestone_summaries: Vec<serde_json::Value> = milestones
            .iter()
            .map(|m| {
                serde_json::json!({
                    "slug": m.slug,
                    "title": m.title,
                    "status": m.status,
                    "features": m.features,
                    "created_at": m.created_at,
                })
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "project": state.project,
            "active_features": state.active_features,
            "active_work": state.active_work,
            "blocked": state.blocked,
            "features": feature_summaries,
            "milestones": milestone_summaries,
            "last_updated": state.last_updated,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
