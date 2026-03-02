//! REST routes for feedback threads.
//!
//! GET    /api/threads                  — list threads (optional ?context= filter)
//! POST   /api/threads                  — create thread
//! GET    /api/threads/:id              — get thread + all posts
//! POST   /api/threads/:id/posts        — append a post
//! DELETE /api/threads/:id              — delete thread

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn thread_to_json(t: &sdlc_core::feedback_thread::FeedbackThread) -> serde_json::Value {
    serde_json::json!({
        "id":         t.id,
        "title":      t.title,
        "context":    t.context,
        "created_at": t.created_at,
        "updated_at": t.updated_at,
        "post_count": t.post_count,
    })
}

fn post_to_json(p: &sdlc_core::feedback_thread::ThreadPost) -> serde_json::Value {
    serde_json::json!({
        "seq":        p.seq,
        "author":     p.author,
        "content":    p.content,
        "created_at": p.created_at,
    })
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ListQuery {
    pub context: Option<String>,
}

/// GET /api/threads — list all threads, optionally filtered by context
pub async fn list_threads(
    State(app): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let threads = sdlc_core::feedback_thread::list_threads(&root, params.context.as_deref())?;
        let list: Vec<serde_json::Value> = threads.iter().map(thread_to_json).collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateBody {
    pub context: String,
    pub title: Option<String>,
}

/// POST /api/threads — create a new thread
pub async fn create_thread(
    State(app): State<AppState>,
    Json(body): Json<CreateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.context.trim().is_empty() {
        return Err(AppError::bad_request("context cannot be empty"));
    }
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let title = body.title.as_deref().unwrap_or("");
        let thread = sdlc_core::feedback_thread::create_thread(&root, &body.context, title)?;
        Ok::<_, sdlc_core::SdlcError>(thread_to_json(&thread))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Get (with posts inline)
// ---------------------------------------------------------------------------

/// GET /api/threads/:id — get a thread and all its posts
pub async fn get_thread(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let thread = sdlc_core::feedback_thread::load_thread(&root, &id_clone)?;
        let posts = sdlc_core::feedback_thread::list_posts(&root, &id_clone)?;
        let mut value = thread_to_json(&thread);
        value["posts"] = serde_json::json!(posts.iter().map(post_to_json).collect::<Vec<_>>());
        Ok::<_, sdlc_core::SdlcError>(value)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Add post
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PostBody {
    pub author: String,
    pub content: String,
}

/// POST /api/threads/:id/posts — append a post to a thread
pub async fn add_post(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<PostBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.author.trim().is_empty() {
        return Err(AppError::bad_request("author cannot be empty"));
    }
    if body.content.trim().is_empty() {
        return Err(AppError::bad_request("content cannot be empty"));
    }
    let root = app.root.clone();
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let post =
            sdlc_core::feedback_thread::add_post(&root, &id_clone, &body.author, &body.content)?;
        // Return the updated thread + all posts
        let thread = sdlc_core::feedback_thread::load_thread(&root, &id_clone)?;
        let posts = sdlc_core::feedback_thread::list_posts(&root, &id_clone)?;
        let mut value = thread_to_json(&thread);
        value["posts"] = serde_json::json!(posts.iter().map(post_to_json).collect::<Vec<_>>());
        value["new_post"] = post_to_json(&post);
        Ok::<_, sdlc_core::SdlcError>(value)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

/// DELETE /api/threads/:id — delete a thread and all its posts
pub async fn delete_thread(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let id_clone = id.clone();
    tokio::task::spawn_blocking(move || {
        sdlc_core::feedback_thread::delete_thread(&root, &id_clone)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::response::IntoResponse;

    fn make_app() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        (dir, app)
    }

    #[tokio::test]
    async fn list_empty_initially() {
        let (_dir, app) = make_app();
        let result = list_threads(State(app), Query(ListQuery { context: None }))
            .await
            .unwrap();
        assert!(result.0.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn create_thread_returns_id_and_context() {
        let (_dir, app) = make_app();
        let body = CreateBody {
            context: "feature:my-slug".to_string(),
            title: None,
        };
        let result = create_thread(State(app), Json(body)).await.unwrap();
        assert!(!result.0["id"].as_str().unwrap().is_empty());
        assert_eq!(result.0["context"], "feature:my-slug");
        assert_eq!(result.0["post_count"], 0);
    }

    #[tokio::test]
    async fn create_with_empty_context_returns_400() {
        let (_dir, app) = make_app();
        let body = CreateBody {
            context: "".to_string(),
            title: None,
        };
        let err = create_thread(State(app), Json(body)).await.unwrap_err();
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn get_thread_returns_thread_with_empty_posts() {
        let (_dir, app) = make_app();
        let body = CreateBody {
            context: "feature:x".to_string(),
            title: Some("Test thread".to_string()),
        };
        let created = create_thread(State(app.clone()), Json(body)).await.unwrap();
        let id = created.0["id"].as_str().unwrap().to_string();

        let result = get_thread(State(app), Path(id)).await.unwrap();
        assert_eq!(result.0["context"], "feature:x");
        let posts = result.0["posts"].as_array().unwrap();
        assert!(posts.is_empty());
    }

    #[tokio::test]
    async fn get_thread_not_found_returns_404() {
        let (_dir, app) = make_app();
        let err = get_thread(State(app), Path("no-such-id".to_string()))
            .await
            .unwrap_err();
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn add_post_appends_and_returns_thread_with_posts() {
        let (_dir, app) = make_app();
        let created = create_thread(
            State(app.clone()),
            Json(CreateBody {
                context: "feature:y".to_string(),
                title: None,
            }),
        )
        .await
        .unwrap();
        let id = created.0["id"].as_str().unwrap().to_string();

        let result = add_post(
            State(app),
            Path(id),
            Json(PostBody {
                author: "human".to_string(),
                content: "Hello thread".to_string(),
            }),
        )
        .await
        .unwrap();

        let posts = result.0["posts"].as_array().unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0]["author"], "human");
        assert_eq!(posts[0]["content"], "Hello thread");
        assert_eq!(result.0["post_count"], 1);
    }

    #[tokio::test]
    async fn add_post_empty_author_returns_400() {
        let (_dir, app) = make_app();
        let err = add_post(
            State(app),
            Path("any-id".to_string()),
            Json(PostBody {
                author: "".to_string(),
                content: "text".to_string(),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn add_post_empty_content_returns_400() {
        let (_dir, app) = make_app();
        let err = add_post(
            State(app),
            Path("any-id".to_string()),
            Json(PostBody {
                author: "human".to_string(),
                content: "".to_string(),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn delete_thread_returns_deleted_true() {
        let (_dir, app) = make_app();
        let created = create_thread(
            State(app.clone()),
            Json(CreateBody {
                context: "feature:del".to_string(),
                title: None,
            }),
        )
        .await
        .unwrap();
        let id = created.0["id"].as_str().unwrap().to_string();

        let result = delete_thread(State(app), Path(id)).await.unwrap();
        assert_eq!(result.0["deleted"], true);
    }

    #[tokio::test]
    async fn get_deleted_thread_returns_404() {
        let (_dir, app) = make_app();
        let created = create_thread(
            State(app.clone()),
            Json(CreateBody {
                context: "feature:gone".to_string(),
                title: None,
            }),
        )
        .await
        .unwrap();
        let id = created.0["id"].as_str().unwrap().to_string();

        let _ = delete_thread(State(app.clone()), Path(id.clone()))
            .await
            .unwrap();

        let err = get_thread(State(app), Path(id)).await.unwrap_err();
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn list_with_context_filter() {
        let (_dir, app) = make_app();
        let _ = create_thread(
            State(app.clone()),
            Json(CreateBody {
                context: "feature:a".to_string(),
                title: None,
            }),
        )
        .await
        .unwrap();
        let _ = create_thread(
            State(app.clone()),
            Json(CreateBody {
                context: "feature:b".to_string(),
                title: None,
            }),
        )
        .await
        .unwrap();

        let result = list_threads(
            State(app),
            Query(ListQuery {
                context: Some("feature:a".to_string()),
            }),
        )
        .await
        .unwrap();
        let arr = result.0.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["context"], "feature:a");
    }
}
