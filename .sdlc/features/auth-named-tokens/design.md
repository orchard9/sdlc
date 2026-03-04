# Design: auth-named-tokens

## Architecture Overview

The feature spans four layers:

1. **Data layer** (`sdlc-core`) — `AuthConfig` struct, CRUD helpers, `auth.yaml` serialization
2. **CLI** (`sdlc-cli`) — `sdlc auth token {add,list,remove}` subcommands
3. **Server** (`sdlc-server`) — REST routes, updated middleware, `auth.yaml` hot-reload watcher
4. **Frontend** — "Tunnel Access" panel in the Settings page

---

## Data Layer: `sdlc-core/src/auth_config.rs`

### Schema

```yaml
# .sdlc/auth.yaml
tokens:
  - name: jordan
    token: xK7mPqR2
    created_at: "2026-03-03T21:00:00Z"
  - name: ci-bot
    token: bNv4wHtL
    created_at: "2026-03-03T21:05:00Z"
```

### Structs

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub tokens: Vec<NamedToken>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamedToken {
    pub name: String,
    pub token: String,
    pub created_at: String,  // RFC 3339
}
```

### Functions

```rust
// Load .sdlc/auth.yaml; returns empty AuthConfig if file missing (no-auth mode)
pub fn load(root: &Path) -> Result<AuthConfig, SdlcError>

// Atomically write .sdlc/auth.yaml
pub fn save(root: &Path, config: &AuthConfig) -> Result<(), SdlcError>

// Generate a random 8-char alphanumeric token (reuses tunnel::generate_token pattern)
pub fn generate_token() -> String

// Add a named token; errors if name already exists
pub fn add_token(root: &Path, name: &str) -> Result<String, SdlcError>

// Remove a named token by name; errors if not found
pub fn remove_token(root: &Path, name: &str) -> Result<(), SdlcError>
```

File location: `.sdlc/auth.yaml` (uses `sdlc_core::io::atomic_write`).

---

## Auth Middleware Updates (`sdlc-server/src/auth.rs`)

### TunnelConfig changes

```rust
#[derive(Clone, Debug)]
pub struct TunnelConfig {
    /// Named tokens. Empty = no-auth mode (all requests pass through).
    pub tokens: Vec<(String, String)>,  // (name, token_value)
    pub app_tunnel_host: Option<String>,
}
```

The old `Option<String> token` field is replaced with `Vec<(String, String)>`. The `none()` constructor returns an empty `tokens` vec. `with_token(token)` adds a single unnamed entry `("default", token)` for backward compatibility during the tunnel start flow (until the named-token path is the primary path).

### Auth flow additions

After cookie check, before the 401 fallback, add:

```
7. Authorization: Bearer <TOKEN> header matches any token → allow
```

The multi-token check iterates `tokens` and returns `true` if any `token_value` matches.

### Hot-reload watcher

A new watcher task (8th watcher, added to `new_with_port`) monitors `.sdlc/auth.yaml` mtime at 800 ms intervals. On change, it reloads `AuthConfig` and atomically updates `tunnel_snapshot.config.tokens`.

Watcher skeleton:
```rust
let auth_file = state.root.join(".sdlc").join("auth.yaml");
let snap_write = state.tunnel_snapshot.clone();
let root_reload = state.root.clone();
handles.push(tokio::spawn(async move {
    let mut last_mtime = None::<std::time::SystemTime>;
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        if let Ok(meta) = tokio::fs::metadata(&auth_file).await {
            if let Ok(mtime) = meta.modified() {
                if last_mtime != Some(mtime) {
                    last_mtime = Some(mtime);
                    // Reload auth.yaml and update snapshot
                    let tokens = sdlc_core::auth_config::load(&root_reload)
                        .map(|c| c.tokens.into_iter().map(|t| (t.name, t.token)).collect())
                        .unwrap_or_default();
                    let mut snap = snap_write.write().await;
                    snap.config.tokens = tokens;
                }
            }
        }
    }
}).abort_handle());
```

---

## CLI: `sdlc auth token {add,list,remove}`

### Command structure

New top-level command `sdlc auth` with subcommand `token`:

```
sdlc auth token add <name>     # Generate token, write to auth.yaml, print auth URL
sdlc auth token list           # Table: name | created_at | token (first 4 chars)***
sdlc auth token remove <name>  # Remove entry from auth.yaml
```

Output of `add`:
```
Added token 'jordan'

  Token:    xK7mPqR2
  Auth URL: http://localhost:3141/?auth=xK7mPqR2

Share the token or URL to grant access when the tunnel is active.
Token stored in .sdlc/auth.yaml
```

Output of `list`:
```
NAME      CREATED               TOKEN
jordan    2026-03-03 21:00:00   xK7m****
ci-bot    2026-03-03 21:05:00   bNv4****
```

---

## REST API (`sdlc-server/src/routes/auth_tokens.rs`)

### Routes

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/auth/tokens` | List all named tokens (name + created_at; no token values) |
| `POST` | `/api/auth/tokens` | Create a named token; returns token value (once only) |
| `DELETE` | `/api/auth/tokens/:name` | Revoke a named token |

### Request/response shapes

**GET /api/auth/tokens**
```json
[
  { "name": "jordan", "created_at": "2026-03-03T21:00:00Z" },
  { "name": "ci-bot", "created_at": "2026-03-03T21:05:00Z" }
]
```

**POST /api/auth/tokens** body: `{ "name": "reviewer" }`
Response 201:
```json
{ "name": "reviewer", "token": "xK7mPqR2", "created_at": "2026-03-03T21:10:00Z" }
```

**DELETE /api/auth/tokens/:name** → 200 `{ "status": "removed" }` or 404.

---

## Frontend: Tunnel Access Panel

A new section in the Settings page (or a new dedicated page at `/settings`) shows the named token list. Since the Settings page does not currently exist as a dedicated route, the token panel will be added to the SecretsPage as a new top-level section "Tunnel Access" following the same card/row pattern as Authorized Keys.

[Mockup](mockup.html)

### UI components

- **Token list card**: shows rows of `name | created_at | revoke button`
- **Add Token modal**: single text field for `name`, on submit calls `POST /api/auth/tokens`, shows the generated token value once with a copy button and a clear warning ("This token will not be shown again")
- **No-tokens empty state**: "No tunnel access tokens. Add one to require authentication when sharing your UI over a tunnel."
- **SSE integration**: refreshes the token list on any SSE `Update` event (the `auth.yaml` watcher fires `SseMessage::Update`)

---

## Startup behavior

On server startup (`AppState::new_with_port`), if `.sdlc/auth.yaml` exists and is non-empty, `tunnel_snapshot.config.tokens` is pre-populated from it. This ensures that a restart with `auth.yaml` in place does not briefly allow unauthenticated access before the first watcher tick.

Pre-population happens in `build_base_state`:
```rust
let auth_tokens = sdlc_core::auth_config::load(&root)
    .map(|c| c.tokens.into_iter().map(|t| (t.name, t.token)).collect::<Vec<_>>())
    .unwrap_or_default();
// ... then set tunnel_snapshot.config.tokens = auth_tokens
```

---

## Backward compatibility

- No `auth.yaml` present → `tokens` vec is empty → middleware is a no-op (existing behavior).
- Tunnel-generated tokens (`POST /api/tunnel` → `generate_token()`) continue to work. When a tunnel starts, the generated token is added to the in-memory `TunnelConfig.tokens` as `("_tunnel", token)` but is NOT written to `auth.yaml` (ephemeral, same as today).
- Existing cookie-based sessions (`sdlc_auth=<token>`) remain valid for all tokens in the list.

---

## File map

| File | Change |
|---|---|
| `crates/sdlc-core/src/auth_config.rs` | New — AuthConfig, NamedToken, CRUD |
| `crates/sdlc-core/src/lib.rs` | Add `pub mod auth_config;` |
| `crates/sdlc-core/src/error.rs` | Add `AuthTokenExists`, `AuthTokenNotFound` error variants |
| `crates/sdlc-server/src/auth.rs` | Replace `Option<String> token` with `Vec<(String,String)> tokens`; add Bearer check |
| `crates/sdlc-server/src/state.rs` | Add 8th watcher for `auth.yaml`; pre-populate tokens on startup |
| `crates/sdlc-server/src/routes/auth_tokens.rs` | New — 3 handlers |
| `crates/sdlc-server/src/routes/mod.rs` | Register new routes |
| `crates/sdlc-server/src/lib.rs` | Mount `/api/auth/tokens` router |
| `crates/sdlc-cli/src/cmd/auth.rs` | New — `sdlc auth token add/list/remove` |
| `crates/sdlc-cli/src/main.rs` | Register `Auth` command |
| `frontend/src/pages/SecretsPage.tsx` | Add Tunnel Access section |
| `frontend/src/api/client.ts` | Add `getAuthTokens`, `addAuthToken`, `removeAuthToken` |
| `frontend/src/lib/types.ts` | Add `AuthToken` type |
