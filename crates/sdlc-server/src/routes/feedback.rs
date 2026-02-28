use axum::{
    extract::{Path, State},
    Json,
};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

/// GET /api/feedback — list all pending feedback notes
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

/// POST /api/feedback — add a new feedback note
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

/// DELETE /api/feedback/:id — delete a feedback note by ID
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
// Submit to Ponder
// ---------------------------------------------------------------------------

/// POST /api/feedback/to-ponder — bundle all notes into a new ponder entry.
/// Returns `{ slug }` so the caller can navigate to the ponder workspace.
pub async fn to_ponder(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let notes = sdlc_core::feedback::list(&root)?;
        if notes.is_empty() {
            return Err(sdlc_core::SdlcError::InvalidSlug(
                "no feedback notes to submit".to_string(),
            ));
        }

        // Generate a unique slug based on the current timestamp.
        let base = chrono::Utc::now().format("feedback-%Y%m%d").to_string();
        let slug = unique_ponder_slug(&root, &base);

        // Create the ponder entry.
        let title = format!("Feedback — {}", chrono::Utc::now().format("%B %d, %Y"));
        sdlc_core::ponder::PonderEntry::create(&root, &slug, &title)?;

        // Capture the notes as a single markdown artifact.
        let md = sdlc_core::feedback::to_markdown(&notes);
        sdlc_core::ponder::capture_content(&root, &slug, "notes.md", &md)?;

        // Clear feedback now that it's been submitted.
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
    serde_json::json!({
        "id": n.id,
        "content": n.content,
        "created_at": n.created_at,
    })
}

/// Find the first unused ponder slug of the form `<base>`, `<base>-2`, etc.
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
    async fn to_ponder_empty_returns_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        assert!(to_ponder(State(app)).await.is_err());
    }
}
