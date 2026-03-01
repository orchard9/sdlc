use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/milestones — list all milestones.
pub async fn list_milestones(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let milestones = sdlc_core::milestone::Milestone::list(&root)?;
        let features = sdlc_core::feature::Feature::list(&root)?;
        let list: Vec<serde_json::Value> = milestones
            .iter()
            .map(|m| {
                serde_json::json!({
                    "slug": m.slug,
                    "title": m.title,
                    "description": m.description,
                    "vision": m.vision,
                    "status": m.compute_status(&features),
                    "features": m.features,
                    "created_at": m.created_at,
                    "updated_at": m.updated_at,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/milestones/:slug — milestone detail.
pub async fn get_milestone(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let m = sdlc_core::milestone::Milestone::load(&root, &slug)?;
        let features = sdlc_core::feature::Feature::list(&root)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": m.slug,
            "title": m.title,
            "description": m.description,
            "vision": m.vision,
            "status": m.compute_status(&features),
            "features": m.features,
            "created_at": m.created_at,
            "updated_at": m.updated_at,
            "skipped_at": m.skipped_at,
            "released_at": m.released_at,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/milestones/:slug/review — classifications for all milestone features.
pub async fn review_milestone(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let m = sdlc_core::milestone::Milestone::load(&root, &slug)?;
        let config = sdlc_core::config::Config::load(&root)?;
        let state = sdlc_core::state::State::load(&root)?;
        let classifier = sdlc_core::classifier::Classifier::new(sdlc_core::rules::default_rules());

        let reviews: Vec<serde_json::Value> = m
            .features
            .iter()
            .filter_map(|fs| sdlc_core::feature::Feature::load(&root, fs).ok())
            .map(|f| {
                let ctx = sdlc_core::classifier::EvalContext {
                    feature: &f,
                    state: &state,
                    config: &config,
                    root: &root,
                };
                let c = classifier.classify(&ctx);
                serde_json::json!({
                    "feature": c.feature,
                    "phase": c.current_phase,
                    "action": c.action,
                    "message": c.message,
                })
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "milestone": slug,
            "features": reviews,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct CreateMilestoneBody {
    pub slug: String,
    pub title: String,
}

/// POST /api/milestones — create a new milestone.
pub async fn create_milestone(
    State(app): State<AppState>,
    Json(body): Json<CreateMilestoneBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let m = sdlc_core::milestone::Milestone::create(&root, body.slug, body.title)?;

        let mut state = sdlc_core::state::State::load(&root)?;
        state.add_milestone(&m.slug);
        state.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": m.slug,
            "title": m.title,
            "status": m.compute_status(&[]),
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct ReorderFeaturesBody {
    pub features: Vec<String>,
}

/// PUT /api/milestones/:slug/features/order — reorder features in a milestone.
pub async fn reorder_milestone_features(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<ReorderFeaturesBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut m = sdlc_core::milestone::Milestone::load(&root, &slug)?;
        let refs: Vec<&str> = body.features.iter().map(|s| s.as_str()).collect();
        m.reorder_features(&refs)?;
        m.save(&root)?;
        let features = sdlc_core::feature::Feature::list(&root)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": m.slug,
            "title": m.title,
            "description": m.description,
            "vision": m.vision,
            "status": m.compute_status(&features),
            "features": m.features,
            "created_at": m.created_at,
            "updated_at": m.updated_at,
            "skipped_at": m.skipped_at,
            "released_at": m.released_at,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct AddFeatureBody {
    pub feature_slug: String,
}

/// GET /api/milestones/:slug/uat-runs — list all UAT runs for a milestone, newest-first.
pub async fn list_milestone_uat_runs(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<sdlc_core::milestone::UatRun>>, AppError> {
    let root = app.root.clone();
    let runs =
        tokio::task::spawn_blocking(move || sdlc_core::milestone::list_uat_runs(&root, &slug))
            .await
            .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(runs))
}

/// GET /api/milestones/:slug/uat-runs/latest — most recent UAT run, or 404 if none.
pub async fn get_latest_milestone_uat_run(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<sdlc_core::milestone::UatRun>, AppError> {
    let root = app.root.clone();
    let run =
        tokio::task::spawn_blocking(move || sdlc_core::milestone::latest_uat_run(&root, &slug))
            .await
            .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    match run {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::not_found(
            "no UAT runs recorded for this milestone",
        )),
    }
}

/// POST /api/milestones/:slug/features — add a feature to a milestone.
pub async fn add_feature_to_milestone(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<AddFeatureBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut m = sdlc_core::milestone::Milestone::load(&root, &slug)?;
        // Verify the feature exists
        sdlc_core::feature::Feature::load(&root, &body.feature_slug)?;

        m.add_feature(&body.feature_slug);
        m.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": m.slug,
            "features": m.features,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
