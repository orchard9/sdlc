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

/// Send a DELETE request via `oneshot` and return (status, parsed JSON body).
async fn delete_req(app: axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let req = axum::http::Request::builder()
        .method("DELETE")
        .uri(uri)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
    (status, json)
}

/// Send a PATCH request with a JSON body via `oneshot` and return (status, parsed JSON body).
async fn patch_json(
    app: axum::Router,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = axum::http::Request::builder()
        .method("PATCH")
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

/// Send a POST request with a JSON body and a custom Host header.
async fn post_json_with_host(
    app: axum::Router,
    uri: &str,
    host: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(uri)
        .header("host", host)
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

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
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

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = get(app, "/api/config").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["project"]["name"], "test-project");
    assert_eq!(json["version"], 1);
}

#[tokio::test]
async fn get_config_returns_error_when_not_initialized() {
    let dir = TempDir::new().unwrap();
    // Deliberately do NOT call init_project.

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _json) = get(app, "/api/config").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_features_returns_empty_list() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
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
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
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
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = get(app, "/api/features/test-feat").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["slug"], "test-feat");
    assert_eq!(json["title"], "Test Feature");
    assert_eq!(json["phase"], "draft");
}

#[tokio::test]
async fn get_feature_directive_returns_full_classification() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc_core::feature::Feature::create_with_description(
        dir.path(),
        "feat-directive".to_string(),
        "Directive Feature".to_string(),
        Some("A feature with a description".to_string()),
    )
    .unwrap();

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = get(app, "/api/features/feat-directive/directive").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["feature"], "feat-directive");
    assert_eq!(json["title"], "Directive Feature");
    assert_eq!(json["description"], "A feature with a description");
    assert_eq!(json["current_phase"], "draft");
    assert_eq!(json["action"], "create_spec");
    assert!(!json["message"].as_str().unwrap_or("").is_empty());
    assert!(!json["next_command"].as_str().unwrap_or("").is_empty());
    assert!(json.get("is_heavy").is_some());
    assert!(json.get("timeout_minutes").is_some());
}

#[tokio::test]
async fn get_feature_directive_returns_error_for_missing_feature() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let req = axum::http::Request::builder()
        .uri("/api/features/does-not-exist/directive")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_artifact_returns_missing_status() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create a feature (no draft artifact file written)
    sdlc_core::feature::Feature::create(dir.path(), "feat-a", "Feature A").unwrap();

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
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

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
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

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
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

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
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

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
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

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
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

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _json) = get(app, "/api/artifacts/nonexistent-feature/spec").await;

    assert_ne!(status, StatusCode::OK);
}

// ---------------------------------------------------------------------------
// draft_artifact endpoint tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn draft_artifact_ok() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create a feature (artifact starts as missing)
    sdlc_core::feature::Feature::create(dir.path(), "feat-draft-ok", "Draft OK").unwrap();

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = post_json(
        app,
        "/api/artifacts/feat-draft-ok/spec/draft",
        serde_json::json!({}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["slug"], "feat-draft-ok");
    assert_eq!(json["artifact_type"], "spec");
    assert_eq!(json["status"], "draft");
}

#[tokio::test]
async fn draft_artifact_feature_not_found() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _json) = post_json(
        app,
        "/api/artifacts/nonexistent/spec/draft",
        serde_json::json!({}),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn draft_artifact_invalid_type() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc_core::feature::Feature::create(dir.path(), "feat-draft-bad-type", "Draft Bad Type")
        .unwrap();

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _json) = post_json(
        app,
        "/api/artifacts/feat-draft-bad-type/bogus-type/draft",
        serde_json::json!({}),
    )
    .await;

    // ArtifactNotFound maps to 404
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// merge_feature endpoint tests
// ---------------------------------------------------------------------------

/// Create a feature whose manifest phase is set directly to `merge`.
/// This bypasses the normal transition artifact gates, which is fine for
/// testing the HTTP endpoint's own phase-check behaviour.
fn create_feature_in_merge_phase(dir: &TempDir, slug: &str) {
    // Create the feature directory + manifest via the normal API, then
    // overwrite the phase field by rewriting the YAML directly.
    sdlc_core::feature::Feature::create(dir.path(), slug, "Merge Feature").unwrap();
    let manifest_path = dir
        .path()
        .join(format!(".sdlc/features/{}/manifest.yaml", slug));
    let data = std::fs::read_to_string(&manifest_path).unwrap();
    let data = data.replace("phase: draft", "phase: merge");
    std::fs::write(&manifest_path, data).unwrap();
}

#[tokio::test]
async fn merge_feature_ok() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    create_feature_in_merge_phase(&dir, "feat-merge-ok");

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = post_json(
        app,
        "/api/features/feat-merge-ok/merge",
        serde_json::json!({}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["slug"], "feat-merge-ok");
    assert_eq!(json["phase"], "released");
    assert_eq!(json["merged"], true);
}

#[tokio::test]
async fn merge_feature_sets_phase_to_released() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    create_feature_in_merge_phase(&dir, "feat-merge-released");

    // Call merge
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _) = post_json(
        app,
        "/api/features/feat-merge-released/merge",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Verify the feature is now in released phase on disk
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = get(app, "/api/features/feat-merge-released").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["phase"], "released");
}

#[tokio::test]
async fn merge_feature_wrong_phase() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create a feature in draft phase (not merge)
    sdlc_core::feature::Feature::create(dir.path(), "feat-merge-wrong", "Merge Wrong Phase")
        .unwrap();

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _json) = post_json(
        app,
        "/api/features/feat-merge-wrong/merge",
        serde_json::json!({}),
    )
    .await;

    // InvalidPhase maps to 400
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn merge_feature_not_found() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _json) = post_json(
        app,
        "/api/features/does-not-exist/merge",
        serde_json::json!({}),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Proxy / app-tunnel integration tests
// ---------------------------------------------------------------------------

/// inject_widget inserts script before </body>
#[test]
fn proxy_inject_widget_before_body() {
    use bytes::Bytes;
    let html = Bytes::from("<html><body><p>Hello</p></body></html>");
    let result = String::from_utf8(sdlc_server::proxy::inject_widget(html)).unwrap();
    assert!(result.contains("<script>"));
    let script_pos = result.find("<script>").unwrap();
    let body_close = result.rfind("</body>").unwrap();
    assert!(script_pos < body_close, "script must precede </body>");
}

/// /__sdlc/feedback is reachable even when a tunnel token is set (no cookie).
#[tokio::test]
async fn sdlc_feedback_endpoint_public_when_tunnel_active() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();

    // Build a router with a tunnel token set.
    let app = sdlc_server::build_router_for_test(
        dir.path().to_path_buf(),
        Some("secret-token".into()),
        None,
    );

    // POST to /__sdlc/feedback without any auth cookie — should succeed (200).
    let (status, _) = post_json_with_host(
        app,
        "/__sdlc/feedback",
        "fancy-rabbit.trycloudflare.com",
        serde_json::json!({ "content": "looks great!" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

/// /api/* routes accessed through the app tunnel host still require auth.
#[tokio::test]
async fn api_routes_blocked_via_app_tunnel_without_auth() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router_for_test(
        dir.path().to_path_buf(),
        Some("secret-token".into()),
        Some("fancy-rabbit.trycloudflare.com".into()),
    );

    let req = axum::http::Request::builder()
        .uri("/api/state")
        .header("host", "fancy-rabbit.trycloudflare.com")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// Non-app-tunnel requests fall through to the SPA (200 for unknown paths).
#[tokio::test]
async fn fallback_serves_spa_when_no_app_tunnel() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // No app tunnel host set — /some-unknown-path should serve the embedded SPA (200).
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let req = axum::http::Request::builder()
        .uri("/features")
        .header("host", "localhost:3141")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // The embedded SPA returns 200 for unknown frontend routes.
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Webhook ingestion tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_webhook_returns_202_with_id() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/webhooks/github")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(b"{\"event\":\"push\"}".to_vec()))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("response should be valid JSON");

    assert_eq!(status, StatusCode::ACCEPTED, "expected 202 Accepted");

    let id_str = json["id"]
        .as_str()
        .expect("response JSON must have an 'id' string field");
    uuid::Uuid::parse_str(id_str).expect("'id' must be a valid UUID");
}

#[tokio::test]
async fn post_webhook_preserves_raw_body_bytes() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);

    let raw_body = b"hello world \x00\xff binary".to_vec();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/webhooks/test")
        .header("content-type", "application/octet-stream")
        .body(axum::body::Body::from(raw_body.clone()))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // Verify the payload was stored correctly by reading from the DB directly.
    let db = sdlc_core::orchestrator::ActionDb::open(&sdlc_core::paths::orchestrator_db_path(
        dir.path(),
    ))
    .expect("ActionDb should open");
    let payloads = db.all_pending_webhooks().expect("should retrieve webhooks");
    assert_eq!(payloads.len(), 1, "one webhook should be stored");
    assert_eq!(
        payloads[0].raw_body, raw_body,
        "raw body bytes must be preserved exactly"
    );
    // route_path is normalized with a leading '/' to match WebhookRoute.path format
    assert_eq!(payloads[0].route_path, "/test");
    assert_eq!(
        payloads[0].content_type,
        Some("application/octet-stream".to_string())
    );
}

// ---------------------------------------------------------------------------
// Knowledge research endpoint tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_knowledge_research_returns_202() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Pre-create the knowledge entry so the handler finds it.
    sdlc_core::knowledge::create(dir.path(), "test-topic", "Test Topic", "uncategorized").unwrap();

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/knowledge/test-topic/research")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"topic":"test"}"#))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("response should be valid JSON");

    assert_eq!(
        status,
        StatusCode::ACCEPTED,
        "expected 202 Accepted, got {status}: {json}"
    );
}

#[tokio::test]
async fn test_knowledge_research_creates_entry_if_missing() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/knowledge/brand-new-topic/research")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"topic":"brand new topic"}"#))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("response should be valid JSON");

    assert_eq!(
        status,
        StatusCode::ACCEPTED,
        "expected 202 Accepted, got {status}: {json}"
    );

    // Verify entry was created on disk.
    let entry_dir = dir.path().join(".sdlc/knowledge/brand-new-topic");
    assert!(
        entry_dir.exists(),
        "knowledge entry directory should have been created"
    );
}

// ---------------------------------------------------------------------------
// Webhook events endpoint tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_webhook_events_empty_db_returns_empty_array() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = get(app, "/api/orchestrator/webhooks/events").await;

    assert_eq!(status, StatusCode::OK, "expected 200 OK");
    assert!(
        json.as_array().map(|a| a.is_empty()).unwrap_or(false),
        "expected empty JSON array, got: {json}"
    );
}

#[tokio::test]
async fn receive_webhook_records_event() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // POST a webhook to trigger event recording.
    let app1 = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/webhooks/test")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(b"{}".to_vec()))
        .unwrap();
    let resp = app1.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    // GET the events and verify one event was recorded.
    let app2 = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = get(app2, "/api/orchestrator/webhooks/events").await;

    assert_eq!(status, StatusCode::OK, "expected 200 OK");
    let events = json.as_array().expect("expected JSON array");
    assert_eq!(events.len(), 1, "expected exactly one event");

    let ev = &events[0];
    assert_eq!(ev["route_path"].as_str(), Some("/test"));

    let outcome = &ev["outcome"];
    assert_eq!(
        outcome["kind"].as_str(),
        Some("received"),
        "outcome.kind should be 'received'"
    );
}

/// QC-4: Sentinel file watcher fires ActionStateChanged when .sdlc/.orchestrator.state is written.
///
/// Construct an AppState directly (so we can subscribe to its event channel), write the sentinel
/// file, and assert that ActionStateChanged arrives within 2 seconds.
#[tokio::test]
async fn sentinel_watcher_fires_action_state_changed() {
    use sdlc_server::state::{AppState, SseMessage};

    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();

    // Construct AppState — this spawns the watcher tasks (we are inside a tokio runtime).
    let state = AppState::new(dir.path().to_path_buf());
    let mut rx = state.event_tx.subscribe();

    // Give the watcher task a moment to start its first poll.
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Write the sentinel file (simulating the orchestrator daemon tick).
    let sentinel = dir.path().join(".sdlc/.orchestrator.state");
    std::fs::write(
        &sentinel,
        r#"{"last_tick_at":"2026-01-01T00:00:00Z","actions_dispatched":0,"webhooks_dispatched":0}"#,
    )
    .unwrap();

    // Wait up to 2 seconds for ActionStateChanged.
    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(2);
    let mut got_signal = false;
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(tokio::time::Duration::from_millis(900), rx.recv()).await {
            Ok(Ok(SseMessage::ActionStateChanged)) => {
                got_signal = true;
                break;
            }
            Ok(Ok(_)) => {} // other messages — keep waiting
            Ok(Err(_)) | Err(_) => break,
        }
    }

    assert!(
        got_signal,
        "ActionStateChanged must be received within 2s of sentinel write"
    );
}

// ---------------------------------------------------------------------------
// Orchestrator actions CRUD integration tests
// ---------------------------------------------------------------------------

fn action_body() -> serde_json::Value {
    serde_json::json!({
        "label": "nightly-audit",
        "tool_name": "quality-check",
        "tool_input": {},
        "next_tick_at": "2099-01-01T00:00:00Z"
    })
}

#[tokio::test]
async fn list_actions_empty() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = get(app, "/api/orchestrator/actions").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_action_success() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = post_json(app, "/api/orchestrator/actions", action_body()).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["label"], "nightly-audit");
    assert_eq!(json["tool_name"], "quality-check");
    assert_eq!(json["trigger"]["type"], "scheduled");
    assert_eq!(json["status"]["type"], "pending");
    assert!(json["id"].is_string());
}

#[tokio::test]
async fn list_actions_returns_created() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create one action
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _) = post_json(app, "/api/orchestrator/actions", action_body()).await;
    assert_eq!(status, StatusCode::CREATED);

    // List should now have one entry
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = get(app, "/api/orchestrator/actions").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["label"], "nightly-audit");
}

#[tokio::test]
async fn create_action_empty_label() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let body = serde_json::json!({
        "label": "",
        "tool_name": "quality-check",
        "tool_input": {},
        "next_tick_at": "2099-01-01T00:00:00Z"
    });
    let (status, _) = post_json(app, "/api/orchestrator/actions", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_action_invalid_tool_name() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let body = serde_json::json!({
        "label": "test",
        "tool_name": "../evil",
        "tool_input": {},
        "next_tick_at": "2099-01-01T00:00:00Z"
    });
    let (status, _) = post_json(app, "/api/orchestrator/actions", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_action_success() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create an action
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (_, json) = post_json(app, "/api/orchestrator/actions", action_body()).await;
    let id = json["id"].as_str().unwrap().to_string();

    // Delete it
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _) = delete_req(app, &format!("/api/orchestrator/actions/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // List should be empty
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = get(app, "/api/orchestrator/actions").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn delete_action_idempotent() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    // Delete with a random UUID that doesn't exist — must return 204
    let (status, _) = delete_req(
        app,
        "/api/orchestrator/actions/00000000-0000-0000-0000-000000000001",
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_action_invalid_uuid() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _) = delete_req(app, "/api/orchestrator/actions/not-a-uuid").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn patch_action_label() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (_, json) = post_json(app, "/api/orchestrator/actions", action_body()).await;
    let id = json["id"].as_str().unwrap().to_string();

    // Patch label
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = patch_json(
        app,
        &format!("/api/orchestrator/actions/{id}"),
        serde_json::json!({ "label": "renamed-audit" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["label"], "renamed-audit");
}

#[tokio::test]
async fn patch_action_recurrence_set() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (_, json) = post_json(app, "/api/orchestrator/actions", action_body()).await;
    let id = json["id"].as_str().unwrap().to_string();

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = patch_json(
        app,
        &format!("/api/orchestrator/actions/{id}"),
        serde_json::json!({ "recurrence_secs": 3600 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["recurrence_secs"], 3600);
}

#[tokio::test]
async fn patch_action_recurrence_clear() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create with recurrence
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let mut body = action_body();
    body["recurrence_secs"] = serde_json::json!(86400);
    let (_, json) = post_json(app, "/api/orchestrator/actions", body).await;
    let id = json["id"].as_str().unwrap().to_string();

    // Clear recurrence
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, json) = patch_json(
        app,
        &format!("/api/orchestrator/actions/{id}"),
        serde_json::json!({ "recurrence_secs": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(json["recurrence_secs"].is_null());
}

#[tokio::test]
async fn patch_action_not_found() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _) = patch_json(
        app,
        "/api/orchestrator/actions/00000000-0000-0000-0000-000000000099",
        serde_json::json!({ "label": "new-label" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_action_empty_label() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (_, json) = post_json(app, "/api/orchestrator/actions", action_body()).await;
    let id = json["id"].as_str().unwrap().to_string();

    let app = sdlc_server::build_router(dir.path().to_path_buf(), 0);
    let (status, _) = patch_json(
        app,
        &format!("/api/orchestrator/actions/{id}"),
        serde_json::json!({ "label": "" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
