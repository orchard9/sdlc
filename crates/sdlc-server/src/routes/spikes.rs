//! REST routes for spikes.
//!
//! Spikes are time-boxed technical investigations whose findings live at
//! `.sdlc/spikes/<slug>/findings.md` (written by the `/sdlc-spike` agent).
//! These routes expose the existing `sdlc_core::spikes` data layer over HTTP.
//!
//! Endpoints:
//!   GET  /api/spikes              — list all spikes, sorted by date descending
//!   GET  /api/spikes/:slug        — single spike detail + raw findings content
//!   POST /api/spikes/:slug/promote — promote an ADAPT spike to a ponder entry

use axum::extract::{Path, State};
use axum::Json;
use sdlc_core::spikes::SpikeVerdict;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Serialization helpers
// ---------------------------------------------------------------------------

fn verdict_str(v: &Option<SpikeVerdict>) -> serde_json::Value {
    match v {
        Some(SpikeVerdict::Adopt) => serde_json::json!("ADOPT"),
        Some(SpikeVerdict::Adapt) => serde_json::json!("ADAPT"),
        Some(SpikeVerdict::Reject) => serde_json::json!("REJECT"),
        None => serde_json::Value::Null,
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/spikes — list all spikes sorted by date descending.
pub async fn list_spikes(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entries = sdlc_core::spikes::list(&root)?;
        let list: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "slug": e.slug,
                    "title": e.title,
                    "verdict": verdict_str(&e.verdict),
                    "date": e.date,
                    "the_question": e.the_question,
                    "ponder_slug": e.ponder_slug,
                    "knowledge_slug": e.knowledge_slug,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/spikes/:slug — single spike detail plus raw findings.md content.
///
/// Returns 404 if the spike directory does not exist.
pub async fn get_spike(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || match sdlc_core::spikes::load(&root, &slug) {
        Ok((entry, findings)) => Ok(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "verdict": verdict_str(&entry.verdict),
            "date": entry.date,
            "the_question": entry.the_question,
            "ponder_slug": entry.ponder_slug,
            "knowledge_slug": entry.knowledge_slug,
            "findings": findings,
        })),
        Err(sdlc_core::SdlcError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(AppError::not_found(e.to_string()))
        }
        Err(e) => Err(AppError(e.into())),
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Promote request body
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, Default)]
pub struct PromoteBody {
    #[serde(default)]
    pub ponder_slug: Option<String>,
}

/// POST /api/spikes/:slug/promote — promote an ADAPT spike to a ponder entry.
///
/// Only ADAPT spikes may be promoted. Returns 422 for any other verdict.
/// Returns 404 if the spike does not exist.
pub async fn promote_spike(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    body: Option<Json<PromoteBody>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let ponder_slug_override = body.and_then(|b| b.0.ponder_slug);

    let result = tokio::task::spawn_blocking(move || {
        // Load entry to validate verdict before promoting.
        let (entry, _) = match sdlc_core::spikes::load(&root, &slug) {
            Ok(v) => v,
            Err(sdlc_core::SdlcError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(AppError::not_found(e.to_string()));
            }
            Err(e) => return Err(AppError(e.into())),
        };

        if !matches!(entry.verdict, Some(SpikeVerdict::Adapt)) {
            return Err(AppError::unprocessable_json(serde_json::json!({
                "error": "only ADAPT spikes can be promoted to a ponder entry"
            })));
        }

        let ponder_slug =
            sdlc_core::spikes::promote_to_ponder(&root, &slug, ponder_slug_override.as_deref())
                .map_err(|e| AppError(e.into()))?;

        Ok(serde_json::json!({ "ponder_slug": ponder_slug }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt as _;
    use tempfile::TempDir;
    use tower::ServiceExt as _;

    fn make_findings(verdict: &str, has_question: bool) -> String {
        let mut s =
            format!("# Spike: Test Spike\n**Verdict:** {verdict}\n**Date:** 2026-03-04\n\n");
        if has_question {
            s.push_str("## The Question\nCan we test this?\n\n");
        }
        s.push_str("## Risks and Open Questions\n- Open question one\n");
        s
    }

    fn write_spike(tmp: &TempDir, slug: &str, findings: &str) {
        let dir = tmp.path().join(".sdlc/spikes").join(slug);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("findings.md"), findings).unwrap();
    }

    async fn body_json(body: Body) -> serde_json::Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    // -----------------------------------------------------------------------
    // GET /api/spikes
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn list_spikes_empty_when_no_dir() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/spikes")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json, serde_json::json!([]));
    }

    #[tokio::test]
    async fn list_spikes_returns_entry() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        write_spike(&tmp, "my-spike", &make_findings("ADOPT", true));
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/spikes")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["slug"], "my-spike");
        assert_eq!(arr[0]["verdict"], "ADOPT");
        assert_eq!(arr[0]["date"], "2026-03-04");
    }

    // -----------------------------------------------------------------------
    // GET /api/spikes/:slug
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn get_spike_returns_404_for_unknown() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/spikes/no-such-spike")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_spike_returns_findings_content() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        let findings = make_findings("ADAPT", true);
        write_spike(&tmp, "adapt-spike", &findings);
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/spikes/adapt-spike")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["verdict"], "ADAPT");
        assert!(
            json["findings"].as_str().unwrap().contains("Test Spike"),
            "findings should contain raw markdown"
        );
    }

    // -----------------------------------------------------------------------
    // POST /api/spikes/:slug/promote
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn promote_returns_404_for_unknown_spike() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/spikes/ghost/promote")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn promote_returns_422_for_adopt_verdict() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        write_spike(&tmp, "adopt-spike", &make_findings("ADOPT", false));
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/spikes/adopt-spike/promote")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn promote_returns_422_for_reject_verdict() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        write_spike(&tmp, "rej-spike", &make_findings("REJECT", false));
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/spikes/rej-spike/promote")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn promote_adapt_spike_returns_ponder_slug() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        write_spike(&tmp, "adapt-spike", &make_findings("ADAPT", true));
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/spikes/adapt-spike/promote")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["ponder_slug"], "adapt-spike");
    }

    #[tokio::test]
    async fn promote_with_ponder_slug_override() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        write_spike(&tmp, "adapt-spike2", &make_findings("ADAPT", true));
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/spikes/adapt-spike2/promote")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"ponder_slug":"custom-ponder"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["ponder_slug"], "custom-ponder");
    }

    #[tokio::test]
    async fn promote_no_verdict_returns_422() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        // Spike with no verdict (no findings.md header)
        let dir = tmp.path().join(".sdlc/spikes/no-verdict");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("findings.md"),
            "# Spike: No Verdict\n\nSome content.\n",
        )
        .unwrap();
        let app = crate::build_router_for_test(tmp.path().to_path_buf(), None, None);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/spikes/no-verdict/promote")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
