//! Native Google OAuth2 for hub mode.
//!
//! Four route handlers: login, callback, verify, logout.
//! Session state is a signed cookie — no external session store needed.

use crate::state::AppState;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// OAuth2 configuration for Google sign-in. Constructed from env vars in hub mode.
/// When `None` in AppState, OAuth routes return 404.
#[derive(Clone, Debug)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub allowed_domains: Vec<String>,
    pub session_secret: Vec<u8>,
    /// Override callback URL. If `None`, derived from request Host header.
    pub redirect_uri: Option<String>,
    /// Cookie domain for session cookies (e.g. ".sdlc.threesix.ai").
    pub cookie_domain: String,
}

impl OAuthConfig {
    /// Build from environment variables. Returns `None` if required vars are missing.
    pub fn from_env() -> Option<Self> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID").ok()?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").ok()?;
        let allowed_domains_str = std::env::var("OAUTH_ALLOWED_DOMAINS").ok()?;
        let session_secret = std::env::var("SESSION_SECRET").ok()?;

        if client_id.is_empty() || client_secret.is_empty() || session_secret.is_empty() {
            return None;
        }

        let allowed_domains: Vec<String> = allowed_domains_str
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        if allowed_domains.is_empty() {
            return None;
        }

        let redirect_uri = std::env::var("OAUTH_REDIRECT_URI").ok();
        let cookie_domain = std::env::var("OAUTH_COOKIE_DOMAIN")
            .unwrap_or_else(|_| ".sdlc.threesix.ai".to_string());

        Some(Self {
            client_id,
            client_secret,
            allowed_domains,
            session_secret: session_secret.into_bytes(),
            redirect_uri,
            cookie_domain,
        })
    }
}

// ---------------------------------------------------------------------------
// Session payload
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SessionPayload {
    pub email: String,
    pub name: String,
    /// Expiry as Unix timestamp (seconds).
    pub exp: i64,
}

/// Sign a session payload: base64(json) + "." + hex(hmac).
pub fn sign_session(secret: &[u8], payload: &SessionPayload) -> Option<String> {
    let json = serde_json::to_vec(payload).ok()?;
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &json);
    let mut mac = HmacSha256::new_from_slice(secret).ok()?;
    mac.update(encoded.as_bytes());
    let sig = hex::encode(&mac.finalize().into_bytes());
    Some(format!("{encoded}.{sig}"))
}

/// Verify and decode a session cookie value.
pub fn verify_session(secret: &[u8], cookie_value: &str) -> Option<SessionPayload> {
    let (encoded, sig_hex) = cookie_value.rsplit_once('.')?;
    let mut mac = HmacSha256::new_from_slice(secret).ok()?;
    mac.update(encoded.as_bytes());
    let expected_sig = mac.finalize().into_bytes();
    let provided_sig = hex::decode(sig_hex).ok()?;
    if !constant_time_eq(&expected_sig, &provided_sig) {
        return None;
    }

    let json_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, encoded).ok()?;
    let payload: SessionPayload = serde_json::from_slice(&json_bytes).ok()?;

    // Check expiry
    let now = chrono::Utc::now().timestamp();
    if payload.exp < now {
        return None;
    }

    Some(payload)
}

/// Constant-time comparison to prevent timing attacks on HMAC verification.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// We need hex encoding for HMAC output. Use a minimal inline implementation
// to avoid adding another dependency.
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, ()> {
        if !s.len().is_multiple_of(2) {
            return Err(());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ()))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// GET /auth/login — redirect to Google sign-in.
pub async fn login(State(app): State<AppState>) -> Response {
    let Some(config) = &app.oauth_config else {
        return (StatusCode::NOT_FOUND, "OAuth not configured").into_response();
    };

    let redirect_uri = config.redirect_uri.clone().unwrap_or_else(|| {
        let domain = config.cookie_domain.trim_start_matches('.');
        format!("https://{domain}/auth/callback")
    });

    // CSRF state: HMAC-signed timestamp
    let now = chrono::Utc::now().timestamp().to_string();
    let mut mac =
        HmacSha256::new_from_slice(&config.session_secret).expect("HMAC accepts any key length");
    mac.update(now.as_bytes());
    let state_sig = hex::encode(&mac.finalize().into_bytes());
    let state = format!("{now}.{state_sig}");

    let params = [
        ("client_id", config.client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", "openid email profile"),
        ("access_type", "online"),
        ("prompt", "select_account"),
        ("state", state.as_str()),
    ];
    let query = params
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let url = format!("https://accounts.google.com/o/oauth2/v2/auth?{query}");

    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", url)
        .body(Body::empty())
        .expect("valid response")
}

/// Query params for the OAuth callback.
#[derive(serde::Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// GET /auth/callback — exchange code, validate domain, set session cookie.
pub async fn callback(
    State(app): State<AppState>,
    Query(params): Query<CallbackParams>,
) -> Response {
    let Some(config) = &app.oauth_config else {
        return (StatusCode::NOT_FOUND, "OAuth not configured").into_response();
    };

    // Handle errors from Google
    if let Some(err) = &params.error {
        return (StatusCode::FORBIDDEN, format!("OAuth error: {err}")).into_response();
    }

    let Some(code) = &params.code else {
        return (StatusCode::BAD_REQUEST, "Missing authorization code").into_response();
    };

    // Validate CSRF state
    if let Some(state) = &params.state {
        if !verify_csrf_state(&config.session_secret, state) {
            return (StatusCode::BAD_REQUEST, "Invalid state parameter").into_response();
        }
    }

    let redirect_uri = config.redirect_uri.clone().unwrap_or_else(|| {
        let domain = config.cookie_domain.trim_start_matches('.');
        format!("https://{domain}/auth/callback")
    });

    // Exchange code for token
    let token_resp = match exchange_code(&app.http_client, config, code, &redirect_uri).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "OAuth token exchange failed");
            return (StatusCode::BAD_GATEWAY, "Token exchange failed").into_response();
        }
    };

    // Fetch user info
    let user_info = match fetch_userinfo(&app.http_client, &token_resp.access_token).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(error = %e, "OAuth userinfo fetch failed");
            return (StatusCode::BAD_GATEWAY, "Failed to fetch user info").into_response();
        }
    };

    // Validate email domain
    let email_domain = user_info
        .email
        .rsplit_once('@')
        .map(|(_, d)| d.to_lowercase())
        .unwrap_or_default();

    if !config.allowed_domains.iter().any(|d| d == &email_domain) {
        tracing::warn!(
            email = %user_info.email,
            domain = %email_domain,
            "OAuth login rejected: domain not allowed"
        );
        return (
            StatusCode::FORBIDDEN,
            format!("Domain @{email_domain} is not allowed"),
        )
            .into_response();
    }

    // Build session cookie
    let exp = chrono::Utc::now().timestamp() + 86400; // 24 hours
    let payload = SessionPayload {
        email: user_info.email.clone(),
        name: user_info.name.unwrap_or_else(|| user_info.email.clone()),
        exp,
    };

    let Some(cookie_value) = sign_session(&config.session_secret, &payload) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Session signing failed").into_response();
    };

    let cookie = format!(
        "sdlc_session={cookie_value}; HttpOnly; Secure; SameSite=Lax; Domain={}; Path=/; Max-Age=86400",
        config.cookie_domain
    );

    tracing::info!(email = %user_info.email, "OAuth login successful");

    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/")
        .header("Set-Cookie", cookie)
        .body(Body::empty())
        .expect("valid response")
}

/// GET /auth/verify — Traefik forwardAuth endpoint.
///
/// Returns 200 with X-Auth-User if session is valid.
/// Also accepts Bearer tokens from `hub_service_tokens` for M2M.
/// Returns 401 if no valid session or token.
pub async fn verify(State(app): State<AppState>, req: axum::extract::Request) -> Response {
    let config = match &app.oauth_config {
        Some(c) => c,
        None => {
            // No OAuth configured — pass through (project mode)
            return Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .expect("valid response");
        }
    };

    // Check session cookie
    if let Some(cookies) = req.headers().get("cookie").and_then(|v| v.to_str().ok()) {
        for part in cookies.split(';') {
            if let Some(val) = part.trim().strip_prefix("sdlc_session=") {
                if let Some(payload) = verify_session(&config.session_secret, val) {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header("X-Auth-User", &payload.email)
                        .body(Body::empty())
                        .expect("valid response");
                }
            }
        }
    }

    // Check Bearer token (M2M path)
    if let Some(auth_header) = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if app.hub_service_tokens.iter().any(|t| t == token) {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("X-Auth-User", "service-token")
                    .body(Body::empty())
                    .expect("valid response");
            }
        }
    }

    // Not authenticated — return 401.
    // Include a Location header hint so Traefik can redirect to login.
    let login_url = {
        let domain = config.cookie_domain.trim_start_matches('.');
        format!("https://{domain}/auth/login")
    };

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(
            "X-Auth-Login-Url",
            HeaderValue::from_str(&login_url)
                .unwrap_or_else(|_| HeaderValue::from_static("/auth/login")),
        )
        .body(Body::empty())
        .expect("valid response")
}

/// POST /auth/logout — clear session cookie, redirect to /.
pub async fn logout(State(app): State<AppState>) -> Response {
    let domain = app
        .oauth_config
        .as_ref()
        .map(|c| c.cookie_domain.as_str())
        .unwrap_or(".sdlc.threesix.ai");

    let clear_cookie = format!(
        "sdlc_session=; HttpOnly; Secure; SameSite=Lax; Domain={domain}; Path=/; Max-Age=0"
    );

    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/")
        .header("Set-Cookie", clear_cookie)
        .body(Body::empty())
        .expect("valid response")
}

// ---------------------------------------------------------------------------
// Google API helpers
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: Option<String>,
    #[allow(dead_code)]
    expires_in: Option<u64>,
}

#[derive(serde::Deserialize, Debug)]
struct UserInfo {
    email: String,
    name: Option<String>,
    #[allow(dead_code)]
    picture: Option<String>,
}

async fn exchange_code(
    http_client: &reqwest::Client,
    config: &OAuthConfig,
    code: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    let resp = http_client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| format!("token request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("token exchange HTTP {status}: {body}"));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| format!("token parse failed: {e}"))
}

async fn fetch_userinfo(
    http_client: &reqwest::Client,
    access_token: &str,
) -> Result<UserInfo, String> {
    let resp = http_client
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("userinfo request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("userinfo HTTP {status}: {body}"));
    }

    resp.json::<UserInfo>()
        .await
        .map_err(|e| format!("userinfo parse failed: {e}"))
}

fn verify_csrf_state(secret: &[u8], state: &str) -> bool {
    let Some((timestamp_str, sig_hex)) = state.rsplit_once('.') else {
        return false;
    };
    let Ok(timestamp) = timestamp_str.parse::<i64>() else {
        return false;
    };

    // State must be less than 10 minutes old
    let now = chrono::Utc::now().timestamp();
    if (now - timestamp).abs() > 600 {
        return false;
    }

    let mut mac = match HmacSha256::new_from_slice(secret) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(timestamp_str.as_bytes());
    let expected = hex::encode(&mac.finalize().into_bytes());
    constant_time_eq(expected.as_bytes(), sig_hex.as_bytes())
}

// We need urlencoding for the authorize URL query params.
mod urlencoding {
    pub fn encode(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                _ => {
                    result.push_str(&format!("%{byte:02X}"));
                }
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "test-client-id".into(),
            client_secret: "test-client-secret".into(),
            allowed_domains: vec!["livelyideo.tv".into(), "masq.me".into()],
            session_secret: b"test-secret-at-least-32-chars-long!!".to_vec(),
            redirect_uri: Some("https://sdlc.threesix.ai/auth/callback".into()),
            cookie_domain: ".sdlc.threesix.ai".into(),
        }
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let secret = b"test-secret-key-32chars-minimum!";
        let payload = SessionPayload {
            email: "test@livelyideo.tv".into(),
            name: "Test User".into(),
            exp: chrono::Utc::now().timestamp() + 3600,
        };
        let signed = sign_session(secret, &payload).unwrap();
        let verified = verify_session(secret, &signed).unwrap();
        assert_eq!(verified.email, "test@livelyideo.tv");
        assert_eq!(verified.name, "Test User");
    }

    #[test]
    fn verify_session_wrong_secret_fails() {
        let secret = b"correct-secret-at-least-32chars!";
        let payload = SessionPayload {
            email: "test@livelyideo.tv".into(),
            name: "Test".into(),
            exp: chrono::Utc::now().timestamp() + 3600,
        };
        let signed = sign_session(secret, &payload).unwrap();
        assert!(verify_session(b"wrong-secret-at-least-32-chars!!", &signed).is_none());
    }

    #[test]
    fn verify_session_expired_fails() {
        let secret = b"test-secret-key-32chars-minimum!";
        let payload = SessionPayload {
            email: "test@livelyideo.tv".into(),
            name: "Test".into(),
            exp: chrono::Utc::now().timestamp() - 100, // already expired
        };
        let signed = sign_session(secret, &payload).unwrap();
        assert!(verify_session(secret, &signed).is_none());
    }

    #[test]
    fn verify_session_tampered_fails() {
        let secret = b"test-secret-key-32chars-minimum!";
        let payload = SessionPayload {
            email: "test@livelyideo.tv".into(),
            name: "Test".into(),
            exp: chrono::Utc::now().timestamp() + 3600,
        };
        let signed = sign_session(secret, &payload).unwrap();
        // Tamper with the signature
        let tampered = format!("{}x", signed);
        assert!(verify_session(secret, &tampered).is_none());
    }

    #[test]
    fn verify_session_garbage_input_returns_none() {
        let secret = b"test-secret-key-32chars-minimum!";
        assert!(verify_session(secret, "").is_none());
        assert!(verify_session(secret, "no-dot-separator").is_none());
        assert!(verify_session(secret, "invalid.deadbeef").is_none());
    }

    #[test]
    fn csrf_state_valid() {
        let secret = b"test-secret-for-csrf";
        let now = chrono::Utc::now().timestamp().to_string();
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(now.as_bytes());
        let sig = hex::encode(&mac.finalize().into_bytes());
        let state = format!("{now}.{sig}");
        assert!(verify_csrf_state(secret, &state));
    }

    #[test]
    fn csrf_state_expired_fails() {
        let secret = b"test-secret-for-csrf";
        let old = (chrono::Utc::now().timestamp() - 700).to_string(); // > 10 min
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(old.as_bytes());
        let sig = hex::encode(&mac.finalize().into_bytes());
        let state = format!("{old}.{sig}");
        assert!(!verify_csrf_state(secret, &state));
    }

    #[test]
    fn urlencoding_special_chars() {
        assert_eq!(urlencoding::encode("hello world"), "hello%20world");
        assert_eq!(urlencoding::encode("a=b&c=d"), "a%3Db%26c%3Dd");
        assert_eq!(
            urlencoding::encode("safe-chars_ok.here~"),
            "safe-chars_ok.here~"
        );
    }

    #[test]
    fn hex_roundtrip() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let encoded = hex::encode(&data);
        assert_eq!(encoded, "deadbeef");
        let decoded = hex::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[tokio::test]
    async fn login_redirects_to_google() {
        let app_state = test_app_state_with_oauth();
        let app = Router::new()
            .route("/auth/login", get(login))
            .with_state(app_state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("location").unwrap().to_str().unwrap();
        assert!(location.starts_with("https://accounts.google.com/o/oauth2/v2/auth"));
        assert!(location.contains("client_id=test-client-id"));
        assert!(location.contains("scope=openid"));
    }

    #[tokio::test]
    async fn verify_with_valid_cookie_returns_200() {
        let config = test_config();
        let payload = SessionPayload {
            email: "user@livelyideo.tv".into(),
            name: "User".into(),
            exp: chrono::Utc::now().timestamp() + 3600,
        };
        let cookie_val = sign_session(&config.session_secret, &payload).unwrap();

        let app_state = test_app_state_with_oauth();
        let app = Router::new()
            .route("/auth/verify", get(verify))
            .with_state(app_state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/verify")
                    .header("cookie", format!("sdlc_session={cookie_val}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let user = resp.headers().get("X-Auth-User").unwrap().to_str().unwrap();
        assert_eq!(user, "user@livelyideo.tv");
    }

    #[tokio::test]
    async fn verify_with_missing_cookie_returns_401() {
        let app_state = test_app_state_with_oauth();
        let app = Router::new()
            .route("/auth/verify", get(verify))
            .with_state(app_state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/verify")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn verify_with_expired_cookie_returns_401() {
        let config = test_config();
        let payload = SessionPayload {
            email: "user@livelyideo.tv".into(),
            name: "User".into(),
            exp: chrono::Utc::now().timestamp() - 100,
        };
        let cookie_val = sign_session(&config.session_secret, &payload).unwrap();

        let app_state = test_app_state_with_oauth();
        let app = Router::new()
            .route("/auth/verify", get(verify))
            .with_state(app_state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/verify")
                    .header("cookie", format!("sdlc_session={cookie_val}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn verify_with_bearer_service_token_returns_200() {
        let mut app_state = test_app_state_with_oauth();
        app_state.hub_service_tokens = vec!["my-service-token".into()];

        let app = Router::new()
            .route("/auth/verify", get(verify))
            .with_state(app_state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/verify")
                    .header("authorization", "Bearer my-service-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let user = resp.headers().get("X-Auth-User").unwrap().to_str().unwrap();
        assert_eq!(user, "service-token");
    }

    #[tokio::test]
    async fn logout_clears_cookie() {
        let app_state = test_app_state_with_oauth();
        let app = Router::new()
            .route("/auth/logout", axum::routing::post(logout))
            .with_state(app_state);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::FOUND);
        let cookie = resp.headers().get("set-cookie").unwrap().to_str().unwrap();
        assert!(cookie.contains("sdlc_session=;"));
        assert!(cookie.contains("Max-Age=0"));
    }

    #[tokio::test]
    async fn login_without_config_returns_404() {
        let app_state = test_app_state_no_oauth();
        let app = Router::new()
            .route("/auth/login", get(login))
            .with_state(app_state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    fn test_app_state_with_oauth() -> AppState {
        let mut state = AppState::new(std::path::PathBuf::from("/tmp/test-oauth"));
        state.oauth_config = Some(Arc::new(test_config()));
        state
    }

    fn test_app_state_no_oauth() -> AppState {
        AppState::new(std::path::PathBuf::from("/tmp/test-oauth-none"))
    }
}
