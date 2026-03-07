use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;
use sdlc_core::comment::{add_comment, CommentFlag, CommentTarget};
use sdlc_core::types::{ActionType, Phase};

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

        // Find parent milestone (if any)
        let milestone_info = sdlc_core::milestone::Milestone::list(&root)
            .unwrap_or_default()
            .into_iter()
            .find(|m| m.features.contains(&f.slug))
            .map(|m| serde_json::json!({ "slug": m.slug, "title": m.title }));

        let artifacts: Vec<serde_json::Value> = f
            .artifacts
            .iter()
            .map(|a| {
                let content = if a.exists_on_disk(&root) {
                    std::fs::read_to_string(root.join(&a.path)).ok()
                } else {
                    None
                };
                serde_json::json!({
                    "artifact_type": a.artifact_type,
                    "status": a.status,
                    "path": a.path,
                    "content": content,
                    "approved_at": a.approved_at,
                    "approved_by": a.approved_by,
                    "rejected_at": a.rejected_at,
                    "rejection_reason": a.rejection_reason,
                    "waived_at": a.waived_at,
                    "waive_reason": a.waive_reason,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": f.slug,
            "title": f.title,
            "description": f.description,
            "phase": f.phase,
            "archived": f.archived,
            "blocked": f.is_blocked(),
            "blockers": f.blockers,
            "artifacts": artifacts,
            "tasks": f.tasks,
            "comments": f.comments,
            "phase_history": f.phase_history,
            "dependencies": f.dependencies,
            "created_at": f.created_at,
            "updated_at": f.updated_at,
            "milestone": milestone_info,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/features/:slug/directive — return the full Classification directive via serde.
///
/// Identical in semantics to `/next` but serializes the `Classification` struct
/// directly, ensuring all fields (including future additions) are always present.
pub async fn get_feature_directive(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<sdlc_core::classifier::Classification>, AppError> {
    let root = app.root.clone();
    let classification = tokio::task::spawn_blocking(move || {
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
        Ok::<_, sdlc_core::SdlcError>(classifier.classify(&ctx))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(classification))
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

/// POST /api/features/:slug/merge — finalize the merge phase, transitioning to released.
pub async fn merge_feature(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::config::Config::load(&root)?;
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;

        if feature.phase != Phase::Merge {
            return Err(sdlc_core::SdlcError::InvalidPhase(format!(
                "cannot finalize merge for '{}' from phase '{}'; move it to 'merge' first",
                slug, feature.phase
            )));
        }

        feature.transition(Phase::Released, &config)?;
        feature.save(&root)?;

        let mut state = sdlc_core::state::State::load(&root)?;
        state.record_action(&slug, ActionType::Merge, Phase::Released, "merged");
        state.complete_directive(&slug);
        state.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": slug,
            "phase": "released",
            "merged": true,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize, Default)]
pub struct RemoveBlockerBody {
    #[serde(default)]
    pub reason: Option<String>,
}

/// DELETE /api/features/:slug/blockers/:idx — remove a blocker by index.
pub async fn remove_blocker(
    State(app): State<AppState>,
    Path((slug, idx)): Path<(String, usize)>,
    body: Option<Json<RemoveBlockerBody>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let reason = body.and_then(|b| b.0.reason);
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        let blocker_text = feature.blockers.get(idx).cloned().unwrap_or_default();
        feature.remove_blocker(idx)?;
        if let Some(r) = reason.filter(|r| !r.trim().is_empty()) {
            add_comment(
                &mut feature.comments,
                &mut feature.next_comment_seq,
                format!("Blocker removed: '{blocker_text}'. Reason: {r}"),
                Some(CommentFlag::Decision),
                CommentTarget::Feature,
                None,
            );
        }
        feature.save(&root)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({ "ok": true }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Human QA submission
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct HumanQaBody {
    pub verdict: String,
    #[serde(default)]
    pub notes: String,
}

/// POST /api/features/:slug/human-qa — submit a human-run QA result for a feature.
///
/// Writes a `qa-results.md` Draft artifact on the feature, ready for the agent to
/// approve via the normal state machine flow.
pub async fn submit_human_qa(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<HumanQaBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    use sdlc_core::types::ArtifactType;

    // Validate verdict string.
    let verdict_display = match body.verdict.as_str() {
        "pass" => "Pass",
        "pass_with_tasks" => "Pass with Tasks",
        "failed" => "Fail",
        other => {
            return Err(AppError::unprocessable_json(serde_json::json!({
                "error": format!("invalid verdict '{}'; must be pass, pass_with_tasks, or failed", other)
            })));
        }
    };

    // Notes required for non-pass verdicts.
    if body.verdict != "pass" && body.notes.trim().is_empty() {
        return Err(AppError::unprocessable_json(serde_json::json!({
            "error": "notes are required when verdict is not pass"
        })));
    }

    let root = app.root.clone();
    let notes = body.notes.clone();
    let verdict_display = verdict_display.to_string();

    let result = tokio::task::spawn_blocking(move || {
        use chrono::Utc;

        // Verify feature exists.
        sdlc_core::feature::Feature::load(&root, &slug)?;

        let now = Utc::now();
        let qa_content = format!(
            "## Verdict\n\
             {verdict_display}\n\
             \n\
             ## Notes\n\
             {notes}\n\
             \n\
             Runner: human (manual)\n\
             Completed: {now}\n"
        );

        // Write qa-results.md.
        let qa_path = root.join(format!(".sdlc/features/{slug}/qa-results.md"));
        sdlc_core::io::atomic_write(&qa_path, qa_content.as_bytes())?;

        // Reload and mark as draft.
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;
        feature.mark_artifact_draft(ArtifactType::QaResults)?;
        feature.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": feature.slug,
            "artifact": "qa_results",
            "status": "draft"
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    let _ = app.event_tx.send(crate::state::SseMessage::Update);

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
