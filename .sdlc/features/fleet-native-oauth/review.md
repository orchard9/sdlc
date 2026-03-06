# Review: fleet-native-oauth

## Files Changed

| File | Change |
|------|--------|
| `crates/sdlc-server/Cargo.toml` | Added `hmac`, `sha2`, `base64` deps |
| `crates/sdlc-server/src/oauth.rs` | **New** — OAuthConfig, session sign/verify, 4 route handlers, 16 tests |
| `crates/sdlc-server/src/lib.rs` | Registered `oauth` module, added `/auth/*` routes |
| `crates/sdlc-server/src/state.rs` | Added `oauth_config: Option<Arc<OAuthConfig>>`, init in `new_with_port_hub` |
| `crates/sdlc-server/src/auth.rs` | Added `/auth/` path bypass (public routes) |

## Findings

### Correct

1. **Session security** — HMAC-SHA256 signed cookies with constant-time comparison. No timing attack surface.
2. **CSRF protection** — State param is HMAC-signed timestamp with 10-min expiry window.
3. **Domain validation** — Email domain checked against allowlist before session is created.
4. **Cookie scope** — `Domain=.sdlc.threesix.ai` covers all subdomains for SSO.
5. **M2M path** — Bearer tokens from `HUB_SERVICE_TOKENS` work alongside OAuth sessions in `/auth/verify`.
6. **Graceful degradation** — No OAuth config = routes return 404, server works in project mode.
7. **Test coverage** — 16 unit tests covering all handlers, edge cases (expired, tampered, wrong secret, missing config).

### Accepted

1. **No `oauth2` crate used** — The spec mentioned it but the implementation uses raw HTTP calls to Google endpoints. This is simpler and avoids a heavyweight dependency for 2 HTTP calls. Accepted.
2. **Inline hex/urlencoding** — Avoids adding `hex` and `urlencoding` crate dependencies for ~20 lines of code. Acceptable.
3. **Dead `verify` handler** — The `verify` handler exists but `verify_from_request` is what's actually wired. The unused handler could be removed but it's harmless.

### No blockers found
