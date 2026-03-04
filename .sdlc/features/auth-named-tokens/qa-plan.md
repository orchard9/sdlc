# QA Plan: auth-named-tokens

## Automated tests (cargo test)

### 1. AuthConfig data layer unit tests (`sdlc-core`)

| Test | Assertion |
|---|---|
| `load_returns_empty_when_no_file` | `auth_config::load(root)` returns `AuthConfig { tokens: [] }` when `auth.yaml` absent |
| `add_token_writes_file` | `add_token(root, "jordan")` creates `auth.yaml` with one entry; returned token is 8 alphanumeric chars |
| `add_token_duplicate_name_errors` | `add_token(root, "jordan")` twice returns `Err(AuthTokenExists("jordan"))` |
| `remove_token_removes_entry` | After add, `remove_token(root, "jordan")` leaves empty tokens list |
| `remove_token_not_found_errors` | `remove_token(root, "ghost")` returns `Err(AuthTokenNotFound("ghost"))` |
| `generate_token_is_8_alphanumeric` | `generate_token()` returns exactly 8 ASCII alphanumeric characters |
| `save_and_load_roundtrip` | `save` then `load` produces identical `AuthConfig` |

### 2. Auth middleware unit tests (`sdlc-server`)

| Test | Assertion |
|---|---|
| `empty_tokens_passthrough` | `TunnelConfig { tokens: vec![] }` → all requests pass (no-auth mode) |
| `multi_token_first_token_cookie_passes` | Two tokens; cookie matches first → 200 |
| `multi_token_second_token_cookie_passes` | Two tokens; cookie matches second → 200 |
| `wrong_token_returns_401` | Valid config but wrong token value → 401 |
| `bearer_header_passes_auth` | `Authorization: Bearer <token>` with valid token → 200 |
| `bearer_header_wrong_token_401` | `Authorization: Bearer wrongtoken` → 401 |
| `bearer_api_path_401_json` | Invalid bearer on `/api/*` → 401 with `Content-Type: application/json` |
| `query_param_auth_any_named_token` | `?auth=<second-token>` with two-token config → 302 + Set-Cookie |
| `localhost_always_bypasses_multi_token` | localhost host header → 200 regardless of tokens |

### 3. REST route unit tests (`sdlc-server`)

| Test | Assertion |
|---|---|
| `get_tokens_empty_when_no_file` | `GET /api/auth/tokens` returns `[]` with no `auth.yaml` |
| `post_token_creates_entry` | `POST /api/auth/tokens { "name": "alice" }` → 201; response includes `name`, `token`, `created_at`; token is 8 chars |
| `post_token_duplicate_returns_409` | Second `POST` with same name → 409 |
| `delete_token_removes_entry` | After create, `DELETE /api/auth/tokens/alice` → 200; subsequent GET returns `[]` |
| `delete_token_not_found_returns_404` | `DELETE /api/auth/tokens/ghost` → 404 |
| `get_tokens_does_not_return_token_values` | GET response objects contain `name` and `created_at` but NOT `token` |

### 4. Integration test: startup pre-population

| Test | Assertion |
|---|---|
| `startup_loads_auth_yaml_into_tunnel_config` | Write `auth.yaml` to temp dir before `AppState::new()`; verify `tunnel_snapshot.config.tokens` is non-empty on startup |

## CLI smoke tests (manual / `cargo run`)

Run with `SDLC_NO_NPM=1 cargo build --all` first.

```bash
# Add a token
sdlc auth token add jordan
# Expected: prints token value, auth URL; exits 0

# List tokens
sdlc auth token list
# Expected: table shows "jordan" row with masked token

# Duplicate name
sdlc auth token add jordan
# Expected: error message, exits 1

# Remove token
sdlc auth token remove jordan
# Expected: confirmation message; exits 0

# Remove nonexistent
sdlc auth token remove ghost
# Expected: error message, exits 1
```

## Build verification

```bash
SDLC_NO_NPM=1 cargo build --all 2>&1 | grep -E "^error"  # must be empty
SDLC_NO_NPM=1 cargo test --all 2>&1 | tail -5             # must show "test result: ok"
cargo clippy --all -- -D warnings 2>&1 | grep "^error"    # must be empty
```

## Frontend smoke test (manual)

1. Start server with at least one token in `auth.yaml`.
2. Navigate to Secrets page — verify "Tunnel Access" card appears with existing token row.
3. Click "Add Token", enter a name, click Create — verify token reveal modal with copy button.
4. Dismiss modal — verify new token row appears in list.
5. Click revoke button on a token — verify row disappears.
6. Open browser Network tab: verify `GET /api/auth/tokens` response does NOT include `token` field in objects.

## Backward-compatibility check

1. Delete `.sdlc/auth.yaml` (or ensure it doesn't exist).
2. Start server — verify it starts in open mode (no auth required, all requests pass through).
3. Start tunnel (`POST /api/tunnel`) — verify that adding the tunnel ephemeral token still gates remote access.
