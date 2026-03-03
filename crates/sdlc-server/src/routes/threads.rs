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
        // Server-native fields
        "id":            t.id,
        "context":       t.context,
        "body":          t.body,
        // Frontend-compat aliases
        "slug":          t.id,
        "title":         t.title,
        "status":        t.status,
        "author":        "",
        "promoted_to":   t.promoted_to,
        "comment_count": t.post_count,
        "created_at":    t.created_at,
        "updated_at":    t.updated_at,
    })
}

fn post_to_comment_json(p: &sdlc_core::feedback_thread::ThreadPost) -> serde_json::Value {
    serde_json::json!({
        "id":           p.seq.to_string(),
        "seq":          p.seq,
        "author":       p.author,
        "body":         p.content,
        "content":      p.content,
        "incorporated": false,
        "created_at":   p.created_at,
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
    /// Namespaced anchor string, e.g. "feature:my-slug". Defaults to "general".
    pub context: Option<String>,
    pub title: Option<String>,
    /// The "core element" — a living summary of the thread's subject.
    pub body: Option<String>,
}

/// POST /api/threads — create a new thread
pub async fn create_thread(
    State(app): State<AppState>,
    Json(body): Json<CreateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let context = body
        .context
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("general")
        .to_string();
    let root = app.root.clone();
    let body_str = body
        .body
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let result = tokio::task::spawn_blocking(move || {
        let title = body.title.as_deref().unwrap_or("");
        let thread =
            sdlc_core::feedback_thread::create_thread(&root, &context, title, body_str.as_deref())?;
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
        // Frontend expects `comments` (with body/incorporated shape) and `body_version`
        value["comments"] =
            serde_json::json!(posts.iter().map(post_to_comment_json).collect::<Vec<_>>());
        value["body_version"] = serde_json::json!(0u32);
        Ok::<_, sdlc_core::SdlcError>(value)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Add post / comment
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PostBody {
    pub author: String,
    pub content: String,
}

/// POST /api/threads/:id/posts — append a post (raw server format)
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
    append_post(app, id, &body.author, &body.content).await
}

#[derive(Deserialize)]
pub struct CommentBody {
    /// Who is posting. Defaults to "human" when absent.
    pub author: Option<String>,
    /// Comment text (called "body" in the frontend).
    pub body: String,
}

/// POST /api/threads/:id/comments — append a comment (frontend-compat shape)
///
/// Returns a `ThreadComment`-shaped JSON object the frontend can insert
/// directly into its local state without a full page refresh.
pub async fn add_comment(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CommentBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.body.trim().is_empty() {
        return Err(AppError::bad_request("body cannot be empty"));
    }
    let author = body
        .author
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("human")
        .to_string();
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let post = sdlc_core::feedback_thread::add_post(&root, &id, &author, body.body.trim())?;
        Ok::<_, sdlc_core::SdlcError>(post_to_comment_json(&post))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

/// Shared helper: append a post and return the full thread + all posts.
async fn append_post(
    app: AppState,
    id: String,
    author: &str,
    content: &str,
) -> Result<Json<serde_json::Value>, AppError> {
    let author = author.to_string();
    let content = content.to_string();
    let root = app.root.clone();
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let post = sdlc_core::feedback_thread::add_post(&root, &id_clone, &author, &content)?;
        let thread = sdlc_core::feedback_thread::load_thread(&root, &id_clone)?;
        let posts = sdlc_core::feedback_thread::list_posts(&root, &id_clone)?;
        let mut value = thread_to_json(&thread);
        value["comments"] =
            serde_json::json!(posts.iter().map(post_to_comment_json).collect::<Vec<_>>());
        value["new_comment"] = post_to_comment_json(&post);
        Ok::<_, sdlc_core::SdlcError>(value)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Patch (status update)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PatchBody {
    pub status: Option<String>,
}

/// PATCH /api/threads/:id — update mutable thread fields (status)
pub async fn patch_thread(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<PatchBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let status = body.status.clone();
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let thread = sdlc_core::feedback_thread::patch_thread(&root, &id, status.as_deref(), None)?;
        Ok::<_, sdlc_core::SdlcError>(thread_to_json(&thread))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Promote to Ponder
// ---------------------------------------------------------------------------

/// POST /api/threads/:id/promote — create a ponder entry from this thread
pub async fn promote_thread(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let thread = sdlc_core::feedback_thread::load_thread(&root, &id)?;
        // Derive a slug from the thread title: lowercase, replace non-alnum with dash, collapse
        let raw: String = thread
            .title
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_lowercase().next().unwrap()
                } else {
                    '-'
                }
            })
            .collect();
        let mut prev_dash = false;
        let slug_base: String = raw
            .chars()
            .filter(|&c| {
                if c == '-' {
                    if prev_dash {
                        return false;
                    }
                    prev_dash = true;
                } else {
                    prev_dash = false;
                }
                true
            })
            .collect();
        let slug_base = slug_base.trim_matches('-').to_string();
        let slug_base = if slug_base.is_empty() {
            thread.id.clone()
        } else {
            slug_base
        };
        // Truncate to 50 chars
        let slug_base = if slug_base.len() > 50 {
            slug_base[..50].trim_end_matches('-').to_string()
        } else {
            slug_base
        };

        // Resolve collisions
        let ponder_slug = {
            if sdlc_core::ponder::PonderEntry::load(&root, &slug_base).is_err() {
                slug_base.clone()
            } else {
                let mut n = 2u32;
                loop {
                    let candidate = format!("{slug_base}-{n}");
                    if sdlc_core::ponder::PonderEntry::load(&root, &candidate).is_err() {
                        break candidate;
                    }
                    n += 1;
                }
            }
        };

        sdlc_core::ponder::PonderEntry::create(&root, &ponder_slug, &thread.title)?;

        // Update thread status
        sdlc_core::feedback_thread::patch_thread(
            &root,
            &thread.id,
            Some("promoted"),
            Some(Some(&ponder_slug)),
        )?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "ponder_slug": ponder_slug,
            "thread_id": thread.id,
        }))
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
            context: Some("feature:my-slug".to_string()),
            title: None,
            body: None,
        };
        let result = create_thread(State(app), Json(body)).await.unwrap();
        assert!(!result.0["id"].as_str().unwrap().is_empty());
        assert_eq!(result.0["slug"], result.0["id"]); // slug alias
        assert_eq!(result.0["context"], "feature:my-slug");
        assert_eq!(result.0["comment_count"], 0);
    }

    #[tokio::test]
    async fn create_with_no_context_defaults_to_general() {
        let (_dir, app) = make_app();
        let body = CreateBody {
            context: None,
            title: None,
            body: None,
        };
        let result = create_thread(State(app), Json(body)).await.unwrap();
        assert_eq!(result.0["context"], "general");
    }

    #[tokio::test]
    async fn create_with_empty_context_defaults_to_general() {
        let (_dir, app) = make_app();
        let body = CreateBody {
            context: Some("".to_string()),
            title: None,
            body: None,
        };
        let result = create_thread(State(app), Json(body)).await.unwrap();
        assert_eq!(result.0["context"], "general");
    }

    #[tokio::test]
    async fn create_with_body_stores_and_get_returns_it() {
        let (_dir, app) = make_app();
        let body = CreateBody {
            context: Some("feature:body".to_string()),
            title: Some("Body thread".to_string()),
            body: Some("My core element".to_string()),
        };
        let created = create_thread(State(app.clone()), Json(body)).await.unwrap();
        assert_eq!(created.0["body"], "My core element");
        let id = created.0["id"].as_str().unwrap().to_string();

        let result = get_thread(State(app), Path(id)).await.unwrap();
        assert_eq!(result.0["body"], "My core element");
    }

    #[tokio::test]
    async fn get_thread_returns_thread_with_empty_comments() {
        let (_dir, app) = make_app();
        let body = CreateBody {
            context: Some("feature:x".to_string()),
            title: Some("Test thread".to_string()),
            body: None,
        };
        let created = create_thread(State(app.clone()), Json(body)).await.unwrap();
        let id = created.0["id"].as_str().unwrap().to_string();

        let result = get_thread(State(app), Path(id)).await.unwrap();
        assert_eq!(result.0["context"], "feature:x");
        let comments = result.0["comments"].as_array().unwrap();
        assert!(comments.is_empty());
        assert_eq!(result.0["body"], serde_json::Value::Null);
        assert_eq!(result.0["body_version"], 0);
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
    async fn add_post_appends_and_returns_thread_with_comments() {
        let (_dir, app) = make_app();
        let created = create_thread(
            State(app.clone()),
            Json(CreateBody {
                context: Some("feature:y".to_string()),
                title: None,
                body: None,
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

        let comments = result.0["comments"].as_array().unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0]["author"], "human");
        assert_eq!(comments[0]["body"], "Hello thread");
        assert_eq!(result.0["comment_count"], 1);
    }

    #[tokio::test]
    async fn add_comment_returns_comment_shape() {
        let (_dir, app) = make_app();
        let created = create_thread(
            State(app.clone()),
            Json(CreateBody {
                context: Some("feature:z".to_string()),
                title: None,
                body: None,
            }),
        )
        .await
        .unwrap();
        let id = created.0["id"].as_str().unwrap().to_string();

        let result = add_comment(
            State(app),
            Path(id),
            Json(CommentBody {
                author: Some("jordan".to_string()),
                body: "Great idea".to_string(),
            }),
        )
        .await
        .unwrap();

        assert_eq!(result.0["author"], "jordan");
        assert_eq!(result.0["body"], "Great idea");
        assert_eq!(result.0["incorporated"], false);
        assert!(!result.0["id"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn add_comment_defaults_author_to_human() {
        let (_dir, app) = make_app();
        let created = create_thread(
            State(app.clone()),
            Json(CreateBody {
                context: None,
                title: None,
                body: None,
            }),
        )
        .await
        .unwrap();
        let id = created.0["id"].as_str().unwrap().to_string();

        let result = add_comment(
            State(app),
            Path(id),
            Json(CommentBody {
                author: None,
                body: "anon".to_string(),
            }),
        )
        .await
        .unwrap();

        assert_eq!(result.0["author"], "human");
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
                context: Some("feature:del".to_string()),
                title: None,
                body: None,
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
                context: Some("feature:gone".to_string()),
                title: None,
                body: None,
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
                context: Some("feature:a".to_string()),
                title: None,
                body: None,
            }),
        )
        .await
        .unwrap();
        let _ = create_thread(
            State(app.clone()),
            Json(CreateBody {
                context: Some("feature:b".to_string()),
                title: None,
                body: None,
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
