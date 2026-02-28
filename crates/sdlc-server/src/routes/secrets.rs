use axum::{
    extract::{Path, State},
    Json,
};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

/// GET /api/secrets/status — summary of configured keys and env files.
pub async fn get_status(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::secrets::load_config(&root)?;
        let envs = sdlc_core::secrets::list_envs(&root)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "key_count": config.keys.len(),
            "env_count": envs.len(),
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

/// GET /api/secrets/keys — list authorized keys (public side only).
pub async fn list_keys(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let keys = sdlc_core::secrets::list_keys(&root)?;
        let list: Vec<serde_json::Value> = keys
            .iter()
            .map(|k| {
                serde_json::json!({
                    "name": k.name,
                    "type": k.key_type.to_string(),
                    "short_id": k.short_id(),
                    "added_at": k.added_at,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct AddKeyBody {
    pub name: String,
    pub public_key: String,
}

/// POST /api/secrets/keys — add a recipient.
pub async fn add_key(
    State(app): State<AppState>,
    Json(body): Json<AddKeyBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let key_type = sdlc_core::secrets::KeyType::infer(&body.public_key);
        sdlc_core::secrets::add_key(&root, &body.name, key_type, &body.public_key)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({ "status": "added" }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

/// DELETE /api/secrets/keys/:name — remove a recipient.
pub async fn remove_key(
    State(app): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::secrets::remove_key(&root, &name)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({ "status": "removed" }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Envs (metadata only — no decryption server-side)
// ---------------------------------------------------------------------------

/// GET /api/secrets/envs — list env files with key names (no secret values).
pub async fn list_envs(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let envs = sdlc_core::secrets::list_envs(&root)?;
        let list: Vec<serde_json::Value> = envs
            .iter()
            .map(|e| {
                serde_json::json!({
                    "env": e.env,
                    "key_names": e.key_names,
                    "updated_at": e.updated_at,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

/// DELETE /api/secrets/envs/:name — delete an env file and its metadata sidecar.
pub async fn delete_env(
    State(app): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::secrets::delete_env(&root, &name)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({ "status": "deleted" }))
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
    use super::*;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn get_status_returns_zero_counts_when_uninitialized() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = get_status(State(app)).await.unwrap();
        assert_eq!(result.0["key_count"], 0);
        assert_eq!(result.0["env_count"], 0);
    }

    #[tokio::test]
    async fn list_keys_returns_empty_when_no_keys() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = list_keys(State(app)).await.unwrap();
        let arr = result.0.as_array().unwrap();
        assert!(arr.is_empty());
    }

    #[tokio::test]
    async fn add_key_and_list() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());

        let body = AddKeyBody {
            name: "alice".to_string(),
            public_key: "ssh-ed25519 AAAA... alice@example.com".to_string(),
        };
        let _ = add_key(State(app.clone()), Json(body)).await.unwrap();

        let result = list_keys(State(app)).await.unwrap();
        let arr = result.0.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "alice");
        assert_eq!(arr[0]["type"], "ssh");
    }

    #[tokio::test]
    async fn list_envs_returns_empty_when_none() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = list_envs(State(app)).await.unwrap();
        let arr = result.0.as_array().unwrap();
        assert!(arr.is_empty());
    }

    #[tokio::test]
    async fn remove_missing_key_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let err = remove_key(State(app), Path("nobody".to_string()))
            .await
            .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_missing_env_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let err = delete_env(State(app), Path("production".to_string()))
            .await
            .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::NOT_FOUND);
    }
}
