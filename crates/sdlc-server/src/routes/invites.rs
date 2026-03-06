//! OTP invite route handlers.
//!
//! Admin endpoints for creating/listing/revoking invites, plus the public
//! OTP verification endpoint that issues a signed session cookie.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::error::AppError;
use crate::invite::{InviteError, InviteStore};
use crate::oauth::{self, SessionPayload};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract and validate an admin session from the `sdlc_session` cookie.
/// The caller must have a Google account on an allowed domain.
fn require_admin_session(headers: &HeaderMap, app: &AppState) -> Result<SessionPayload, AppError> {
    let config = app
        .oauth_config
        .as_ref()
        .ok_or_else(|| AppError::unauthorized("OAuth not configured"))?;

    let cookies = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    for part in cookies.split(';') {
        if let Some(val) = part.trim().strip_prefix("sdlc_session=") {
            if let Some(payload) = oauth::verify_session(&config.session_secret, val) {
                // Check email domain is in allowed_domains
                let email_domain = payload
                    .email
                    .rsplit_once('@')
                    .map(|(_, d)| d.to_lowercase())
                    .unwrap_or_default();
                if config.allowed_domains.iter().any(|d| d == &email_domain) {
                    return Ok(payload);
                }
                return Err(AppError::unauthorized("domain not in allowed list"));
            }
        }
    }

    Err(AppError::unauthorized("valid admin session required"))
}

/// Get a reference to the invite store, or 404 if not initialized.
fn store(app: &AppState) -> Result<&InviteStore, AppError> {
    app.invite_store
        .get()
        .ok_or_else(|| AppError::not_found("invite system not available"))
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// POST /api/invites — create a new OTP invite.
pub async fn create_invite(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateInviteRequest>,
) -> Result<Response, AppError> {
    let session = require_admin_session(&headers, &app)?;
    let store = store(&app)?;

    let (record, otp) = store
        .create(&body.email, &session.email)
        .await
        .map_err(AppError::bad_request)?;

    // Fire-and-forget OTP email delivery (if notify is configured).
    if let Some(client) = app.notify_client.clone() {
        client.send_otp_background(record.email.clone(), otp.clone());
    }

    let response = serde_json::json!({
        "id": record.id,
        "email": record.email,
        "otp": otp,
        "created_at": record.created_at,
        "expires_at": record.expires_at,
    });

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

/// GET /api/invites — list all invites.
pub async fn list_invites(
    State(app): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::invite::InviteRecord>>, AppError> {
    require_admin_session(&headers, &app)?;
    let store = store(&app)?;
    let records = store.list().await.map_err(AppError::bad_request)?;
    Ok(Json(records))
}

/// DELETE /api/invites/{id} — revoke an invite.
pub async fn revoke_invite(
    State(app): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    require_admin_session(&headers, &app)?;
    let store = store(&app)?;
    store.revoke(&id).await.map_err(AppError::not_found)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /auth/otp — verify an OTP and issue a session cookie.
pub async fn verify_otp(
    State(app): State<AppState>,
    Json(body): Json<VerifyOtpRequest>,
) -> Result<Response, AppError> {
    let config = app
        .oauth_config
        .as_ref()
        .ok_or_else(|| AppError::unauthorized("OAuth not configured"))?;

    let store = store(&app)?;

    let record = match store.verify(&body.email, &body.otp).await {
        Ok(r) => r,
        Err(InviteError::RateLimited) => {
            return Err(AppError::too_many_requests("too many attempts"));
        }
        Err(InviteError::InvalidOrExpired) => {
            return Err(AppError::unauthorized("invalid or expired code"));
        }
    };

    // Build session cookie (same as OAuth callback)
    let exp = chrono::Utc::now().timestamp() + 86400; // 24 hours
    let payload = SessionPayload {
        email: record.email.clone(),
        name: record.email.clone(),
        exp,
    };

    let cookie_value = oauth::sign_session(&config.session_secret, &payload)
        .ok_or_else(|| AppError::bad_request("session signing failed"))?;

    let cookie = format!(
        "sdlc_session={cookie_value}; HttpOnly; Secure; SameSite=Lax; Domain={}; Path=/; Max-Age=86400",
        config.cookie_domain
    );

    let body = serde_json::json!({
        "email": record.email,
        "name": record.email,
    });

    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Set-Cookie", cookie)
        .body(axum::body::Body::from(
            serde_json::to_string(&body).unwrap_or_default(),
        ))
        .expect("valid response"))
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CreateInviteRequest {
    pub email: String,
}

#[derive(serde::Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub otp: String,
}
