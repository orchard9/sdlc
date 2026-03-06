# Tasks: fleet-native-oauth

## Implementation Order

1. **[T1] Add dependencies** — `oauth2`, `hmac`, `sha2`, `base64` to sdlc-server Cargo.toml
2. **[T9] Wire OAuthConfig into AppState** — struct + env var init in `new_with_port_hub`
3. **[T2] Create oauth.rs** — module with OAuthConfig, SessionPayload, cookie sign/verify helpers
4. **[T7] Session cookie** — HMAC-SHA256 signing, base64 encoding, expiry validation
5. **[T3] GET /auth/login** — build authorize URL, CSRF state param
6. **[T4] GET /auth/callback** — code exchange, userinfo fetch, domain validation, set cookie
7. **[T5] GET /auth/verify** — forwardAuth endpoint, check cookie + bearer fallback
8. **[T6] POST /auth/logout** — clear cookie, redirect
9. **[T8] Integration with auth.rs** — ensure Bearer tokens + hub_service_tokens still work
10. **[T10] Route registration** — /auth/* routes before auth middleware in lib.rs
11. **[T11] Unit tests** — full coverage of all handlers and edge cases
