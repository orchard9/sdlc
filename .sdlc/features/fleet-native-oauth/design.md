# Design: fleet-native-oauth

## File Changes

### New Files

**`crates/sdlc-server/src/oauth.rs`** — Core module (~200 lines)

```rust
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub allowed_domains: Vec<String>,
    pub session_secret: Vec<u8>,  // HMAC key
    pub redirect_uri: Option<String>,
}

pub struct SessionPayload {
    pub email: String,
    pub name: String,
    pub exp: i64,
}
```

Functions:
- `build_authorize_url(config, state) -> String`
- `exchange_code(config, http_client, code) -> Result<TokenResponse>`
- `fetch_userinfo(http_client, access_token) -> Result<UserInfo>`
- `sign_session(config, payload) -> String` (base64 + HMAC)
- `verify_session(config, cookie_value) -> Option<SessionPayload>`
- Route handlers: `login`, `callback`, `verify`, `logout`

### Modified Files

**`crates/sdlc-server/src/lib.rs`**
- Register `/auth/login`, `/auth/callback`, `/auth/verify`, `/auth/logout` routes
- These routes MUST be registered before the auth middleware layer (lines 700-706)
- Actually: register them on a nested router that does NOT have the auth layer

**`crates/sdlc-server/src/state.rs`**
- Add `pub oauth_config: Option<OAuthConfig>` to `AppState`
- Initialize from env vars in `new_with_port_hub`

**`crates/sdlc-server/Cargo.toml`**
- Add `oauth2 = "4"` dependency
- Add `hmac = "0.12"` and `sha2 = "0.10"` for cookie signing
- Add `base64 = "0.22"` for cookie encoding

### Helm Chart

**`k3s-fleet/deployments/helm/sdlc-server/templates/middleware-google-auth.yaml`**
- Change forwardAuth address from `oauth2-proxy.{{ .Values.auth.proxyNamespace }}` to:
  `http://sdlc-hub.sdlc-hub.svc.cluster.local:8080/auth/verify`

**`k3s-fleet/deployments/helm/sdlc-server/values.yaml`**
- Remove `auth.proxyNamespace` (no longer needed)
- Keep `auth.enabled` toggle

## Route Registration Strategy

```rust
// Public auth routes — no auth middleware
let auth_routes = Router::new()
    .route("/auth/login", get(oauth::login))
    .route("/auth/callback", get(oauth::callback))
    .route("/auth/verify", get(oauth::verify))
    .route("/auth/logout", post(oauth::logout));

// Main router with auth middleware
let app = Router::new()
    .merge(auth_routes)  // ← before auth layer
    .route("/api/health", get(...))
    // ... all other routes ...
    .layer(auth_middleware)
```

The key insight: `/auth/login` and `/auth/callback` must be accessible without any session. They are the routes that CREATE sessions. So they go on a separate router that merges before the auth middleware layer.

## Cookie Flow Diagram

```
Browser                    Traefik                    Hub (/auth)
   |                         |                           |
   |-- GET *.sdlc.threesix --+                           |
   |                         |-- forwardAuth /auth/verify |
   |                         |                           |-- check cookie
   |                         |<-- 401 --------------------|
   |<-- 302 /auth/login -----|                           |
   |                         |                           |
   |-- GET /auth/login ------+-------------------------->|
   |<-- 302 Google -----------+---------<----------------|
   |                         |                           |
   |-- (Google auth flow) ---|                           |
   |                         |                           |
   |-- GET /auth/callback ---+-------------------------->|
   |                         |                  exchange code, verify domain
   |<-- 302 / + Set-Cookie --+---------<----------------|
   |                         |                           |
   |-- GET *.sdlc.threesix --+  (cookie on .sdlc.threesix.ai)
   |                         |-- forwardAuth /auth/verify |
   |                         |                           |-- check cookie ✓
   |                         |<-- 200 + X-Auth-User -----|
   |<-- (proxied response) --|                           |
```

## Test Plan

1. `login` handler builds correct Google URL with all params
2. `callback` rejects email from disallowed domain → 403
3. `callback` accepts email from allowed domain → sets cookie, 302
4. `verify` with valid unexpired cookie → 200 + X-Auth-User header
5. `verify` with expired cookie → 401
6. `verify` with missing cookie → 401
7. `verify` with tampered cookie (bad HMAC) → 401
8. `verify` with valid Bearer token → 200 (M2M path)
9. `logout` clears cookie
10. Integration: existing `auth.rs` Bearer tokens still work for `/api/*`
