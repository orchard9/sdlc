use axum::extract::{Query, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(serde::Deserialize)]
pub struct ReadyParams {
    pub phase: Option<String>,
}

/// GET /api/query/search?q=<query>&limit=<n>
pub async fn search(
    State(app): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let limit = params.limit.unwrap_or(10);

        // Feature search
        let features = sdlc_core::feature::Feature::list(&root)?;
        let feature_index = sdlc_core::search::FeatureIndex::build(&features, &root)?;
        let feature_results = feature_index.search(&params.q, limit)?;

        let out: Vec<serde_json::Value> = feature_results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "slug": r.slug,
                    "title": r.title,
                    "phase": r.phase,
                    "score": r.score,
                })
            })
            .collect();

        // Ponder search
        let ponder_entries = sdlc_core::ponder::PonderEntry::list(&root)?;
        let ponder_artifacts: Vec<_> = ponder_entries
            .iter()
            .map(|e| {
                let arts = sdlc_core::ponder::list_artifacts(&root, &e.slug).unwrap_or_default();
                (e.clone(), arts)
            })
            .collect();
        let ponder_index = sdlc_core::search::PonderIndex::build(&ponder_artifacts, &root)?;
        let ponder_out = ponder_index
            .search(&params.q, limit)?
            .iter()
            .map(|r| {
                serde_json::json!({
                    "slug": r.slug,
                    "title": r.title,
                    "status": r.status,
                    "score": r.score,
                })
            })
            .collect::<Vec<_>>();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "results": out,
            "ponder_results": ponder_out,
            "parse_error": serde_json::Value::Null,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/query/search-tasks?q=<query>&limit=<n>
pub async fn search_tasks(
    State(app): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let limit = params.limit.unwrap_or(10);
        let features = sdlc_core::feature::Feature::list(&root)?;
        let index = sdlc_core::search::TaskIndex::build(&features)?;
        let results = index.search(&params.q, limit)?;

        let out: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "feature_slug": r.feature_slug,
                    "task_id": r.task_id,
                    "title": r.title,
                    "status": r.status,
                    "score": r.score,
                })
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "results": out,
            "parse_error": serde_json::Value::Null,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/query/blocked
pub async fn blocked(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let features = sdlc_core::feature::Feature::list(&root)?;
        let out: Vec<serde_json::Value> = features
            .iter()
            .filter(|f| f.is_blocked())
            .map(|f| {
                serde_json::json!({
                    "slug": f.slug,
                    "title": f.title,
                    "blockers": f.blockers,
                })
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(out))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/query/ready?phase=<optional>
pub async fn ready(
    State(app): State<AppState>,
    Query(params): Query<ReadyParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::config::Config::load(&root)?;
        let state = sdlc_core::state::State::load(&root)?;
        let features = sdlc_core::feature::Feature::list(&root)?;
        let classifier = sdlc_core::classifier::Classifier::new(sdlc_core::rules::default_rules());

        let out: Vec<serde_json::Value> = features
            .iter()
            .filter(|f| !f.archived && !f.is_blocked())
            .filter(|f| {
                params
                    .phase
                    .as_deref()
                    .is_none_or(|p| f.phase.to_string() == p)
            })
            .filter_map(|f| {
                let ctx = sdlc_core::classifier::EvalContext {
                    feature: f,
                    state: &state,
                    config: &config,
                    root: &root,
                };
                let c = classifier.classify(&ctx);
                if matches!(
                    c.action,
                    sdlc_core::types::ActionType::WaitForApproval
                        | sdlc_core::types::ActionType::Done
                        | sdlc_core::types::ActionType::UnblockDependency
                ) {
                    None
                } else {
                    Some(serde_json::json!({
                        "slug": f.slug,
                        "phase": f.phase.to_string(),
                        "action": c.action.as_str(),
                        "message": c.message,
                        "next_command": c.next_command,
                    }))
                }
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(out))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/query/needs-approval
pub async fn needs_approval(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::config::Config::load(&root)?;
        let state = sdlc_core::state::State::load(&root)?;
        let features = sdlc_core::feature::Feature::list(&root)?;
        let classifier = sdlc_core::classifier::Classifier::new(sdlc_core::rules::default_rules());

        let out: Vec<serde_json::Value> = features
            .iter()
            .filter(|f| !f.archived)
            .filter_map(|f| {
                let ctx = sdlc_core::classifier::EvalContext {
                    feature: f,
                    state: &state,
                    config: &config,
                    root: &root,
                };
                let c = classifier.classify(&ctx);
                if is_approval_action(c.action) {
                    Some(serde_json::json!({
                        "slug": f.slug,
                        "phase": f.phase.to_string(),
                        "action": c.action.as_str(),
                        "message": c.message,
                        "next_command": c.next_command,
                    }))
                } else {
                    None
                }
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(out))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// Returns true for actions that require verification or human sign-off before the phase
/// can advance. Includes both agent-executable approve_* steps and the WaitForApproval
/// HITL gate — the latter surfaces features that are explicitly blocked pending human
/// sign-off, which is a distinct consumer use-case from the agentive approve_* actions.
fn is_approval_action(action: sdlc_core::types::ActionType) -> bool {
    matches!(
        action,
        sdlc_core::types::ActionType::ApproveSpec
            | sdlc_core::types::ActionType::ApproveDesign
            | sdlc_core::types::ActionType::ApproveTasks
            | sdlc_core::types::ActionType::ApproveQaPlan
            | sdlc_core::types::ActionType::ApproveReview
            | sdlc_core::types::ActionType::ApproveAudit
            | sdlc_core::types::ActionType::ApproveMerge
            | sdlc_core::types::ActionType::WaitForApproval
    )
}
