# App Tunnel Feedback FAB (Reverse HTTP Proxy)

cloudflared tunnels to `sdlc-server:3141`, which reverse-proxies to the user's local dev app and injects a feedback widget into every HTML response. Reviewers leave notes that flow into `.sdlc/feedback.yaml` and can be submitted as ponder entries.

## Entry Points

| Layer | File | Line | Purpose |
|---|---|---|---|
| Frontend UI | `frontend/src/pages/NetworkPage.tsx` | 204 | `AppTunnelSection` — port input, start/stop, feedback badge |
| Frontend UI | `frontend/src/pages/FeedbackPage.tsx` | 1 | View, delete, submit-to-ponder |
| API — App Tunnel | `crates/sdlc-server/src/routes/app_tunnel.rs` | 54 | `GET/POST/DELETE /api/app-tunnel` |
| API — Feedback | `crates/sdlc-server/src/routes/feedback.rs` | 14 | `GET/POST/DELETE /api/feedback`, `POST /api/feedback/to-ponder` |
| Public Widget | `crates/sdlc-server/src/routes/feedback.rs` | 38 | `POST /__sdlc/feedback` (auth-exempt alias) |
| Proxy Fallback | `crates/sdlc-server/src/proxy.rs` | 115 | `proxy_handler` — all non-matched routes |
| Auth Gate | `crates/sdlc-server/src/auth.rs` | 55 | `auth_middleware` — evaluates app tunnel host bypass |

## Execution Flows

### Tunnel Start
```
User → NetworkPage: port input + "Start" click
  → POST /api/app-tunnel { port: 3000 }
  → routes/app_tunnel.rs::start_app_tunnel()
    → Tunnel::start(sdlc_port)          # spawns cloudflared to sdlc-server:3141
    → extract_host_from_url()           # "fancy-rabbit.trycloudflare.com"
    → TunnelConfig.app_tunnel_host = Some(host)   # registers hostname in auth
    → AppState.app_tunnel_port = 3000             # stored for proxy use
    → persist_app_port() → .sdlc/config.yaml
  → Response: { active: true, url, configured_port }
```

### Proxy Request (reviewer)
```
https://fancy-rabbit.trycloudflare.com/page
  → cloudflared → sdlc-server:3141
  → auth_middleware:
      Host == app_tunnel_host AND path != /api/* → BYPASS AUTH
  → proxy_handler():
      Host matches app_tunnel_host → proxy mode
      upstream = http://127.0.0.1:3000/page
      reqwest::Client (AppState.http_client) → upstream
      Content-Type: text/html → inject_widget()
        → insert <script>WIDGET_JS</script> before </body>
      → return modified response
```

### Feedback Capture
```
Reviewer submits widget form
  → POST /__sdlc/feedback { content: "[page: URL]\nfeedback text" }
  → auth_middleware: path starts with /__sdlc/ → ALWAYS PUBLIC
  → routes/feedback.rs::add_note()
  → sdlc_core::feedback::add() → .sdlc/feedback.yaml (atomic write)
  → Widget shows "✓ Saved!" toast
```

### Feedback → Ponder
```
User on FeedbackPage → "Submit to Ponder"
  → POST /api/feedback/to-ponder
  → routes/feedback.rs::to_ponder()
    → unique_ponder_slug() → "feedback-20260228"
    → ponder::create() → .sdlc/roadmap/feedback-20260228/manifest.yaml
    → ponder::capture_content("notes.md") → formatted notes
    → sdlc_core::feedback::clear() → empties .sdlc/feedback.yaml
  → Response: { slug: "feedback-20260228", note_count: 3 }
  → Frontend: navigate /ponder/feedback-20260228
```

## Auth Rules (`auth.rs:55`, priority-ordered)

| Priority | Condition | Result |
|---|---|---|
| 1 | No tunnel token set | PASS |
| 2 | Host is localhost/127.0.0.1 | PASS |
| 3 | Path starts with `/__sdlc/` | PASS (always public) |
| 4 | Host == `app_tunnel_host` AND path ≠ `/api/*` | PASS (proxy bypass) |
| 5 | Valid `sdlc_auth` cookie | PASS |
| 6 | `?auth=TOKEN` query param | Set cookie + redirect |
| 7 | Everything else | 401 |

## Key Types

```rust
// auth.rs:16
pub struct TunnelConfig {
    pub token: Option<String>,
    pub app_tunnel_host: Option<String>,  // hostname bypass for proxy
}

// state.rs:201
pub struct AppState {
    pub app_tunnel_port: Arc<RwLock<Option<u16>>>,
    pub app_tunnel_handle: Arc<Mutex<Option<Tunnel>>>,
    pub app_tunnel_url: Arc<RwLock<Option<String>>>,
    pub http_client: reqwest::Client,  // no-redirect policy
}
```

```typescript
// frontend/src/lib/types.ts:403
interface FeedbackNote { id: string; content: string; created_at: string }
interface AppTunnelStatus { active: boolean; url?: string; configured_port?: number }
```

## Storage

| What | Where | Format |
|---|---|---|
| Feedback notes | `.sdlc/feedback.yaml` | YAML array; IDs sequential F1, F2, F3 (not reset on delete) |
| App port preference | `.sdlc/config.yaml` | `app_port: u16` |
| Submitted feedback | `.sdlc/roadmap/feedback-*/notes.md` | Markdown artifact in ponder entry |

## Router Registration (`lib.rs`)

```
POST  /__sdlc/feedback          → feedback::add_note   (public alias, auth-exempt)
GET   /api/feedback             → feedback::list_notes
POST  /api/feedback             → feedback::add_note
DELETE /api/feedback/{id}       → feedback::delete_note
POST  /api/feedback/to-ponder   → feedback::to_ponder
GET   /api/app-tunnel           → app_tunnel::get_app_tunnel
POST  /api/app-tunnel           → app_tunnel::start_app_tunnel
DELETE /api/app-tunnel          → app_tunnel::stop_app_tunnel
PUT   /api/app-tunnel/port      → app_tunnel::set_app_port
*     (fallback)                → proxy::proxy_handler
```

## Test Coverage

| File | Tests | Notes |
|---|---|---|
| `auth.rs` | 11 | Full matrix: no-token, localhost, `/__sdlc/`, app_tunnel_host bypass, cookie, query param, API blocking |
| `proxy.rs` | 7 | Widget injection, URL parsing, invalid UTF-8, upstream URI building |
| `routes/app_tunnel.rs` | 2 | Inactive state, stop-when-none error |
| `routes/feedback.rs` | 4 | List empty, add/list, delete missing, to_ponder with no notes |
| `tests/integration.rs` | 4 | Widget injection e2e, `/__sdlc/feedback` public, API blocked via app tunnel, SPA fallback |
| `sdlc-core/feedback.rs` | 6 | Add/list, sequential IDs, delete, ID persistence, clear, markdown format |

**Gap:** `start_app_tunnel()` / `stop_app_tunnel()` have no unit tests — require a live cloudflared process.

## Known Constraints

- Widget JS (`proxy.rs:38`) is a 70-line inline `const &str` — requires server recompile to change
- `inject_widget()` buffers the entire HTML body before injection — may add latency on large pages
- Widget FAB behavior on client-side-routed SPAs (after client navigation) is untested

## Dependencies

```toml
reqwest = { version = "0.12", features = ["rustls-tls", "stream"] }
bytes = "1"
```
