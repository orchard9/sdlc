use axum::{
    extract::{Path, State},
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
}
