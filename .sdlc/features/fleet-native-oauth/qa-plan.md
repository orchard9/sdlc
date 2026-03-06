# QA Plan: fleet-native-oauth

## Unit Tests (cargo test)

1. **login handler** — returns 302 with correct Google authorize URL params (client_id, scope, redirect_uri, state)
2. **callback — allowed domain** — email@livelyideo.tv → sets cookie, 302 to /
3. **callback — disallowed domain** — email@gmail.com → 403
4. **callback — missing code param** — 400
5. **verify — valid cookie** — returns 200 with X-Auth-User header
6. **verify — expired cookie** — returns 401
7. **verify — missing cookie** — returns 401
8. **verify — tampered cookie (bad HMAC)** — returns 401
9. **verify — valid Bearer token (M2M)** — returns 200
10. **verify — invalid Bearer token** — returns 401
11. **logout** — clears cookie (Max-Age=0), 302 to /
12. **sign_session / verify_session roundtrip** — sign then verify returns original payload
13. **verify_session with wrong secret** — returns None

## Integration Tests

14. **No OAuth config (project mode)** — /auth/* routes return 404 or passthrough, server starts normally
15. **Existing auth.rs unaffected** — Bearer tokens via Authorization header still work for /api/* routes
16. **Cookie domain** — verify cookie Domain attribute is `.sdlc.threesix.ai`

## Build Verification

17. `SDLC_NO_NPM=1 cargo test --all` passes
18. `cargo clippy --all -- -D warnings` passes
