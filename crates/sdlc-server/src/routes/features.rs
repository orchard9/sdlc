use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/features — list all features.
pub async fn list_features(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let features = sdlc_core::feature::Feature::list(&root)?;
        let list: Vec<serde_json::Value> = features
            .iter()
            .map(|f| {
                serde_json::json!({
                    "slug": f.slug,
                    "title": f.title,
                    "description": f.description,
                    "phase": f.phase,
                    "archived": f.archived,
                    "blocked": f.is_blocked(),
                    "task_summary": sdlc_core::task::summarize(&f.tasks),
                    "updated_at": f.updated_at,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/features/:slug — full feature detail.
pub async fn get_feature(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let f = sdlc_core::feature::Feature::load(&root, &slug)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": f.slug,
            "title": f.title,
            "description": f.description,
            "phase": f.phase,
            "archived": f.archived,
            "blocked": f.is_blocked(),
            "blockers": f.blockers,
            "artifacts": f.artifacts,
            "tasks": f.tasks,
            "comments": f.comments,
            "phase_history": f.phase_history,
            "dependencies": f.dependencies,
            "created_at": f.created_at,
            "updated_at": f.updated_at,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/features/:slug/next — classify next action.
pub async fn get_feature_next(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::config::Config::load(&root)?;
        let state = sdlc_core::state::State::load(&root)?;
        let feature = sdlc_core::feature::Feature::load(&root, &slug)?;

        let ctx = sdlc_core::classifier::EvalContext {
            feature: &feature,
            state: &state,
            config: &config,
            root: &root,
        };
        let classifier = sdlc_core::classifier::Classifier::new(sdlc_core::rules::default_rules());
        let c = classifier.classify(&ctx);

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "feature": c.feature,
            "title": c.title,
            "description": c.description,
            "current_phase": c.current_phase,
            "action": c.action,
            "message": c.message,
            "next_command": c.next_command,
            "output_path": c.output_path,
            "transition_to": c.transition_to,
            "task_id": c.task_id,
            "is_heavy": c.is_heavy,
            "timeout_minutes": c.timeout_minutes,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct CreateFeatureBody {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// POST /api/features — create a new feature.
pub async fn create_feature(
    State(app): State<AppState>,
    Json(body): Json<CreateFeatureBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let f = sdlc_core::feature::Feature::create_with_description(
            &root,
            body.slug,
            body.title,
            body.description,
        )?;

        // Add to active features in state
        if let Ok(mut state) = sdlc_core::state::State::load(&root) {
            state.add_active_feature(&f.slug);
            let _ = state.save(&root);
        }

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": f.slug,
            "title": f.title,
            "phase": f.phase,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct TransitionBody {
    pub phase: String,
}

/// POST /api/features/:slug/transition — advance feature phase.
pub async fn transition_feature(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<TransitionBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::config::Config::load(&root)?;
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        let target: sdlc_core::types::Phase =
            body.phase.parse().map_err(|e: sdlc_core::SdlcError| e)?;

        feature.transition(target, &config)?;
        feature.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": feature.slug,
            "phase": feature.phase,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
