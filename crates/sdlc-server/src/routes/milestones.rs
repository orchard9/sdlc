use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;
use crate::subprocess;

/// GET /api/milestones — list all milestones.
pub async fn list_milestones(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let milestones = sdlc_core::milestone::Milestone::list(&root)?;
        let list: Vec<serde_json::Value> = milestones
            .iter()
            .map(|m| {
                serde_json::json!({
                    "slug": m.slug,
                    "title": m.title,
                    "description": m.description,
                    "status": m.status,
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
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": m.slug,
            "title": m.title,
            "description": m.description,
            "status": m.status,
            "features": m.features,
            "created_at": m.created_at,
            "updated_at": m.updated_at,
            "completed_at": m.completed_at,
            "cancelled_at": m.cancelled_at,
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

        if let Ok(mut state) = sdlc_core::state::State::load(&root) {
            state.add_milestone(&m.slug);
            let _ = state.save(&root);
        }

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": m.slug,
            "title": m.title,
            "status": m.status,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct RunMilestoneBody {
    pub mode: Option<String>,
}

/// POST /api/milestones/:slug/run — spawn sdlc_milestone_driver subprocess.
pub async fn run_milestone(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<RunMilestoneBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();

    // Validate milestone exists on a blocking thread
    tokio::task::spawn_blocking({
        let root = root.clone();
        let slug = slug.clone();
        move || sdlc_core::milestone::Milestone::load(&root, &slug)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    let mode = body.mode.unwrap_or_else(|| "auto".to_string());
    let argv = vec![
        "python".to_string(),
        "-m".to_string(),
        "sdlc_milestone_driver".to_string(),
        "--milestone".to_string(),
        slug.clone(),
        "--mode".to_string(),
        mode,
        "--root".to_string(),
        root.to_string_lossy().to_string(),
    ];

    let run_id = uuid::Uuid::new_v4().to_string();
    let handle = subprocess::spawn_process(argv, &root);

    app.sweep_completed_runs().await;
    app.runs.write().await.insert(run_id.clone(), handle);

    Ok(Json(serde_json::json!({ "run_id": run_id })))
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
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": m.slug,
            "title": m.title,
            "description": m.description,
            "status": m.status,
            "features": m.features,
            "created_at": m.created_at,
            "updated_at": m.updated_at,
            "completed_at": m.completed_at,
            "cancelled_at": m.cancelled_at,
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
