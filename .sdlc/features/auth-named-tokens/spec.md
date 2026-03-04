# Spec: auth-named-tokens

## Problem

The current tunnel auth system uses a single shared opaque token generated at runtime and stored only in memory. This token:

1. Has no human-readable identity — the QR code/passcode output shows a random 8-character string with no context about who it belongs to.
2. Is ephemeral — every `sdlc ui --tunnel` restart generates a new token, invalidating all previously shared links.
3. Supports only one token — there is no way to grant access to multiple parties (e.g. a team member and a CI bot) with independently revocable credentials.
4. Has no persistence layer — there is no `auth.yaml` file that could be checked into version control or managed via `sdlc` CLI.

The result: sharing access to a running SDLC UI over a tunnel is friction-heavy (paste the passcode, resend on restart) and lacks the audit trail expected of an enterprise governance tool.

## Solution

Introduce a named-token auth system backed by `.sdlc/auth.yaml`:

- **Named tokens**: each token has a human-readable label (e.g. `jordan`, `ci-bot`, `reviewer`).
- **Persistent**: tokens are written to `.sdlc/auth.yaml` (safe to commit — it contains only token hashes or plaintext short tokens).
- **Hot-reload**: the server watches `.sdlc/auth.yaml` and reloads the token list without restart.
- **Optional auth mode**: when no tokens are configured AND no tunnel is active, the server operates in open mode (current behavior).
- **UI panel**: the Settings/Secrets area gets a "Tunnel Access" section showing active named tokens with add/revoke controls.
- **Multi-token validation**: any valid named token grants access; invalid or missing token → 401.

## Scope

### In scope

1. `.sdlc/auth.yaml` schema: `{ tokens: [{ name: string, token: string, created_at: ISO8601 }] }`.
2. `sdlc auth token add <name>` — generate a token, write to `auth.yaml`, print the auth URL.
3. `sdlc auth token list` — list all named tokens (name, created_at, short token preview).
4. `sdlc auth token remove <name>` — revoke a token by name.
5. Server hot-reload: watcher on `.sdlc/auth.yaml` updates the in-memory `TunnelConfig` token list.
6. `TunnelConfig` extended to hold a `Vec<(String, String)>` (name, token) instead of `Option<String>`.
7. Auth middleware updated to check `Bearer` header in addition to `sdlc_auth` cookie (for programmatic API clients).
8. `GET /api/auth/tokens` — list named tokens (name, created_at; no token values).
9. `POST /api/auth/tokens` — create a named token, returns the token value (once only).
10. `DELETE /api/auth/tokens/:name` — revoke by name.
11. Frontend "Tunnel Access" panel under Settings showing the token list with add/revoke UI.
12. SSE broadcast on `auth.yaml` change so the frontend refreshes without polling.

### Out of scope

- Token expiry / TTL (future feature).
- Scoped permissions (read-only vs read-write tokens).
- OAuth / external identity providers.
- Changing the QR-code bootstrap flow — `?auth=TOKEN` still works for any valid token.

## Constraints

- **Backward compatibility**: if `.sdlc/auth.yaml` does not exist, the server falls back to the existing single-token-in-memory behavior. No migration required.
- **No plaintext secrets in YAML at rest**: tokens stored in `auth.yaml` are the actual token strings (short, human-typeable). The file is `.gitignore`-able by default but may be committed by teams who accept that tradeoff — document this clearly.
- **Hot-reload must be atomic**: the watcher reads the file after a debounce; the auth middleware reads a `RwLock` snapshot updated atomically.
- **Production safety**: the watcher loop must have a sleep + bounded debounce (≥ 200 ms) to prevent CPU spin. Use the same mtime-comparison pattern already used for `state.yaml` and `changelog.yaml` watchers in `state.rs`.

## Acceptance Criteria

1. `sdlc auth token add jordan` writes `auth.yaml`, prints `?auth=<token>` URL, exits 0.
2. `sdlc auth token list` prints a table with name and creation date.
3. `sdlc auth token remove jordan` removes the entry from `auth.yaml`.
4. Running `sdlc ui` with `auth.yaml` present and non-empty enables auth gate automatically.
5. Adding a token via the UI panel while the server is running is reflected in auth without restart.
6. A request with `Authorization: Bearer <token>` passes auth (programmatic clients).
7. A request with `?auth=<token>` for any named token gets a session cookie and redirects.
8. A request with no valid token/cookie → 401.
9. Removing a token via `sdlc auth token remove` immediately invalidates it (next request after hot-reload fires → 401).
10. Tests: auth middleware unit tests cover multi-token matching, named-token lookup, and `Bearer` header.
