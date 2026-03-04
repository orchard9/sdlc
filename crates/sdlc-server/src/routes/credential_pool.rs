/// REST handlers for the Claude credential pool.
///
/// All write endpoints (add, toggle, delete) require SDLC_AGENT_TOKEN bearer
/// authentication so only trusted callers (the claude-credentials tool) can
/// mutate the pool.
///
/// GET endpoints return pool status/listing without the token field — tokens
/// are never returned over the wire.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use tracing::warn;

use crate::{
    credential_pool::{CredentialRow, OptionalCredentialPool, PoolStatus},
    error::AppError,
    state::AppState,
};

// ---------------------------------------------------------------------------
// Auth helper
// ---------------------------------------------------------------------------

fn require_agent_token(headers: &HeaderMap, app: &AppState) -> Result<(), AppError> {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    if token == app.agent_token.as_str() {
        Ok(())
    } else {
        Err(AppError::unauthorized(
            "invalid or missing SDLC_AGENT_TOKEN",
        ))
    }
}

// ---------------------------------------------------------------------------
// Pool access helper
// ---------------------------------------------------------------------------

fn active_pool(app: &AppState) -> Result<&crate::credential_pool::CredentialPool, AppError> {
    match app.credential_pool.get() {
        Some(OptionalCredentialPool::Active(p)) => Ok(p),
        Some(OptionalCredentialPool::Disabled) => Err(AppError(anyhow::anyhow!(
            "credential pool is disabled — DATABASE_URL not configured"
        ))),
        None => Err(AppError(anyhow::anyhow!(
            "credential pool is still initializing — retry in a moment"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
pub struct StatusResponse {
    pub connected: bool,
    pub status: Option<PoolStatus>,
    pub message: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/credential-pool — pool status
// ---------------------------------------------------------------------------

pub async fn get_status(State(app): State<AppState>) -> Json<StatusResponse> {
    match app.credential_pool.get() {
        None => Json(StatusResponse {
            connected: false,
            status: None,
            message: Some("initializing".to_string()),
        }),
        Some(OptionalCredentialPool::Disabled) => Json(StatusResponse {
            connected: false,
            status: None,
            message: Some("disabled — DATABASE_URL not configured".to_string()),
        }),
        Some(OptionalCredentialPool::Active(pool)) => match pool.status().await {
            Ok(s) => Json(StatusResponse {
                connected: true,
                status: Some(s),
                message: None,
            }),
            Err(e) => {
                warn!(error = %e, "credential pool status query failed");
                Json(StatusResponse {
                    connected: false,
                    status: None,
                    message: Some(format!("error: {e}")),
                })
            }
        },
    }
}

// ---------------------------------------------------------------------------
// GET /api/credential-pool/credentials — list credentials (no token field)
// ---------------------------------------------------------------------------

pub async fn list_credentials(
    State(app): State<AppState>,
) -> Result<Json<Vec<CredentialRow>>, AppError> {
    let pool = active_pool(&app)?;
    let rows = pool
        .list()
        .await
        .map_err(|e| AppError(anyhow::anyhow!("credential list failed: {e}")))?;
    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// POST /api/credential-pool/credentials — add a credential
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct AddCredentialBody {
    pub account_name: String,
    pub token: String,
}

pub async fn add_credential(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AddCredentialBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    require_agent_token(&headers, &app)?;
    if body.account_name.trim().is_empty() {
        return Err(AppError::bad_request("account_name is required"));
    }
    if body.token.trim().is_empty() {
        return Err(AppError::bad_request("token is required"));
    }
    let pool = active_pool(&app)?;
    let id = pool
        .add(&body.account_name, &body.token)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("add credential failed: {e}")))?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id, "account_name": body.account_name })),
    ))
}

// ---------------------------------------------------------------------------
// PATCH /api/credential-pool/credentials/:id — toggle is_active
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct PatchCredentialBody {
    pub is_active: bool,
}

pub async fn patch_credential(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<PatchCredentialBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_agent_token(&headers, &app)?;
    let pool = active_pool(&app)?;
    let updated = pool
        .set_active(id, body.is_active)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("patch credential failed: {e}")))?;
    if updated {
        Ok(Json(
            serde_json::json!({ "id": id, "is_active": body.is_active }),
        ))
    } else {
        Err(AppError::not_found(format!("credential {id} not found")))
    }
}

// ---------------------------------------------------------------------------
// DELETE /api/credential-pool/credentials/:id — remove a credential
// ---------------------------------------------------------------------------

pub async fn delete_credential(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    require_agent_token(&headers, &app)?;
    let pool = active_pool(&app)?;
    let deleted = pool
        .delete(id)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("delete credential failed: {e}")))?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("credential {id} not found")))
    }
}
