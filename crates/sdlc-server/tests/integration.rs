use axum::http::StatusCode;
use http_body_util::BodyExt;
use tempfile::TempDir;
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Bootstrap a minimal SDLC project inside the given temp directory.
fn init_project(dir: &TempDir) {
    let config = sdlc_core::config::Config::new("test-project");
    sdlc_core::io::ensure_dir(&dir.path().join(".sdlc")).unwrap();
    sdlc_core::io::ensure_dir(&dir.path().join(".sdlc/features")).unwrap();
    sdlc_core::io::ensure_dir(&dir.path().join(".sdlc/milestones")).unwrap();
    config.save(dir.path()).unwrap();
    let state = sdlc_core::state::State::new("test-project");
    state.save(dir.path()).unwrap();
}

/// Send a GET request via `oneshot` and return (status, parsed JSON body).
async fn get(app: axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let req = axum::http::Request::builder()
        .uri(uri)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
    (status, json)
}

/// Send a POST request with a JSON body via `oneshot` and return (status, parsed JSON body).
async fn post_json(
    app: axum::Router,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
    (status, json)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a feature and mark a specific artifact as draft, then write content to disk.
fn create_feature_with_draft_artifact(
    dir: &TempDir,
    slug: &str,
    artifact_type: sdlc_core::types::ArtifactType,
    content: &str,
) {
    let mut feature =
        sdlc_core::feature::Feature::create(dir.path(), slug, "Test Feature").unwrap();
    feature.mark_artifact_draft(artifact_type).unwrap();
    feature.save(dir.path()).unwrap();
    let artifact_path = dir.path().join(format!(
        ".sdlc/features/{}/{}",
        slug,
        artifact_type.filename()
    ));
    std::fs::write(artifact_path, content).unwrap();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_state_returns_project_summary() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = get(app, "/api/state").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["project"], "test-project");
    assert!(json["features"].is_array());
    assert!(json["milestones"].is_array());
}

#[tokio::test]
async fn get_config_returns_project_config() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = get(app, "/api/config").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["project"]["name"], "test-project");
    assert_eq!(json["version"], 1);
}

#[tokio::test]
async fn get_config_returns_error_when_not_initialized() {
    let dir = TempDir::new().unwrap();
    // Deliberately do NOT call init_project.

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, _json) = get(app, "/api/config").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_features_returns_empty_list() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = get(app, "/api/features").await;

    assert_eq!(status, StatusCode::OK);
    let arr = json.as_array().expect("expected JSON array");
    assert!(arr.is_empty(), "expected empty features list");
}

#[tokio::test]
async fn create_and_get_feature() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // POST /api/features — create a new feature
    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = post_json(
        app,
        "/api/features",
        serde_json::json!({
            "slug": "test-feat",
            "title": "Test Feature"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["slug"], "test-feat");
    assert_eq!(json["title"], "Test Feature");

    // GET /api/features/test-feat — retrieve the feature we just created
    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = get(app, "/api/features/test-feat").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["slug"], "test-feat");
    assert_eq!(json["title"], "Test Feature");
    assert_eq!(json["phase"], "draft");
}

#[tokio::test]
async fn get_artifact_returns_missing_status() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create a feature (no draft artifact file written)
    sdlc_core::feature::Feature::create(dir.path(), "feat-a", "Feature A").unwrap();

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = get(app, "/api/artifacts/feat-a/spec").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "missing");
    assert!(json["content"].is_null());
}

#[tokio::test]
async fn get_artifact_returns_content_when_file_exists() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    create_feature_with_draft_artifact(
        &dir,
        "feat-b",
        sdlc_core::types::ArtifactType::Spec,
        "# Spec Content",
    );

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = get(app, "/api/artifacts/feat-b/spec").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "draft");
    assert_eq!(json["content"], "# Spec Content");
}

#[tokio::test]
async fn approve_artifact_succeeds() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    create_feature_with_draft_artifact(
        &dir,
        "feat-c",
        sdlc_core::types::ArtifactType::Spec,
        "# Spec",
    );

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = post_json(
        app,
        "/api/artifacts/feat-c/spec/approve",
        serde_json::json!({ "by": "test" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "approved");
}

#[tokio::test]
async fn reject_artifact_succeeds() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    create_feature_with_draft_artifact(
        &dir,
        "feat-d",
        sdlc_core::types::ArtifactType::Spec,
        "# Spec",
    );

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = post_json(
        app,
        "/api/artifacts/feat-d/spec/reject",
        serde_json::json!({ "reason": "needs more detail" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "rejected");
}

#[tokio::test]
async fn waive_artifact_succeeds() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Waive works on a missing artifact too
    sdlc_core::feature::Feature::create(dir.path(), "feat-e", "Feature E").unwrap();

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = post_json(
        app,
        "/api/artifacts/feat-e/spec/waive",
        serde_json::json!({ "reason": "simple feature, no spec needed" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "waived");
}

#[tokio::test]
async fn get_feature_includes_artifact_content() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    create_feature_with_draft_artifact(
        &dir,
        "feat-f",
        sdlc_core::types::ArtifactType::Spec,
        "# Feature Spec",
    );

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, json) = get(app, "/api/features/feat-f").await;

    assert_eq!(status, StatusCode::OK);
    let artifacts = json["artifacts"].as_array().expect("artifacts is array");
    let spec = artifacts
        .iter()
        .find(|a| a["artifact_type"] == "spec")
        .expect("spec artifact present");
    assert_eq!(spec["status"], "draft");
    assert_eq!(spec["content"], "# Feature Spec");
}

#[tokio::test]
async fn artifact_not_found_returns_error() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf());
    let (status, _json) = get(app, "/api/artifacts/nonexistent-feature/spec").await;

    assert_ne!(status, StatusCode::OK);
}
