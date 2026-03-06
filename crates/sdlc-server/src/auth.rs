use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::state::TunnelSnapshot;

/// Controls tunnel authentication.
///
/// When `tokens` is empty the middleware is a transparent no-op — all requests
/// pass through (open mode). Tokens are populated from `.sdlc/auth.yaml` on
/// startup and hot-reloaded whenever the file changes. The ephemeral tunnel
/// token (generated on `POST /api/tunnel`) is added as `("_tunnel", token)` in
/// memory only and is never written to disk.
#[derive(Clone, Debug)]
pub struct TunnelConfig {
    /// Named token list: `(name, token_value)`. Empty = no-auth (open) mode.
    pub tokens: Vec<(String, String)>,
    /// Hostname of the active app tunnel (e.g. "fancy-rabbit.trycloudflare.com").
    /// When set, requests arriving with this Host header bypass SDLC auth so the
    /// reverse-proxy can serve the user's app without auth friction for reviewers.
    /// `/api/*` paths via the app tunnel host still require auth.
    pub app_tunnel_host: Option<String>,
}

impl TunnelConfig {
    /// No tokens configured — middleware passes all requests through (open mode).
    pub fn none() -> Self {
        Self {
            tokens: Vec::new(),
            app_tunnel_host: None,
        }
    }

    /// Single ephemeral token (backward-compat shim for tunnel start flow).
    /// Adds `("_tunnel", token)` to the token list without writing to disk.
    pub fn with_token(token: String) -> Self {
        Self {
            tokens: vec![("_tunnel".to_string(), token)],
            app_tunnel_host: None,
        }
    }

    /// Named token list constructor — used when loading from `auth.yaml`.
    pub fn with_tokens(tokens: Vec<(String, String)>) -> Self {
        Self {
            tokens,
            app_tunnel_host: None,
        }
    }

    /// Builder: set the app tunnel hostname.
    pub fn with_app_tunnel_host(mut self, host: String) -> Self {
        self.app_tunnel_host = Some(host);
        self
    }

    /// Returns `true` if `value` matches any token in the list.
    pub fn is_valid_token(&self, value: &str) -> bool {
        self.tokens.iter().any(|(_, t)| t == value)
    }
}

/// Axum middleware that gates requests behind named tokens when any are configured.
///
/// Auth flow (evaluated in order):
/// 1. `tokens` is empty → passthrough (no-auth / open mode)
/// 2. `Host` header is `localhost` or `127.0.0.1` → passthrough (local always allowed)
/// 3. Path starts with `/__sdlc/` → passthrough (feedback widget endpoint, always public)
/// 4. Host == `app_tunnel_host` AND path does NOT start with `/api/` → passthrough
///    (proxy requests bypass SDLC auth; `/api/*` via app tunnel still gets normal auth)
/// 5. Cookie `sdlc_auth` matches any token → passthrough
/// 6. `Authorization: Bearer <TOKEN>` matches any token → passthrough
/// 7. Query param `?auth=TOKEN` matches any token → set session cookie, 302 to same path
/// 8. None matched → 401 (JSON for `/api/*`, HTML for everything else)
pub async fn auth_middleware(
    State(snapshot): State<Arc<RwLock<TunnelSnapshot>>>,
    req: Request,
    next: Next,
) -> Response {
    let snap = snapshot.read().await;
    if snap.config.tokens.is_empty() {
        drop(snap);
        return next.run(req).await;
    }
    let config = snap.config.clone();
    let app_tunnel_host = config.app_tunnel_host.clone();
    let oauth_enabled = snap.oauth_enabled;
    drop(snap);

    // Local access is always allowed regardless of token.
    let host_value = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bare_host = host_value.split(':').next().unwrap_or(&host_value);
    if bare_host == "localhost" || bare_host == "127.0.0.1" {
        return next.run(req).await;
    }

    // Health check — always public so k8s probes work without auth.
    if req.uri().path() == "/api/health" {
        return next.run(req).await;
    }

    // Feedback widget endpoint — always public regardless of token or host.
    if req.uri().path().starts_with("/__sdlc/") {
        return next.run(req).await;
    }

    // OAuth routes — always public (login/callback create sessions, verify is for Traefik).
    if req.uri().path().starts_with("/auth/") {
        return next.run(req).await;
    }

    // App tunnel host: proxy requests bypass SDLC auth; /api/* still requires auth.
    if let Some(ref athost) = app_tunnel_host {
        if bare_host == athost && !req.uri().path().starts_with("/api/") {
            return next.run(req).await;
        }
    }

    // Valid session cookie — allow if it matches any tunnel token.
    if let Some(cookies) = req.headers().get("cookie").and_then(|v| v.to_str().ok()) {
        for part in cookies.split(';') {
            let trimmed = part.trim();
            if let Some(val) = trimmed.strip_prefix("sdlc_auth=") {
                if config.is_valid_token(val) {
                    return next.run(req).await;
                }
            }
            // OAuth session cookie — if present, pass through (OAuth handlers validate it).
            if oauth_enabled && trimmed.starts_with("sdlc_session=") {
                return next.run(req).await;
            }
        }
    }

    // Authorization: Bearer <TOKEN> — allow if it matches any token.
    if let Some(auth_header) = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(bearer_val) = auth_header.strip_prefix("Bearer ") {
            if config.is_valid_token(bearer_val) {
                return next.run(req).await;
            }
        }
    }

    // One-time bootstrap via `?auth=TOKEN` — set cookie and redirect.
    let uri = req.uri().clone();
    if let Some(query) = uri.query() {
        if let Some(val) = extract_auth_param(query) {
            if config.is_valid_token(val) {
                let path_only = strip_auth_param(uri.path(), query);
                let destination = format!("https://{host_value}{path_only}");
                let cookie = format!(
                    "sdlc_auth={val}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=2592000"
                );
                return Response::builder()
                    .status(302)
                    .header("Location", destination)
                    .header("Set-Cookie", cookie)
                    .body(Body::empty())
                    .expect("infallible: all header values are valid ASCII");
            }
        }
    }

    // Unauthorized — JSON for API routes, HTML/redirect for everything else.
    if req.uri().path().starts_with("/api/") {
        Response::builder()
            .status(401)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"unauthorized"}"#))
            .expect("infallible: all header values are valid ASCII")
    } else if oauth_enabled {
        // OAuth mode: redirect to login page.
        Response::builder()
            .status(302)
            .header("Location", "/auth/login")
            .body(Body::empty())
            .expect("infallible: all header values are valid ASCII")
    } else {
        Response::builder()
            .status(401)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Body::from(concat!(
                "<!DOCTYPE html><html><head><title>Access Denied</title></head>",
                "<body style=\"font-family:sans-serif;padding:2rem\">",
                "<h1>Access Denied</h1>",
                "<p>Scan the QR code displayed in the terminal to access the SDLC UI.</p>",
                "</body></html>",
            )))
            .expect("infallible: all header values are valid ASCII")
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_auth_param(query: &str) -> Option<&str> {
    query.split('&').find_map(|kv| kv.strip_prefix("auth="))
}

fn strip_auth_param(path: &str, query: &str) -> String {
    let remaining: Vec<&str> = query
        .split('&')
        .filter(|kv| !kv.starts_with("auth="))
        .collect();
    if remaining.is_empty() {
        path.to_string()
    } else {
        format!("{}?{}", path, remaining.join("&"))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{body::Body, http::Request, middleware, routing::get, Router};
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn test_app(config: TunnelConfig) -> Router {
        let arc = Arc::new(RwLock::new(TunnelSnapshot {
            config,
            url: None,
            oauth_enabled: false,
        }));
        Router::new()
            .route("/", get(ok_handler))
            .route("/api/state", get(ok_handler))
            .route("/__sdlc/feedback", get(ok_handler))
            .layer(middleware::from_fn_with_state(arc, auth_middleware))
    }

    #[tokio::test]
    async fn no_token_passes_through() {
        let resp = test_app(TunnelConfig::none())
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn empty_tokens_passthrough() {
        let config = TunnelConfig::with_tokens(vec![]);
        let resp = test_app(config)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn localhost_bypasses_auth() {
        let resp = test_app(TunnelConfig::with_token("secret".into()))
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("host", "localhost:3141")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn loopback_bypasses_auth() {
        let resp = test_app(TunnelConfig::with_token("secret".into()))
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("host", "127.0.0.1:3141")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_cookie_passes_through() {
        let resp = test_app(TunnelConfig::with_token("secret".into()))
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("host", "abc.trycloudflare.com")
                    .header("cookie", "sdlc_auth=secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn multi_token_first_token_cookie_passes() {
        let config = TunnelConfig::with_tokens(vec![
            ("jordan".to_string(), "tok1xxxx".to_string()),
            ("ci-bot".to_string(), "tok2yyyy".to_string()),
        ]);
        let resp = test_app(config)
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("host", "abc.trycloudflare.com")
                    .header("cookie", "sdlc_auth=tok1xxxx")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn multi_token_second_token_cookie_passes() {
        let config = TunnelConfig::with_tokens(vec![
            ("jordan".to_string(), "tok1xxxx".to_string()),
            ("ci-bot".to_string(), "tok2yyyy".to_string()),
        ]);
        let resp = test_app(config)
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("host", "abc.trycloudflare.com")
                    .header("cookie", "sdlc_auth=tok2yyyy")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_header_passes_auth() {
        let config =
            TunnelConfig::with_tokens(vec![("jordan".to_string(), "bearer1x".to_string())]);
        let resp = test_app(config)
            .oneshot(
                Request::builder()
                    .uri("/api/state")
                    .header("host", "abc.trycloudflare.com")
                    .header("authorization", "Bearer bearer1x")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bearer_header_wrong_token_401() {
        let config =
            TunnelConfig::with_tokens(vec![("jordan".to_string(), "bearer1x".to_string())]);
        let resp = test_app(config)
            .oneshot(
                Request::builder()
                    .uri("/api/state")
                    .header("host", "abc.trycloudflare.com")
                    .header("authorization", "Bearer wrongtok")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_query_param_sets_cookie_and_redirects() {
        let resp = test_app(TunnelConfig::with_token("secret".into()))
            .oneshot(
                Request::builder()
                    .uri("/?auth=secret")
                    .header("host", "abc.trycloudflare.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("location").unwrap().to_str().unwrap();
        assert_eq!(location, "https://abc.trycloudflare.com/");
        let cookie = resp.headers().get("set-cookie").unwrap().to_str().unwrap();
        assert!(cookie.contains("sdlc_auth=secret"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("Max-Age=2592000"));
    }

    #[tokio::test]
    async fn missing_token_returns_401_html() {
        let resp = test_app(TunnelConfig::with_token("secret".into()))
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("host", "abc.trycloudflare.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let ct = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(ct.contains("text/html"));
    }

    #[tokio::test]
    async fn api_path_without_token_returns_401_json() {
        let resp = test_app(TunnelConfig::with_token("secret".into()))
            .oneshot(
                Request::builder()
                    .uri("/api/state")
                    .header("host", "abc.trycloudflare.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let ct = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(ct.contains("application/json"));
    }

    #[tokio::test]
    async fn sdlc_path_bypasses_auth() {
        let resp = test_app(TunnelConfig::with_token("secret".into()))
            .oneshot(
                Request::builder()
                    .uri("/__sdlc/feedback")
                    .header("host", "abc.trycloudflare.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_tunnel_host_bypasses_auth_for_non_api() {
        let config = TunnelConfig::with_token("secret".into())
            .with_app_tunnel_host("fancy-rabbit.trycloudflare.com".into());
        let resp = test_app(config)
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("host", "fancy-rabbit.trycloudflare.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_tunnel_host_still_blocks_api_routes() {
        let config = TunnelConfig::with_token("secret".into())
            .with_app_tunnel_host("fancy-rabbit.trycloudflare.com".into());
        let resp = test_app(config)
            .oneshot(
                Request::builder()
                    .uri("/api/state")
                    .header("host", "fancy-rabbit.trycloudflare.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn strip_auth_only_param() {
        assert_eq!(strip_auth_param("/", "auth=abc"), "/");
    }

    #[test]
    fn strip_auth_preserves_other_params() {
        assert_eq!(
            strip_auth_param("/foo", "bar=1&auth=abc&baz=2"),
            "/foo?bar=1&baz=2"
        );
    }

    #[test]
    fn extract_auth_param_found() {
        assert_eq!(extract_auth_param("auth=tok"), Some("tok"));
        assert_eq!(extract_auth_param("x=1&auth=tok"), Some("tok"));
    }

    #[test]
    fn extract_auth_param_not_found() {
        assert_eq!(extract_auth_param("x=1"), None);
    }
}
