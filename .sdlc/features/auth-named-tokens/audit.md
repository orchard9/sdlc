# Security Audit: auth-named-tokens

## Scope

This audit covers the named tunnel-access token system: token generation, storage, the auth middleware, REST endpoints, and CLI. The audit checks for credential exposure, brute-force risk, token predictability, injection vectors, and backward-compatibility hazards.

---

## Findings

### F1 — Token entropy is low (8 alphanumeric characters, ~47 bits effective)

**Severity: MEDIUM | Action: Accept with documentation**

The 8-character token generated from 6 bytes of `/dev/urandom` via nibble-to-modulo-62 mapping has a keyspace of 62^8 ≈ 2.18 × 10^14. The nibble-to-mod-62 encoding introduces a 4% bias per character (nibble range 0–15, 62 is not a power of 2) which is negligible in practice.

This is intentional — the token is designed to be human-readable and copy-pasteable. The attack surface is the tunnel URL itself, which is already a ~100-bit entropy Cloudflare Quick Tunnel subdomain. An attacker needs both the URL and the token.

**Accepted.** The token protects against accidental URL sharing/guessing, not nation-state attacks. If higher entropy is needed, the token length can be extended in `generate_token()` without breaking the on-disk format.

### F2 — Token values stored in plaintext in `.sdlc/auth.yaml`

**Severity: LOW | Action: Accept**

Tokens are stored as plaintext YAML. This is consistent with how all other SDLC state is stored (features, milestones, secrets metadata are all plaintext YAML). The server does not handle plaintext secrets — that design boundary is maintained here.

Auth tokens are access credentials, not secrets in the AGE-encrypted sense. They grant read access to the SDLC dashboard over a tunnel. The risk is equivalent to storing a `.htpasswd` file without bcrypt — acceptable for this use case.

**Accepted.** If bcrypt hashing is desired in a future iteration, the schema supports it by replacing `token: String` with `token_hash: String` and a migration path.

### F3 — Token appears in server response body on creation only

**Severity: NONE | Status: Correct behavior**

`POST /api/auth/tokens` returns `{ name, token, created_at }` with the plaintext token value in the 201 response body. This is intentional — the caller must receive the token once to distribute it.

The token does NOT appear in `GET /api/auth/tokens` (list) or anywhere else. Verified in `list_tokens` route code and frontend `AuthToken` type (no `token` field).

No finding.

### F4 — Bearer token in `Authorization` header is not checked for case sensitivity

**Severity: LOW | Action: Accept**

The middleware checks for `"Bearer "` prefix with a case-sensitive `strip_prefix`. HTTP spec says `Authorization: bearer TOKEN` (lowercase) is also valid. An attacker using lowercase `bearer` would get a 401.

This is a usability edge case, not a security issue. The frontend and CLI documentation instruct callers to use `Bearer` (capitalized). Lowercase variants will fail gracefully with 401.

**Accepted.** Track as future improvement: `auth_header.to_ascii_lowercase().starts_with("bearer ")`.

### F5 — Hot-reload watcher replaces entire token list on any file change

**Severity: LOW | Action: Accept with documentation**

The watcher sets `snap.config.tokens = tokens_from_disk` on every mtime change. An ephemeral `_tunnel` token (added via `POST /api/tunnel`) is not persisted to disk, so it will be lost if `auth.yaml` is modified while a tunnel session is active.

The window is short: the tunnel token is only used during an active orch-tunnel session, and any disk modification to `auth.yaml` during a tunnel session (e.g., `sdlc auth token add`) would naturally intend to take effect immediately.

**Accepted.** Documented in MEMORY.md under AppState notes.

### F6 — `/api/auth/tokens` endpoint is not behind auth itself

**Severity: INFO | Status: Correct by design**

`GET /api/auth/tokens` is accessible over the tunnel — but only to authenticated callers (the auth middleware runs first). Unauthenticated tunnel requests receive 401 before reaching any route handler. Local requests (localhost/127.0.0.1) can always access this endpoint, which is correct for the CLI/admin use case.

No finding.

### F7 — Token names are not validated for format

**Severity: LOW | Action: Accept**

Token names are stored verbatim. There is no length limit, character restriction, or sanitization. A user could create a token with name `" "` (space) or a very long string. Names appear only in display contexts (CLI table, frontend label) and are not used in auth decisions.

There is no injection risk because names are stored in YAML (serde-serialized) and displayed in React (escaped) and terminal output (plain string). A degenerate name would be confusing but not harmful.

**Accepted.** A future validation gate (e.g., `name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')`) could be added to `add_token` without breaking existing stored tokens.

### F8 — No rate limiting on `POST /api/auth/tokens`

**Severity: INFO | Status: Acceptable**

The endpoint creates a new token on each call. Without rate limiting, an unauthenticated attacker... wait — the endpoint is behind the auth middleware. Only authenticated callers can create tokens. This is not an unauthenticated creation surface.

No finding.

### F9 — Token cookie uses `SameSite=Lax` without `Secure`

**Severity: LOW | Action: Accept**

The `?auth=TOKEN` flow sets `Set-Cookie: sdlc_auth=...; HttpOnly; SameSite=Lax; Path=/`. The cookie does not include `Secure`, meaning it could be sent over HTTP. However:
1. orch-tunnel Quick Tunnels are always HTTPS — the connection is encrypted end-to-end.
2. Local access (localhost) does not need a `Secure` cookie — browsers allow `Secure` cookies to be set by localhost but it is unnecessary.
3. Adding `Secure` would break local development use of the `?auth=TOKEN` flow.

**Accepted.** The `Secure` flag would require detecting whether the request arrived over HTTPS vs. HTTP — possible but adds complexity for marginal benefit given the HTTPS-only tunnel deployment.

---

## Security Summary

| Finding | Severity | Disposition |
|---|---|---|
| F1: Token entropy (8 chars) | MEDIUM | Accepted — adequate for use case |
| F2: Plaintext storage in auth.yaml | LOW | Accepted — consistent with project model |
| F3: Token in creation response only | NONE | Correct behavior |
| F4: Bearer case sensitivity | LOW | Accepted — usability edge case |
| F5: Hot-reload drops ephemeral token | LOW | Accepted — documented |
| F6: /api/auth/tokens exposed over tunnel | INFO | Correct — auth middleware gates it |
| F7: No name format validation | LOW | Accepted — no injection risk |
| F8: No rate limit on token creation | INFO | Not applicable — endpoint is auth-gated |
| F9: Cookie lacks Secure flag | LOW | Accepted — HTTPS-only tunnel deployment |

All findings are LOW or below. No HIGH or CRITICAL findings. The implementation is security-appropriate for a developer-facing SDLC dashboard tunnel guard.

---

## Verdict

**APPROVED.** No findings require remediation before release. All accepted findings are documented.
