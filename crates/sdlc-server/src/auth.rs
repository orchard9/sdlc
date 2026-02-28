use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

/// Controls tunnel authentication.
///
/// When `token` is `None` the middleware is a transparent no-op — all requests
/// pass through. Set a token only when a public tunnel is active.
#[derive(Clone)]
pub struct TunnelConfig {
    pub token: Option<String>,
}

impl TunnelConfig {
    /// No tunnel active — middleware passes all requests through.
    pub fn none() -> Self {
        Self { token: None }
    }

    /// Tunnel is active with the given shared token.
    pub fn with_token(token: String) -> Self {
        Self { token: Some(token) }
    }
}

/// Axum middleware that gates requests behind a shared token when a tunnel
/// is active.
///
/// Auth flow (evaluated in order):
/// 1. `token` is `None` → passthrough (tunnel not active)
/// 2. `Host` header is `localhost` or `127.0.0.1` → passthrough (local always allowed)
/// 3. Cookie `sdlc_auth` matches token → passthrough
/// 4. Query param `?auth=TOKEN` matches → set session cookie, 302 to same path without param
/// 5. None matched → 401 (JSON for `/api/*`, HTML for everything else)
pub async fn auth_middleware(
    State(config): State<TunnelConfig>,
    req: Request,
    next: Next,
) -> Response {
    let Some(ref token) = config.token else {
        return next.run(req).await;
    };

    // Local access is always allowed regardless of token.
    if let Some(host) = req.headers().get("host").and_then(|h| h.to_str().ok()) {
        let bare = host.split(':').next().unwrap_or(host);
        if bare == "localhost" || bare == "127.0.0.1" {
            return next.run(req).await;
        }
    }

    // Valid session cookie — allow.
    if let Some(cookies) = req.headers().get("cookie").and_then(|v| v.to_str().ok()) {
        for part in cookies.split(';') {
            if let Some(val) = part.trim().strip_prefix("sdlc_auth=") {
                if val == token {
                    return next.run(req).await;
                }
            }
        }
    }

    // One-time bootstrap via `?auth=TOKEN` — set cookie and redirect.
    let uri = req.uri().clone();
    if let Some(query) = uri.query() {
        if let Some(val) = extract_auth_param(query) {
            if val == token {
                let destination = strip_auth_param(uri.path(), query);
                let cookie = format!("sdlc_auth={token}; HttpOnly; SameSite=Lax; Path=/");
                return Response::builder()
                    .status(302)
                    .header("Location", destination)
                    .header("Set-Cookie", cookie)
                    .body(Body::empty())
                    .expect("infallible: all header values are valid ASCII");
            }
        }
    }

    // Unauthorized — JSON for API routes, HTML for everything else.
    if req.uri().path().starts_with("/api/") {
        Response::builder()
            .status(401)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"unauthorized"}"#))
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
        Router::new()
            .route("/", get(ok_handler))
            .route("/api/state", get(ok_handler))
            .layer(middleware::from_fn_with_state(config, auth_middleware))
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
        assert_eq!(location, "/");
        let cookie = resp.headers().get("set-cookie").unwrap().to_str().unwrap();
        assert!(cookie.contains("sdlc_auth=secret"));
        assert!(cookie.contains("HttpOnly"));
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
