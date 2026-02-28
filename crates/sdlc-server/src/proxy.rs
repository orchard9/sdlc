//! Reverse-proxy handler for the app tunnel.
//!
//! When cloudflared tunnels to sdlc-server and the `Host` header matches the
//! active app tunnel hostname, this handler proxies the request to the user's
//! local dev server and injects a feedback FAB widget into HTML responses.

use axum::{
    body::Body,
    extract::{Request, State},
    http::Uri,
    response::Response,
};
use bytes::Bytes;
use futures::StreamExt;

use crate::{embed, state::AppState};

// ---------------------------------------------------------------------------
// Hop-by-hop headers — must not be forwarded in either direction.
// ---------------------------------------------------------------------------

const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "proxy-connection",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

// ---------------------------------------------------------------------------
// Feedback FAB widget — injected as an inline <script> before </body>.
// ---------------------------------------------------------------------------

const WIDGET_JS: &str = r#"(function () {
  if (window.__sdlcFabLoaded) return;
  window.__sdlcFabLoaded = true;

  var style = document.createElement('style');
  style.textContent = [
    '#__sdlc-fab{position:fixed;bottom:20px;right:20px;z-index:2147483647;font-family:sans-serif}',
    '#__sdlc-btn{width:48px;height:48px;border-radius:50%;background:#6366f1;border:none;cursor:pointer;',
    'box-shadow:0 4px 12px rgba(0,0,0,.3);color:#fff;font-size:22px;display:flex;align-items:center;',
    'justify-content:center;transition:transform .15s}',
    '#__sdlc-btn:hover{transform:scale(1.1)}',
    '#__sdlc-panel{display:none;position:absolute;bottom:58px;right:0;width:280px;background:#1e1e2e;',
    'border:1px solid #44475a;border-radius:12px;padding:14px;box-shadow:0 8px 24px rgba(0,0,0,.5)}',
    '#__sdlc-panel.open{display:block}',
    '#__sdlc-textarea{width:100%;box-sizing:border-box;height:90px;background:#282a36;color:#f8f8f2;',
    'border:1px solid #44475a;border-radius:8px;padding:8px;font-size:13px;resize:none;outline:none}',
    '#__sdlc-submit{margin-top:8px;width:100%;padding:7px;background:#6366f1;color:#fff;border:none;',
    'border-radius:8px;font-size:13px;cursor:pointer;font-weight:600}',
    '#__sdlc-submit:hover{background:#4f46e5}',
    '#__sdlc-toast{margin-top:8px;text-align:center;font-size:12px;color:#50fa7b;display:none}',
  ].join('');
  document.head.appendChild(style);

  var fab = document.createElement('div');
  fab.id = '__sdlc-fab';
  fab.innerHTML = [
    '<button id="__sdlc-btn" title="Leave feedback">+</button>',
    '<div id="__sdlc-panel">',
    '<textarea id="__sdlc-textarea" placeholder="Describe your feedback…"></textarea>',
    '<button id="__sdlc-submit">Submit</button>',
    '<div id="__sdlc-toast">✓ Saved!</div>',
    '</div>',
  ].join('');
  document.body.appendChild(fab);

  var btn = document.getElementById('__sdlc-btn');
  var panel = document.getElementById('__sdlc-panel');
  var textarea = document.getElementById('__sdlc-textarea');
  var submit = document.getElementById('__sdlc-submit');
  var toast = document.getElementById('__sdlc-toast');

  btn.addEventListener('click', function () {
    panel.classList.toggle('open');
    if (panel.classList.contains('open')) textarea.focus();
  });

  submit.addEventListener('click', function () {
    var content = ('[page: ' + location.href + ']\n' + textarea.value).trim();
    if (!content) return;
    submit.disabled = true;
    fetch('/__sdlc/feedback', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content: content }),
    })
      .then(function () {
        textarea.value = '';
        toast.style.display = 'block';
        setTimeout(function () {
          toast.style.display = 'none';
          panel.classList.remove('open');
          submit.disabled = false;
        }, 1500);
      })
      .catch(function () {
        submit.disabled = false;
        alert('Failed to save feedback — please try again.');
      });
  });
})();"#;

// ---------------------------------------------------------------------------
// Public handler
// ---------------------------------------------------------------------------

/// Fallback handler: proxy to user's app if Host matches app_tunnel_host,
/// otherwise serve the embedded SPA.
pub async fn proxy_handler(State(app): State<AppState>, req: Request) -> Response {
    // Extract the Host header (strip port if present).
    let host_value = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bare_host = host_value
        .split(':')
        .next()
        .unwrap_or(&host_value)
        .to_string();

    // Check whether this request is destined for the app tunnel.
    let app_tunnel_host = app.tunnel_config.read().await.app_tunnel_host.clone();
    let is_app_tunnel = app_tunnel_host.as_deref().is_some_and(|h| h == bare_host);

    if !is_app_tunnel {
        // Not an app tunnel request — serve the embedded SPA.
        return embed::static_handler(req.uri().clone()).await;
    }

    // Resolve the upstream port.
    let user_port = match *app.app_tunnel_port.read().await {
        Some(p) => p,
        None => {
            return Response::builder()
                .status(502)
                .header("Content-Type", "text/plain")
                .body(Body::from("No app tunnel port configured"))
                .expect("infallible");
        }
    };

    // Build upstream URL.
    let upstream_url = build_upstream_uri(user_port, req.uri());

    // Convert axum Request into a reqwest RequestBuilder.
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .unwrap_or(reqwest::Method::GET);

    // Collect request headers, stripping hop-by-hop, host, and accept-encoding.
    // We strip accept-encoding so the upstream always returns uncompressed content;
    // this lets inject_widget operate on plain text rather than compressed bytes.
    let mut req_headers = reqwest::header::HeaderMap::new();
    for (name, value) in req.headers() {
        let lower = name.as_str().to_ascii_lowercase();
        if lower == "host" || lower == "accept-encoding" || HOP_BY_HOP.contains(&lower.as_str()) {
            continue;
        }
        if let Ok(v) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
            if let Ok(n) = reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()) {
                req_headers.insert(n, v);
            }
        }
    }

    // Buffer request body (up to 10 MB).
    let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return Response::builder()
                .status(400)
                .body(Body::empty())
                .expect("infallible");
        }
    };

    // Send to upstream.
    let upstream_resp = match app
        .http_client
        .request(method, &upstream_url)
        .headers(req_headers)
        .body(body_bytes.to_vec())
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return Response::builder()
                .status(502)
                .header("Content-Type", "text/plain")
                .body(Body::from("Could not reach app dev server"))
                .expect("infallible");
        }
    };

    // Build response: copy status + headers.
    let status = axum::http::StatusCode::from_u16(upstream_resp.status().as_u16())
        .unwrap_or(axum::http::StatusCode::BAD_GATEWAY);

    let mut builder = Response::builder().status(status);

    let content_type = upstream_resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    for (name, value) in upstream_resp.headers() {
        let lower = name.as_str().to_ascii_lowercase();
        // Strip hop-by-hop; also strip content-length/content-encoding when we
        // inject into HTML (length changes).
        if HOP_BY_HOP.contains(&lower.as_str()) {
            continue;
        }
        let is_html = content_type.contains("text/html");
        if is_html && (lower == "content-length" || lower == "content-encoding") {
            continue;
        }
        if let Ok(v) = axum::http::HeaderValue::from_bytes(value.as_bytes()) {
            builder = builder.header(name.as_str(), v);
        }
    }

    // For HTML responses: buffer and inject widget.
    if content_type.contains("text/html") {
        let body_bytes = match upstream_resp.bytes().await {
            Ok(b) => b,
            Err(_) => {
                return Response::builder()
                    .status(502)
                    .body(Body::empty())
                    .expect("infallible");
            }
        };
        let injected = inject_widget(body_bytes);
        builder.body(Body::from(injected)).expect("infallible")
    } else {
        // Stream non-HTML responses without buffering.
        let stream = upstream_resp
            .bytes_stream()
            .map(|chunk| chunk.map_err(std::io::Error::other));
        builder.body(Body::from_stream(stream)).expect("infallible")
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Inject the feedback widget `<script>` immediately before `</body>`.
/// If the response is not valid UTF-8 or has no `</body>`, append at end.
pub fn inject_widget(body: Bytes) -> Vec<u8> {
    let html = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return body.to_vec(),
    };

    let script = format!("<script>{WIDGET_JS}</script>");
    if let Some(pos) = html.rfind("</body>") {
        let mut out = String::with_capacity(html.len() + script.len());
        out.push_str(&html[..pos]);
        out.push_str(&script);
        out.push_str(&html[pos..]);
        out.into_bytes()
    } else {
        // No </body> tag — append at end (handles HTML fragments).
        let mut out = html.to_string();
        out.push_str(&script);
        out.into_bytes()
    }
}

/// Build `http://127.0.0.1:{port}{path_and_query}`.
fn build_upstream_uri(port: u16, uri: &Uri) -> String {
    let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
    format!("http://127.0.0.1:{port}{path_and_query}")
}

/// Extract the hostname from a full URL (strips scheme and path).
pub fn extract_host_from_url(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_widget_before_body_close() {
        let html = Bytes::from("<html><body><p>Hello</p></body></html>");
        let result = String::from_utf8(inject_widget(html)).unwrap();
        assert!(result.contains("<script>"));
        assert!(result.contains("</body>"));
        // Script comes before </body>
        let script_pos = result.find("<script>").unwrap();
        let body_pos = result.rfind("</body>").unwrap();
        assert!(script_pos < body_pos);
    }

    #[test]
    fn inject_widget_no_body_tag_appends_at_end() {
        let html = Bytes::from("<p>Fragment</p>");
        let result = String::from_utf8(inject_widget(html)).unwrap();
        assert!(result.ends_with("</script>"));
    }

    #[test]
    fn inject_widget_invalid_utf8_passthrough() {
        let bytes = Bytes::from(vec![0xFF, 0xFE, 0x00]);
        let result = inject_widget(bytes.clone());
        assert_eq!(result, bytes.to_vec());
    }

    #[test]
    fn extract_host_from_https_url() {
        assert_eq!(
            extract_host_from_url("https://fancy-rabbit.trycloudflare.com/some/path"),
            "fancy-rabbit.trycloudflare.com"
        );
    }

    #[test]
    fn extract_host_from_http_url() {
        assert_eq!(
            extract_host_from_url("http://localhost:3000/"),
            "localhost:3000"
        );
    }

    #[test]
    fn build_upstream_uri_with_path_and_query() {
        let uri: Uri = "/foo/bar?baz=1".parse().unwrap();
        assert_eq!(
            build_upstream_uri(3000, &uri),
            "http://127.0.0.1:3000/foo/bar?baz=1"
        );
    }

    #[test]
    fn build_upstream_uri_root() {
        let uri: Uri = "/".parse().unwrap();
        assert_eq!(build_upstream_uri(3000, &uri), "http://127.0.0.1:3000/");
    }
}
