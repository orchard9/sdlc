use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

/// GET /api/feedback -- list all pending feedback notes
pub async fn list_notes(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let notes = sdlc_core::feedback::list(&root)?;
        let list: Vec<serde_json::Value> = notes.iter().map(note_to_json).collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Add
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct AddBody {
    pub content: String,
}

/// POST /api/feedback -- add a new feedback note
pub async fn add_note(
    State(app): State<AppState>,
    Json(body): Json<AddBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let note = sdlc_core::feedback::add(&root, &body.content)?;
        Ok::<_, sdlc_core::SdlcError>(note_to_json(&note))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

/// DELETE /api/feedback/:id -- delete a feedback note by ID
pub async fn delete_note(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let deleted = sdlc_core::feedback::delete(&root, &id_clone)?;
        Ok::<_, sdlc_core::SdlcError>(deleted)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    if !result {
        return Err(AppError::not_found(format!(
            "feedback note '{id}' not found"
        )));
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------------------------
// Update (feedback-edit)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct UpdateBody {
    pub content: String,
}

/// PATCH /api/feedback/:id -- update a feedback note's content
pub async fn update_note(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.content.trim().is_empty() {
        return Err(AppError::bad_request("content cannot be empty"));
    }
    let root = app.root.clone();
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::feedback::update(&root, &id_clone, &body.content)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    match result {
        Some(note) => Ok(Json(note_to_json(&note))),
        None => Err(AppError::not_found(format!(
            "feedback note '{id}' not found"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Enrich (feedback-enrich)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct EnrichBody {
    pub content: String,
    pub source: String,
}

/// POST /api/feedback/:id/enrich -- append an enrichment block to a note
pub async fn enrich_note(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<EnrichBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let note = sdlc_core::feedback::enrich(&root, &id_clone, &body.source, &body.content)?;
        Ok::<_, sdlc_core::SdlcError>(note_to_json(&note))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Submit to Ponder
// ---------------------------------------------------------------------------

/// POST /api/feedback/to-ponder -- bundle all notes into a new ponder entry.
pub async fn to_ponder(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let notes = sdlc_core::feedback::list(&root)?;
        if notes.is_empty() {
            return Err(sdlc_core::SdlcError::InvalidSlug(
                "no feedback notes to submit".to_string(),
            ));
        }
        let base = chrono::Utc::now().format("feedback-%Y%m%d").to_string();
        let slug = unique_ponder_slug(&root, &base);
        let title = format!(
            "Feedback \u{2014} {}",
            chrono::Utc::now().format("%B %d, %Y")
        );
        sdlc_core::ponder::PonderEntry::create(&root, &slug, &title)?;
        let md = sdlc_core::feedback::to_markdown(&notes);
        sdlc_core::ponder::capture_content(&root, &slug, "notes.md", &md)?;
        sdlc_core::feedback::clear(&root)?;
        Ok::<_, sdlc_core::SdlcError>(
            serde_json::json!({ "slug": slug, "note_count": notes.len() }),
        )
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn note_to_json(n: &sdlc_core::feedback::FeedbackNote) -> serde_json::Value {
    let enrichments: Vec<serde_json::Value> = n
        .enrichments
        .iter()
        .map(|e| {
            serde_json::json!({
                "source": e.source,
                "content": e.content,
                "added_at": e.added_at,
            })
        })
        .collect();
    serde_json::json!({
        "id": n.id,
        "content": n.content,
        "created_at": n.created_at,
        "updated_at": n.updated_at,
        "enrichments": enrichments,
    })
}

fn unique_ponder_slug(root: &std::path::Path, base: &str) -> String {
    let first = base.to_string();
    if !sdlc_core::paths::ponder_dir(root, &first).exists() {
        return first;
    }
    let mut n = 2u32;
    loop {
        let candidate = format!("{base}-{n}");
        if !sdlc_core::paths::ponder_dir(root, &candidate).exists() {
            return candidate;
        }
        n += 1;
    }
}

// ---------------------------------------------------------------------------
// Slack feedback ingestion
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct SlackContextMessage {
    pub user_name: String,
    pub text: String,
    #[allow(dead_code)]
    pub ts: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SlackFeedbackPayload {
    pub source: String,
    #[allow(dead_code)]
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
    #[allow(dead_code)]
    pub user_id: Option<String>,
    pub user_name: String,
    pub text: String,
    pub message_ts: Option<String>,
    #[allow(dead_code)]
    pub thread_ts: Option<String>,
    pub context_messages: Option<Vec<SlackContextMessage>>,
}

/// Render context messages and the trigger message into a markdown body.
fn render_slack_context_markdown(
    context_messages: Option<&[SlackContextMessage]>,
    trigger_user: &str,
    trigger_text: &str,
    message_ts: Option<&str>,
) -> String {
    let mut md = String::new();

    // Dedup marker (HTML comment, invisible in rendered markdown)
    if let Some(ts) = message_ts {
        md.push_str(&format!("<!-- slack:message_ts={ts} -->\n\n"));
    }

    if let Some(msgs) = context_messages {
        if !msgs.is_empty() {
            md.push_str("## Conversation Context\n\n");
            for msg in msgs {
                md.push_str(&format!("**{}**: {}\n\n", msg.user_name, msg.text));
            }
            md.push_str("---\n\n");
        }
    }

    md.push_str(&format!("**{trigger_user}**: {trigger_text}"));
    md
}

/// POST /api/feedback/slack — receive a normalized Slack payload and create a
/// feedback thread with conversation context.
pub async fn receive_slack_feedback(
    State(app): State<AppState>,
    Json(body): Json<SlackFeedbackPayload>,
) -> Result<axum::response::Response, AppError> {
    // --- Validation ---
    if body.source != "slack" {
        return Err(AppError::bad_request("source must be 'slack'"));
    }
    if body.text.trim().is_empty() {
        return Err(AppError::bad_request("text is required"));
    }
    if body.user_name.trim().is_empty() {
        return Err(AppError::bad_request("user_name is required"));
    }

    let root = app.root.clone();
    let message_ts = body.message_ts.clone();
    let channel_name = body.channel_name.clone();
    let user_name = body.user_name.clone();
    let text = body.text.clone();

    let result = tokio::task::spawn_blocking(
        move || -> Result<Result<serde_json::Value, serde_json::Value>, anyhow::Error> {
            // --- Dedup check ---
            if let Some(ref ts) = message_ts {
                let marker = format!("<!-- slack:message_ts={ts} -->");
                let threads = sdlc_core::feedback_thread::list_threads(&root, None)?;
                for thread in &threads {
                    if let Some(ref thread_body) = thread.body {
                        if thread_body.contains(&marker) {
                            return Ok::<_, anyhow::Error>(Err(serde_json::json!({
                                "error": "duplicate",
                                "existing_thread_id": thread.id,
                            })));
                        }
                    }
                }
            }

            // --- Build context ---
            let context = match channel_name.as_deref() {
                Some(ch) if !ch.is_empty() => format!("slack:{ch}"),
                _ => "slack:dm".to_string(),
            };

            let title = {
                let suffix = " (via Slack)";
                let max_text = 120 - suffix.len();
                if text.len() <= max_text {
                    format!("{text}{suffix}")
                } else {
                    let truncated: String = text.chars().take(max_text).collect();
                    format!("{truncated}{suffix}")
                }
            };

            let body_md = render_slack_context_markdown(
                body.context_messages.as_deref(),
                &user_name,
                &text,
                message_ts.as_deref(),
            );

            let thread =
                sdlc_core::feedback_thread::create_thread(&root, &context, &title, Some(&body_md))?;

            // Add the trigger message as the first post
            sdlc_core::feedback_thread::add_post(&root, &thread.id, &user_name, &text)?;

            // Reload to get updated post_count
            let thread = sdlc_core::feedback_thread::load_thread(&root, &thread.id)?;

            Ok(Ok(thread_to_json_ext(&thread)))
        },
    )
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    match result {
        Ok(json) => Ok((StatusCode::CREATED, Json(json)).into_response()),
        Err(conflict_json) => Ok((StatusCode::CONFLICT, Json(conflict_json)).into_response()),
    }
}

/// Like `note_to_json` but for feedback threads (used by Slack endpoint).
fn thread_to_json_ext(t: &sdlc_core::feedback_thread::FeedbackThread) -> serde_json::Value {
    serde_json::json!({
        "id": t.id,
        "context": t.context,
        "title": t.title,
        "body": t.body,
        "status": t.status,
        "comment_count": t.post_count,
        "created_at": t.created_at,
        "updated_at": t.updated_at,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;

    #[tokio::test]
    async fn list_empty_initially() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = list_notes(State(app)).await.unwrap();
        assert!(result.0.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn add_and_list_note() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = AddBody {
            content: "This is a test note".to_string(),
        };
        let _ = add_note(State(app.clone()), Json(body)).await.unwrap();
        let result = list_notes(State(app)).await.unwrap();
        let arr = result.0.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["content"], "This is a test note");
    }

    #[tokio::test]
    async fn delete_missing_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let err = delete_note(State(app), Path("F99".to_string()))
            .await
            .unwrap_err();
        use axum::response::IntoResponse;
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn update_existing_note_returns_200() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = AddBody {
            content: "Original".to_string(),
        };
        let _ = add_note(State(app.clone()), Json(body)).await.unwrap();
        let update_body = UpdateBody {
            content: "Updated".to_string(),
        };
        let result = update_note(
            State(app.clone()),
            Path("F1".to_string()),
            Json(update_body),
        )
        .await
        .unwrap();
        assert_eq!(result.0["content"], "Updated");
        assert!(!result.0["updated_at"].is_null());
    }

    #[tokio::test]
    async fn update_missing_note_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = UpdateBody {
            content: "Whatever".to_string(),
        };
        let err = update_note(State(app), Path("F99".to_string()), Json(body))
            .await
            .unwrap_err();
        use axum::response::IntoResponse;
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn update_with_empty_content_returns_400() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = UpdateBody {
            content: "".to_string(),
        };
        let err = update_note(State(app), Path("F1".to_string()), Json(body))
            .await
            .unwrap_err();
        use axum::response::IntoResponse;
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn to_ponder_empty_returns_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        assert!(to_ponder(State(app)).await.is_err());
    }

    #[tokio::test]
    async fn enrich_note_returns_updated_note() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = AddBody {
            content: "Original note".to_string(),
        };
        let _ = add_note(State(app.clone()), Json(body)).await.unwrap();
        let enrich_body = EnrichBody {
            content: "Some context".to_string(),
            source: "user".to_string(),
        };
        let result = enrich_note(
            State(app.clone()),
            Path("F1".to_string()),
            Json(enrich_body),
        )
        .await
        .unwrap();
        let enrichments = result.0["enrichments"].as_array().unwrap();
        assert_eq!(enrichments.len(), 1);
        assert_eq!(enrichments[0]["source"], "user");
        assert_eq!(enrichments[0]["content"], "Some context");
    }

    #[tokio::test]
    async fn enrich_note_missing_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let enrich_body = EnrichBody {
            content: "context".to_string(),
            source: "user".to_string(),
        };
        let err = enrich_note(State(app), Path("F99".to_string()), Json(enrich_body))
            .await
            .unwrap_err();
        use axum::response::IntoResponse;
        assert_eq!(
            err.into_response().status(),
            axum::http::StatusCode::NOT_FOUND
        );
    }

    // -----------------------------------------------------------------------
    // Slack feedback endpoint tests
    // -----------------------------------------------------------------------

    fn make_slack_app() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        (dir, app)
    }

    fn slack_payload(
        text: &str,
        user_name: &str,
        source: &str,
        message_ts: Option<&str>,
        context_messages: Option<Vec<SlackContextMessage>>,
    ) -> SlackFeedbackPayload {
        SlackFeedbackPayload {
            source: source.to_string(),
            channel_id: Some("C123".to_string()),
            channel_name: Some("sdlc-bugs".to_string()),
            user_id: Some("U123".to_string()),
            user_name: user_name.to_string(),
            text: text.to_string(),
            message_ts: message_ts.map(|s| s.to_string()),
            thread_ts: None,
            context_messages,
        }
    }

    #[tokio::test]
    async fn slack_valid_payload_creates_thread() {
        let (_dir, app) = make_slack_app();
        let body = slack_payload("login broken on mobile", "jordan", "slack", None, None);
        let resp = receive_slack_feedback(State(app), Json(body))
            .await
            .unwrap()
            .into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let bytes = axum::body::to_bytes(resp.into_body(), 10_000)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(!json["id"].as_str().unwrap().is_empty());
        assert!(json["context"].as_str().unwrap().starts_with("slack:"));
        assert!(json["title"]
            .as_str()
            .unwrap()
            .contains("login broken on mobile"));
        assert!(json["comment_count"].as_u64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn slack_missing_text_returns_400() {
        let (_dir, app) = make_slack_app();
        let body = slack_payload("", "jordan", "slack", None, None);
        let err = receive_slack_feedback(State(app), Json(body))
            .await
            .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn slack_missing_user_name_returns_400() {
        let (_dir, app) = make_slack_app();
        let body = slack_payload("some text", "", "slack", None, None);
        let err = receive_slack_feedback(State(app), Json(body))
            .await
            .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn slack_wrong_source_returns_400() {
        let (_dir, app) = make_slack_app();
        let body = slack_payload("some text", "jordan", "telegram", None, None);
        let err = receive_slack_feedback(State(app), Json(body))
            .await
            .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn slack_duplicate_message_ts_returns_409() {
        let (_dir, app) = make_slack_app();
        let ts = "1234567890.123456";
        let body1 = slack_payload("first msg", "jordan", "slack", Some(ts), None);
        let resp1 = receive_slack_feedback(State(app.clone()), Json(body1))
            .await
            .unwrap()
            .into_response();
        assert_eq!(resp1.status(), StatusCode::CREATED);

        let body2 = slack_payload("second msg", "alice", "slack", Some(ts), None);
        let resp2 = receive_slack_feedback(State(app), Json(body2))
            .await
            .unwrap()
            .into_response();
        assert_eq!(resp2.status(), StatusCode::CONFLICT);
        let bytes = axum::body::to_bytes(resp2.into_body(), 10_000)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "duplicate");
        assert!(!json["existing_thread_id"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn slack_no_context_messages_works() {
        let (_dir, app) = make_slack_app();
        let body = slack_payload("simple msg", "jordan", "slack", None, None);
        let resp = receive_slack_feedback(State(app), Json(body))
            .await
            .unwrap()
            .into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let bytes = axum::body::to_bytes(resp.into_body(), 10_000)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let body_str = json["body"].as_str().unwrap();
        assert!(!body_str.contains("Conversation Context"));
        assert!(body_str.contains("**jordan**: simple msg"));
    }

    #[tokio::test]
    async fn slack_with_context_messages_renders_markdown() {
        let (_dir, app) = make_slack_app();
        let ctx = vec![
            SlackContextMessage {
                user_name: "alice".to_string(),
                text: "I saw the same bug".to_string(),
                ts: Some("111.222".to_string()),
            },
            SlackContextMessage {
                user_name: "bob".to_string(),
                text: "me too".to_string(),
                ts: None,
            },
        ];
        let body = slack_payload("login broken", "jordan", "slack", None, Some(ctx));
        let resp = receive_slack_feedback(State(app), Json(body))
            .await
            .unwrap()
            .into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let bytes = axum::body::to_bytes(resp.into_body(), 10_000)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let body_str = json["body"].as_str().unwrap();
        assert!(body_str.contains("## Conversation Context"));
        assert!(body_str.contains("**alice**: I saw the same bug"));
        assert!(body_str.contains("**bob**: me too"));
        assert!(body_str.contains("**jordan**: login broken"));
    }

    #[tokio::test]
    async fn slack_no_message_ts_skips_dedup() {
        let (_dir, app) = make_slack_app();
        let body1 = slack_payload("msg one", "jordan", "slack", None, None);
        let resp1 = receive_slack_feedback(State(app.clone()), Json(body1))
            .await
            .unwrap()
            .into_response();
        assert_eq!(resp1.status(), StatusCode::CREATED);

        let body2 = slack_payload("msg two", "jordan", "slack", None, None);
        let resp2 = receive_slack_feedback(State(app), Json(body2))
            .await
            .unwrap()
            .into_response();
        assert_eq!(resp2.status(), StatusCode::CREATED);

        // Both created — different IDs
        let b1 = axum::body::to_bytes(resp1.into_body(), 10_000)
            .await
            .unwrap();
        let b2 = axum::body::to_bytes(resp2.into_body(), 10_000)
            .await
            .unwrap();
        let j1: serde_json::Value = serde_json::from_slice(&b1).unwrap();
        let j2: serde_json::Value = serde_json::from_slice(&b2).unwrap();
        assert_ne!(j1["id"], j2["id"]);
    }
}
