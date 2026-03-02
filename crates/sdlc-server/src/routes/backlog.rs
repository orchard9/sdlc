use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sdlc_core::backlog::{BacklogKind, BacklogStatus, BacklogStore};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Parameter types
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, Default)]
pub struct ListBacklogQuery {
    pub status: Option<String>,
    pub source_feature: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/backlog
// ---------------------------------------------------------------------------

pub async fn list_backlog(
    State(app): State<AppState>,
    Query(params): Query<ListBacklogQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let status_filter = params
            .status
            .as_deref()
            .map(parse_status)
            .transpose()
            .map_err(sdlc_core::SdlcError::InvalidSlug)?;

        let items = BacklogStore::list(&root, status_filter, params.source_feature.as_deref())?;
        let list: Vec<serde_json::Value> = items
            .iter()
            .map(|item| serde_json::to_value(item).unwrap_or(serde_json::Value::Null))
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/backlog
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CreateBacklogBody {
    pub title: String,
    pub kind: BacklogKind,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub evidence: Option<String>,
    #[serde(default)]
    pub source_feature: Option<String>,
}

pub async fn create_backlog_item(
    State(app): State<AppState>,
    Json(body): Json<CreateBacklogBody>,
) -> Result<impl IntoResponse, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let item = BacklogStore::add(
            &root,
            body.title,
            body.kind,
            body.description,
            body.evidence,
            body.source_feature,
        )?;
        Ok::<_, sdlc_core::SdlcError>(
            serde_json::to_value(&item).unwrap_or(serde_json::Value::Null),
        )
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok((StatusCode::CREATED, Json(result)))
}

// ---------------------------------------------------------------------------
// POST /api/backlog/:id/park
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct ParkBacklogBody {
    pub park_reason: String,
}

pub async fn park_backlog_item(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ParkBacklogBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let item = BacklogStore::park(&root, &id, body.park_reason)?;
        Ok::<_, sdlc_core::SdlcError>(
            serde_json::to_value(&item).unwrap_or(serde_json::Value::Null),
        )
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/backlog/:id/promote
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct PromoteBacklogBody {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// POST /api/backlog/:id/promote
///
/// 1. Creates a new draft feature using the provided slug + title.
/// 2. Marks the backlog item as promoted (records promoted_to = slug).
///
/// Returns `{ "promoted_to": "<slug>" }` on success.
/// Returns 409 if a feature with the given slug already exists.
/// Returns 404 if the backlog item id does not exist.
pub async fn promote_backlog_item(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<PromoteBacklogBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        // Step 1: create the feature (returns FeatureExists → 409 if slug taken)
        sdlc_core::feature::Feature::create_with_description(
            &root,
            body.slug.clone(),
            body.title,
            body.description,
        )?;

        // Step 2: mark the backlog item promoted
        BacklogStore::mark_promoted(&root, &id, &body.slug)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({ "promoted_to": body.slug }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn parse_status(s: &str) -> Result<BacklogStatus, String> {
    match s {
        "open" => Ok(BacklogStatus::Open),
        "parked" => Ok(BacklogStatus::Parked),
        "promoted" => Ok(BacklogStatus::Promoted),
        other => Err(format!("unknown backlog status: '{other}'")),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    fn make_app(dir: &tempfile::TempDir) -> AppState {
        // Initialize .sdlc directory so Feature::create works
        let sdlc = dir.path().join(".sdlc");
        std::fs::create_dir_all(&sdlc).unwrap();
        AppState::new(dir.path().to_path_buf())
    }

    // -----------------------------------------------------------------------
    // list_backlog
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn list_empty_when_no_backlog_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);
        let result = list_backlog(State(app), Query(ListBacklogQuery::default()))
            .await
            .unwrap();
        let arr = result.0.as_array().unwrap();
        assert!(arr.is_empty());
    }

    #[tokio::test]
    async fn list_returns_open_items() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "auth.rs: token race".to_string(),
            sdlc_core::backlog::BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();

        let result = list_backlog(State(app), Query(ListBacklogQuery::default()))
            .await
            .unwrap();
        let arr = result.0.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "B1");
        assert_eq!(arr[0]["kind"], "concern");
        assert_eq!(arr[0]["status"], "open");
    }

    #[tokio::test]
    async fn list_status_filter_open() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "open item".to_string(),
            sdlc_core::backlog::BacklogKind::Idea,
            None,
            None,
            None,
        )
        .unwrap();
        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "to park".to_string(),
            sdlc_core::backlog::BacklogKind::Debt,
            None,
            None,
            None,
        )
        .unwrap();
        sdlc_core::backlog::BacklogStore::park(dir.path(), "B2", "low priority".to_string())
            .unwrap();

        let result = list_backlog(
            State(app),
            Query(ListBacklogQuery {
                status: Some("open".to_string()),
                source_feature: None,
            }),
        )
        .await
        .unwrap();
        let arr = result.0.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "B1");
    }

    #[tokio::test]
    async fn list_source_feature_filter() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "A".to_string(),
            sdlc_core::backlog::BacklogKind::Concern,
            None,
            None,
            Some("feature-x".to_string()),
        )
        .unwrap();
        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "B".to_string(),
            sdlc_core::backlog::BacklogKind::Idea,
            None,
            None,
            Some("feature-y".to_string()),
        )
        .unwrap();

        let result = list_backlog(
            State(app),
            Query(ListBacklogQuery {
                status: None,
                source_feature: Some("feature-x".to_string()),
            }),
        )
        .await
        .unwrap();
        let arr = result.0.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["source_feature"], "feature-x");
    }

    // -----------------------------------------------------------------------
    // park_backlog_item
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn park_item_updates_status() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "concern A".to_string(),
            sdlc_core::backlog::BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();

        let result = park_backlog_item(
            State(app),
            Path("B1".to_string()),
            Json(ParkBacklogBody {
                park_reason: "revisit after v2".to_string(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(result.0["status"], "parked");
        assert_eq!(result.0["park_reason"], "revisit after v2");
    }

    #[tokio::test]
    async fn park_empty_reason_returns_422() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "concern A".to_string(),
            sdlc_core::backlog::BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();

        let err = park_backlog_item(
            State(app),
            Path("B1".to_string()),
            Json(ParkBacklogBody {
                park_reason: "".to_string(),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(
            err.into_response().status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[tokio::test]
    async fn park_whitespace_reason_returns_422() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "concern A".to_string(),
            sdlc_core::backlog::BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();

        let err = park_backlog_item(
            State(app),
            Path("B1".to_string()),
            Json(ParkBacklogBody {
                park_reason: "   ".to_string(),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(
            err.into_response().status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[tokio::test]
    async fn park_missing_item_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        let err = park_backlog_item(
            State(app),
            Path("B99".to_string()),
            Json(ParkBacklogBody {
                park_reason: "reason".to_string(),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::NOT_FOUND);
    }

    // -----------------------------------------------------------------------
    // promote_backlog_item
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn promote_item_creates_feature_and_marks_promoted() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "auth race concern".to_string(),
            sdlc_core::backlog::BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();

        let result = promote_backlog_item(
            State(app),
            Path("B1".to_string()),
            Json(PromoteBacklogBody {
                slug: "auth-race-fix".to_string(),
                title: "Fix auth race condition".to_string(),
                description: None,
            }),
        )
        .await
        .unwrap();

        // Result contains promoted_to slug
        assert_eq!(result.0["promoted_to"], "auth-race-fix");

        // Feature directory was created
        let feature_dir = dir
            .path()
            .join(".sdlc")
            .join("features")
            .join("auth-race-fix");
        assert!(
            feature_dir.exists(),
            "feature dir should exist at {:?}",
            feature_dir
        );

        // Backlog item is marked promoted
        let item = sdlc_core::backlog::BacklogStore::get(dir.path(), "B1").unwrap();
        assert_eq!(item.status, sdlc_core::backlog::BacklogStatus::Promoted);
        assert_eq!(item.promoted_to.as_deref(), Some("auth-race-fix"));
    }

    #[tokio::test]
    async fn promote_duplicate_slug_returns_409() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        sdlc_core::backlog::BacklogStore::add(
            dir.path(),
            "some concern".to_string(),
            sdlc_core::backlog::BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();

        // Create the feature first so slug is taken
        sdlc_core::feature::Feature::create(dir.path(), "existing-feature", "Existing").unwrap();

        let err = promote_backlog_item(
            State(app),
            Path("B1".to_string()),
            Json(PromoteBacklogBody {
                slug: "existing-feature".to_string(),
                title: "Existing".to_string(),
                description: None,
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn promote_missing_item_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = make_app(&dir);

        let err = promote_backlog_item(
            State(app),
            Path("B99".to_string()),
            Json(PromoteBacklogBody {
                slug: "my-feature".to_string(),
                title: "My Feature".to_string(),
                description: None,
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::NOT_FOUND);
    }
}
