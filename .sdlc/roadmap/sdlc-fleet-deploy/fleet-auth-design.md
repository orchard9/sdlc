# Fleet Auth Design

## Context

The original fleet architecture declared "sdlc is FROZEN — zero Rust changes." That held
for the infrastructure work (Helm chart, Woodpecker pipelines, ESO, TLS). It does not hold
for auth. The fleet exposes `<slug>.sdlc.threesix.ai` on public DNS. Every pod is
unauthenticated today. Auth requires Rust changes to `sdlc-server`.

## Decision

**Google OAuth 2.0 with org restriction.** Extensible provider trait so GitHub, Gitea, or
others can be added later without touching middleware logic.

## Auth Mode Enum

Replace `TunnelConfig` with a proper `AuthMode`:

```rust
pub enum AuthMode {
    None,              // local — passthrough (existing behavior unchanged)
    Tunnel(String),    // tunnel token (existing behavior unchanged)
    OAuth(OAuthConfig) // cluster mode — new
}
```

Detection at startup: if `SDLC_AUTH_PROVIDER` env var is set → OAuth mode. Otherwise
existing logic is unchanged.

## Provider Trait (extensibility)

```rust
pub trait OAuthProvider: Send + Sync {
    fn name(&self) -> &str;
    fn authorization_url(&self, state: &str, redirect_uri: &str) -> String;
    async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<UserInfo>;
}

pub struct UserInfo {
    pub email: String,
    pub name: String,     // e.g. "Jordan Washburn"
    pub provider: String, // e.g. "google"
}
```

Adding GitHub or Gitea later = implement the trait, register it. No middleware changes.

## Google Provider — Org Restriction

Google's `hd` (hosted domain) parameter gates authentication to a single Google Workspace
org. Set `hd=orchard9.io` (or equivalent) in the authorization URL. Google enforces it;
server also validates the `hd` claim from userinfo on callback.

## Config (env vars → Helm Secret)

```
SDLC_AUTH_PROVIDER=google
SDLC_AUTH_CLIENT_ID=...
SDLC_AUTH_CLIENT_SECRET=...
SDLC_AUTH_ALLOWED_DOMAIN=orchard9.io
SDLC_AUTH_SESSION_SECRET=...   # 32-byte HMAC key
SDLC_AUTH_BASE_URL=https://myproject.sdlc.threesix.ai
```

Helm chart injects these from a Kubernetes Secret per project namespace.

## Session — Stateless

Signed cookie: `base64(json({ email, name, provider, exp })).hmac_sig`

No Redis, no DB. HMAC validates on every request. 7-day expiry. Cookie name: `sdlc_session`.

## Routes

```
GET /auth/login     → build Google auth URL with state param + hd, redirect
GET /auth/callback  → exchange code, validate hd claim, set cookie, redirect to /
GET /auth/logout    → clear cookie, redirect to /auth/login
```

## Agent Bypass

Agents send `Authorization: Bearer <token>` header. Token set via `SDLC_AGENT_TOKEN` env
var in the Helm chart. Middleware grants access and stamps `agent` as the identity.
Humans get the cookie flow; agents get the bearer token. Same middleware, two paths.

## Identity Payoff

`approved_by` on every artifact becomes real in cluster mode. Currently null everywhere.
With OAuth, every `sdlc artifact approve` in a cluster pod carries the authenticated email.

## What Changes in Rust

- `crates/sdlc-server/src/auth.rs` — new `AuthMode` enum, OAuth middleware path, session validation
- New route handlers: `/auth/login`, `/auth/callback`, `/auth/logout`
- `crates/sdlc-server/src/state.rs` — carry identity through request context
- `crates/sdlc-core/src/feature.rs` — pass identity to `approved_by` on artifact approval
- `crates/sdlc-server/src/lib.rs` — read env vars at startup, select auth mode

## What Changes in Helm

- `values.yaml` — new `auth:` block with `provider`, `allowedDomain`, `baseUrl`
- New Kubernetes Secret template for OAuth credentials
- `deployment.yaml` — inject auth env vars from Secret

## Open Question

What is the orchard9 Google Workspace domain? (needed for `SDLC_AUTH_ALLOWED_DOMAIN`)
