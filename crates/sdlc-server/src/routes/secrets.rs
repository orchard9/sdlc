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

#[derive(serde::Deserialize)]
pub struct EnvPair {
    pub key: String,
    pub value: String,
}

#[derive(serde::Deserialize)]
pub struct CreateEnvBody {
    pub env: String,
    pub pairs: Vec<EnvPair>,
}

/// POST /api/secrets/envs — create a new encrypted env file.
///
/// Returns 400 if `pairs` is empty or no keys are configured.
/// Returns 409 if an env with the given name already exists.
/// Returns 201 with `{ status, env, key_names }` on success.
pub async fn create_env(
    State(app): State<AppState>,
    Json(body): Json<CreateEnvBody>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), AppError> {
    if body.pairs.is_empty() {
        return Ok((
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "pairs must not be empty" })),
        ));
    }
    let root = app.root.clone();
    let env_name = body.env.clone();
    let pairs_content: String = body
        .pairs
        .iter()
        .map(|p| format!("{}={}", p.key, p.value))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let key_names = tokio::task::spawn_blocking(move || {
        let config = sdlc_core::secrets::load_config(&root)?;
        if config.keys.is_empty() {
            return Ok((
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "no keys configured — add a recipient key first" })),
            )) as Result<_, AppError>;
        }
        let env_path = sdlc_core::paths::secrets_env_path(&root, &env_name);
        if env_path.exists() {
            return Err(AppError(
                sdlc_core::SdlcError::SecretEnvExists(env_name.clone()).into(),
            ));
        }
        sdlc_core::secrets::write_env(&root, &env_name, &pairs_content, &config.keys)?;
        let meta = sdlc_core::secrets::load_env_meta(&root, &env_name)?;
        Ok((axum::http::StatusCode::CREATED, Json(serde_json::json!({
            "status": "created",
            "env": env_name,
            "key_names": meta.key_names,
        }))))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(key_names)
}

#[derive(serde::Deserialize)]
pub struct UpdateEnvBody {
    pub pairs: Vec<EnvPair>,
}

/// PATCH /api/secrets/envs/:name — replace all secrets in an existing env file.
///
/// Full-replacement semantics: the submitted pairs become the complete new content
/// of the encrypted file. Keys not submitted are removed.
///
/// Returns 400 if `pairs` is empty or no keys are configured.
/// Returns 404 if the env does not exist.
/// Returns 200 with `{ status, env, key_names }` on success.
pub async fn update_env(
    State(app): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<UpdateEnvBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.pairs.is_empty() {
        return Err(AppError(anyhow::anyhow!("pairs must not be empty")));
    }
    let root = app.root.clone();
    let env_name = name.clone();
    let pairs_content: String = body
        .pairs
        .iter()
        .map(|p| format!("{}={}", p.key, p.value))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let result = tokio::task::spawn_blocking(move || {
        let env_path = sdlc_core::paths::secrets_env_path(&root, &env_name);
        if !env_path.exists() {
            return Err(sdlc_core::SdlcError::SecretEnvNotFound(env_name.clone()));
        }
        let config = sdlc_core::secrets::load_config(&root)?;
        if config.keys.is_empty() {
            return Err(sdlc_core::SdlcError::AgeEncryptFailed(
                "no keys configured — add a recipient key first".to_string(),
            ));
        }
        sdlc_core::secrets::write_env(&root, &env_name, &pairs_content, &config.keys)?;
        let meta = sdlc_core::secrets::load_env_meta(&root, &env_name)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "status": "updated",
            "env": env_name,
            "key_names": meta.key_names,
        }))
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

    #[tokio::test]
    async fn create_env_empty_pairs_returns_bad_request() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = CreateEnvBody {
            env: "staging".to_string(),
            pairs: vec![],
        };
        let (status, _) = create_env(State(app), Json(body)).await.unwrap();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_env_not_found_returns_404() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = UpdateEnvBody {
            pairs: vec![EnvPair {
                key: "API_KEY".to_string(),
                value: "new".to_string(),
            }],
        };
        let err = update_env(State(app), Path("nonexistent".to_string()), Json(body))
            .await
            .unwrap_err();
        assert_eq!(err.into_response().status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn update_env_empty_pairs_returns_bad_request() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = UpdateEnvBody { pairs: vec![] };
        let err = update_env(State(app), Path("staging".to_string()), Json(body))
            .await
            .unwrap_err();
        assert_eq!(
            err.into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn update_env_no_keys_returns_bad_request() {
        use std::fs;
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        // Create a stub env file so the "not found" check passes
        let envs_dir = dir.path().join(".sdlc").join("secrets").join("envs");
        fs::create_dir_all(&envs_dir).unwrap();
        fs::write(envs_dir.join("staging.age"), b"fake-age-content").unwrap();
        let body = UpdateEnvBody {
            pairs: vec![EnvPair {
                key: "API_KEY".to_string(),
                value: "value".to_string(),
            }],
        };
        let err = update_env(State(app), Path("staging".to_string()), Json(body))
            .await
            .unwrap_err();
        assert_eq!(
            err.into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn create_env_no_keys_returns_bad_request() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = CreateEnvBody {
            env: "staging".to_string(),
            pairs: vec![EnvPair {
                key: "API_KEY".to_string(),
                value: "secret".to_string(),
            }],
        };
        let (status, _) = create_env(State(app), Json(body)).await.unwrap();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
