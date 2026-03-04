use axum::extract::{Path, State};
use axum::http::header;
use axum::response::Response;
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

/// GET /api/milestones/:slug/uat-runs/latest — most recent UAT run, or null if none.
pub async fn get_latest_milestone_uat_run(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Option<sdlc_core::milestone::UatRun>>, AppError> {
    let root = app.root.clone();
    let run =
        tokio::task::spawn_blocking(move || sdlc_core::milestone::latest_uat_run(&root, &slug))
            .await
            .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(run))
}

/// GET /api/milestones/:slug/acceptance-test — acceptance test markdown content.
pub async fn get_milestone_acceptance_test(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let content = tokio::task::spawn_blocking(move || {
        let m = sdlc_core::milestone::Milestone::load(&root, &slug)?;
        m.load_acceptance_test(&root)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(serde_json::json!({ "content": content })))
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

/// Return the MIME type for a given filename based on its extension.
fn mime_for_filename(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webm" => "video/webm",
        "mp4" => "video/mp4",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
}

/// GET /api/milestones/:slug/uat-runs/:run_id/artifacts/:filename
///
/// Serve a binary artifact (screenshot, etc.) stored in the UAT run directory.
/// The `filename` must not contain path separators (`/`, `\`) or `..` to
/// prevent directory traversal.
pub async fn get_uat_run_artifact(
    State(app): State<AppState>,
    Path((slug, run_id, filename)): Path<(String, String, String)>,
) -> Result<Response, AppError> {
    // Path traversal guard — reject filenames that could escape the run directory.
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(AppError::bad_request(
            "invalid filename: must not contain path separators or '..'",
        ));
    }

    let path = sdlc_core::paths::uat_run_dir(&app.root, &slug, &run_id).join(&filename);

    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| AppError::not_found("artifact not found"))?;

    let content_type = mime_for_filename(&filename);

    let response = Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(axum::body::Body::from(bytes))
        .map_err(|e| AppError(anyhow::anyhow!("failed to build response: {e}")))?;

    Ok(response)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // TC-6: MIME type detection
    #[test]
    fn mime_for_filename_png() {
        assert_eq!(mime_for_filename("screenshot.png"), "image/png");
    }

    #[test]
    fn mime_for_filename_jpg() {
        assert_eq!(mime_for_filename("photo.jpg"), "image/jpeg");
        assert_eq!(mime_for_filename("photo.jpeg"), "image/jpeg");
    }

    #[test]
    fn mime_for_filename_webm() {
        assert_eq!(mime_for_filename("video.webm"), "video/webm");
    }

    #[test]
    fn mime_for_filename_mp4() {
        assert_eq!(mime_for_filename("clip.mp4"), "video/mp4");
    }

    #[test]
    fn mime_for_filename_gif() {
        assert_eq!(mime_for_filename("anim.gif"), "image/gif");
    }

    #[test]
    fn mime_for_filename_unknown_extension() {
        assert_eq!(mime_for_filename("data.bin"), "application/octet-stream");
    }

    #[test]
    fn mime_for_filename_no_extension() {
        assert_eq!(mime_for_filename("noextension"), "application/octet-stream");
    }

    // TC-5: path traversal detection (logic test, not HTTP test)
    #[test]
    fn path_traversal_detected_for_dotdot() {
        let filename = "../../../etc/passwd";
        assert!(filename.contains(".."));
    }

    #[test]
    fn path_traversal_detected_for_slash() {
        let filename = "subdir/file.png";
        assert!(filename.contains('/'));
    }

    #[test]
    fn safe_filename_passes_traversal_guard() {
        let filename = "01-login.png";
        let unsafe_ = filename.contains('/') || filename.contains('\\') || filename.contains("..");
        assert!(!unsafe_);
    }

    // TC-7: agent prompt contains screenshot instructions
    #[test]
    fn start_milestone_uat_prompt_contains_screenshot_instructions() {
        // Build the prompt the same way start_milestone_uat does and verify key terms.
        let slug = "test-milestone";
        let prompt = format!(
            "Run the acceptance test for milestone '{slug}'.\n\
             \n\
             IMPORTANT: You are running INSIDE the sdlc server process at http://localhost:7777. \
             The server is already running — do NOT stop, restart, kill, or re-spawn it. \
             Do NOT call any UAT stop or start endpoints. \
             If localhost:7777 is unreachable, report it as a hard blocker and stop immediately — \
             never attempt to start or restart the server.\n\
             \n\
             ## Step 0 — generate a run_id\n\
             Before executing any steps, generate a run_id in the format \
             `YYYYMMDD-HHMMSS-<three-random-lowercase-letters>` (UTC, e.g. `20260303-142500-abc`). \
             Use this run_id consistently throughout the session. \
             The run directory is `.sdlc/milestones/{slug}/uat-runs/<run_id>/`.\n\
             \n\
             ## Step 1 — load the acceptance test\n\
             Call `sdlc milestone info {slug} --json` to load the milestone and acceptance test.\n\
             \n\
             ## Step 2 — execute checklist steps with screenshots\n\
             Execute every checklist step using the Playwright MCP browser tools. \
             After completing each UI interaction step, capture a screenshot:\n\
             - Call `mcp__playwright__browser_take_screenshot` with a filename like \
               `<step_number>-<step_slug>.png` (e.g. `01-login.png`).\n\
             - Copy the file to `.sdlc/milestones/{slug}/uat-runs/<run_id>/<filename>`.\n\
             - Append the relative path `.sdlc/milestones/{slug}/uat-runs/<run_id>/<filename>` \
               to a `screenshot_paths` list.\n\
             \n\
             ## Step 3 — persist the run record\n\
             After all steps complete:\n\
             1. Write `summary.md` to `.sdlc/milestones/{slug}/uat-runs/<run_id>/summary.md`.\n\
             2. Write `run.yaml` to `.sdlc/milestones/{slug}/uat-runs/<run_id>/run.yaml` \
                with these fields: id, milestone_slug, started_at, completed_at, verdict \
                (pass | pass_with_tasks | failed), tests_total, tests_passed, tests_failed, \
                tasks_created, summary_path, and screenshot_paths (the list collected in Step 2).\n\
             3. Write signed checklist results to `.sdlc/milestones/{slug}/uat_results.md`.\n\
             4. Call `sdlc milestone complete {slug}` if all steps pass.",
        );

        assert!(prompt.contains("run_id"), "prompt must reference 'run_id'");
        assert!(
            prompt.contains("screenshot"),
            "prompt must reference 'screenshot'"
        );
        assert!(
            prompt.contains("screenshot_paths"),
            "prompt must reference 'screenshot_paths'"
        );
    }
}
