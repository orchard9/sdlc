use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// List tokens
// ---------------------------------------------------------------------------

/// GET /api/auth/tokens — list all named tokens (name + created_at only, no values).
pub async fn list_tokens(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::auth_config::load(&root)?;
        let list: Vec<serde_json::Value> = config
            .tokens
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "created_at": t.created_at,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Create token
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CreateTokenBody {
    pub name: String,
}

/// POST /api/auth/tokens — generate and persist a new named token.
///
/// Returns 201 with `{ name, token, created_at }` on success.
/// The `token` field is only returned here; subsequent list calls omit it.
/// Returns 409 if a token with that name already exists.
pub async fn create_token(
    State(app): State<AppState>,
    Json(body): Json<CreateTokenBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let root = app.root.clone();
    let name = body.name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let token = sdlc_core::auth_config::add_token(&root, &name)?;
        let config = sdlc_core::auth_config::load(&root)?;
        let entry = config
            .tokens
            .iter()
            .find(|t| t.name == name)
            .expect("token was just added");
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "name": entry.name,
            "token": token,
            "created_at": entry.created_at,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok((StatusCode::CREATED, Json(result)))
}

// ---------------------------------------------------------------------------
// Delete token
// ---------------------------------------------------------------------------

/// DELETE /api/auth/tokens/:name — remove a named token.
///
/// Returns 200 `{ "status": "removed" }` on success.
/// Returns 404 if no token with that name exists.
pub async fn delete_token(
    State(app): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    tokio::task::spawn_blocking(move || {
        sdlc_core::auth_config::remove_token(&root, &name)?;
        Ok::<_, sdlc_core::SdlcError>(())
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn list_tokens_returns_empty_when_none() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = list_tokens(State(app)).await.unwrap();
        let arr = result.0.as_array().unwrap();
        assert!(arr.is_empty());
    }

    #[tokio::test]
    async fn create_and_list_token() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());

        let body = CreateTokenBody {
            name: "alice".to_string(),
        };
        let (status, result) = create_token(State(app.clone()), Json(body)).await.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(result.0["name"], "alice");
        assert_eq!(result.0["token"].as_str().unwrap().len(), 8);
        assert!(result.0["created_at"].is_string());

        let list_result = list_tokens(State(app)).await.unwrap();
        let arr = list_result.0.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "alice");
        // token value must NOT appear in list response
        assert!(arr[0].get("token").is_none());
    }

    #[tokio::test]
    async fn create_token_duplicate_returns_409() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());

        let body = CreateTokenBody {
            name: "bob".to_string(),
        };
        let _ = create_token(State(app.clone()), Json(body)).await.unwrap();

        let body2 = CreateTokenBody {
            name: "bob".to_string(),
        };
        let err = create_token(State(app), Json(body2)).await.unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn delete_token_removes_entry() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());

        let body = CreateTokenBody {
            name: "carol".to_string(),
        };
        let _ = create_token(State(app.clone()), Json(body)).await.unwrap();

        let result = delete_token(State(app.clone()), Path("carol".to_string()))
            .await
            .unwrap();
        assert_eq!(result.0["status"], "removed");

        let list_result = list_tokens(State(app)).await.unwrap();
        let arr = list_result.0.as_array().unwrap();
        assert!(arr.is_empty());
    }

    #[tokio::test]
    async fn delete_missing_token_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let err = delete_token(State(app), Path("ghost".to_string()))
            .await
            .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::NOT_FOUND);
    }
}
