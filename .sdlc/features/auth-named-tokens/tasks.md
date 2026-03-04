# Tasks: auth-named-tokens

## T1 — Add AuthConfig data layer to sdlc-core

Create `crates/sdlc-core/src/auth_config.rs` with:
- `AuthConfig` struct with `tokens: Vec<NamedToken>`
- `NamedToken` struct with `name`, `token`, `created_at` fields
- `load(root: &Path) -> Result<AuthConfig, SdlcError>` — returns empty config if file missing
- `save(root: &Path, config: &AuthConfig) -> Result<(), SdlcError>` — atomic write
- `generate_token() -> String` — 8-char alphanumeric
- `add_token(root: &Path, name: &str) -> Result<String, SdlcError>` — errors if name exists
- `remove_token(root: &Path, name: &str) -> Result<(), SdlcError>` — errors if name not found

Add `pub mod auth_config;` to `crates/sdlc-core/src/lib.rs`.

Add `AuthTokenExists(String)` and `AuthTokenNotFound(String)` variants to `crates/sdlc-core/src/error.rs`.

## T2 — Update TunnelConfig to hold named token vec

In `crates/sdlc-server/src/auth.rs`:
- Replace `pub token: Option<String>` with `pub tokens: Vec<(String, String)>` (name, value pairs)
- Update `none()` to return empty `tokens` vec
- Add `with_tokens(tokens: Vec<(String, String)>)` constructor
- Keep `with_token(token: String)` as backward-compat shim adding `("_tunnel", token)`
- Update auth middleware: iterate `tokens` to find a match (cookie check, Bearer header check, query param check)
- Add `Authorization: Bearer <TOKEN>` check after cookie check (step 6.5 in flow)
- Update all unit tests in auth.rs to use new `tokens` field

## T3 — Pre-populate tokens on startup and add auth.yaml hot-reload watcher

In `crates/sdlc-server/src/state.rs`:
- In `build_base_state`: load `auth_config::load(&root)` and set `tunnel_snapshot.config.tokens` from it
- In `new_with_port`: add 8th watcher for `.sdlc/auth.yaml` mtime (800 ms interval, same pattern as other watchers)
  - On change: reload `AuthConfig`, update `tunnel_snapshot.config.tokens` atomically
- Update watcher count in tracing log from "7" to "8"

## T4 — Add REST routes for named tokens

Create `crates/sdlc-server/src/routes/auth_tokens.rs` with:
- `GET /api/auth/tokens` — list tokens (name + created_at only; no token values)
- `POST /api/auth/tokens` — create a named token; returns token value (one-time); body: `{ "name": string }`
- `DELETE /api/auth/tokens/:name` — revoke by name; 404 if not found

Register in `crates/sdlc-server/src/routes/mod.rs` and mount in `crates/sdlc-server/src/lib.rs`.

Write unit tests for all three handlers.

## T5 — Add sdlc auth CLI commands

Create `crates/sdlc-cli/src/cmd/auth.rs` with:
- `sdlc auth token add <name>` — calls `auth_config::add_token`, prints token value and auth URL
- `sdlc auth token list` — loads `auth.yaml`, prints table (name | created_at | token prefix****)
- `sdlc auth token remove <name>` — calls `auth_config::remove_token`, confirms removal

Register `Auth` command in `crates/sdlc-cli/src/main.rs`.

## T6 — Frontend: Tunnel Access panel in SecretsPage

In `frontend/src/lib/types.ts`, add:
```ts
export interface AuthToken {
  name: string
  created_at: string
}
```

In `frontend/src/api/client.ts`, add:
- `getAuthTokens(): Promise<AuthToken[]>`
- `addAuthToken(name: string): Promise<{ name: string; token: string; created_at: string }>`
- `removeAuthToken(name: string): Promise<void>`

In `frontend/src/pages/SecretsPage.tsx`:
- Add `TunnelAccessSection` component (card with token rows, add/revoke buttons)
- Add `AddTokenModal` component (name input, submit → show token reveal state, copy button, "not shown again" warning)
- Load tokens alongside keys/envs in `refresh()`
- Place the Tunnel Access card above Authorized Keys

## T7 — Tests: auth middleware multi-token unit tests

In `crates/sdlc-server/src/auth.rs` test module, add:
- `multi_token_first_token_passes` — two tokens in config, cookie matches first
- `multi_token_second_token_passes` — two tokens in config, cookie matches second
- `bearer_header_passes_auth` — `Authorization: Bearer <token>` header accepted
- `bearer_header_wrong_token_401` — wrong bearer token → 401
- `empty_tokens_passthrough` — empty tokens vec → no auth required (open mode)
