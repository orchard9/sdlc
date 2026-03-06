# Spec: fleet-native-oauth

## Problem

The fleet control plane features reference `oauth2-proxy` as the auth layer, but:
- oauth2-proxy is unreliable (frequent failures, complex config surface)
- It adds Redis or oversized cookie session storage as a dependency
- It's a separate process that crashes independently
- Its session management fights with our existing `auth.rs` token system

## Solution

Build Google OAuth2 directly into `sdlc-server`'s hub mode. Four route handlers, signed session cookies, zero additional infrastructure.

## Route Handlers

### `GET /auth/login`
- Build Google authorize URL: `https://accounts.google.com/o/oauth2/v2/auth`
- Scopes: `openid email profile`
- State param: HMAC-signed timestamp (CSRF protection)
- Redirect URI: `https://sdlc.threesix.ai/auth/callback`
- Response: 302 redirect to Google

### `GET /auth/callback`
- Exchange authorization code for access token via `https://oauth2.googleapis.com/token`
- Fetch userinfo from `https://openidconnect.googleapis.com/v1/userinfo`
- Validate email domain against `OAUTH_ALLOWED_DOMAINS` (comma-separated)
- On success: set `sdlc_session` cookie, 302 redirect to `/`
- On failure: 403 with error message

### `GET /auth/verify`
- Traefik forwardAuth endpoint
- Check `sdlc_session` cookie → valid + not expired? Return 200 with `X-Auth-User` header
- Also check `Authorization: Bearer <token>` against `HUB_SERVICE_TOKENS` (M2M path)
- Invalid/missing/expired: return 401

### `POST /auth/logout`
- Clear `sdlc_session` cookie (set Max-Age=0)
- 302 redirect to `/`

## Session Cookie Format

```
sdlc_session=<base64(payload)>.<hmac_signature>
```

Payload (JSON):
```json
{
  "email": "user@livelyideo.tv",
  "name": "Jordan",
  "exp": 1741305600
}
```

Cookie attributes:
- `HttpOnly; Secure; SameSite=Lax`
- `Domain=.sdlc.threesix.ai` (covers all subdomains)
- `Path=/`
- `Max-Age=86400` (24 hours)

Signature: `HMAC-SHA256(payload_bytes, SESSION_SECRET)`

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `GOOGLE_CLIENT_ID` | Yes (hub) | GCP OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | Yes (hub) | GCP OAuth client secret |
| `OAUTH_ALLOWED_DOMAINS` | Yes (hub) | Comma-separated: `livelyideo.tv,masq.me,virtualcommunities.ai` |
| `SESSION_SECRET` | Yes (hub) | 32+ char secret for HMAC signing |
| `OAUTH_REDIRECT_URI` | No | Override callback URL (default: auto-detect from Host header) |

## Integration with Existing Auth

The existing `auth.rs` middleware continues unchanged. The `/auth/verify` endpoint is a separate Traefik forwardAuth target. The flow:

1. Request arrives at Traefik for `*.sdlc.threesix.ai`
2. Traefik sends subrequest to `http://sdlc-hub.sdlc-hub.svc.cluster.local:8080/auth/verify`
3. Hub checks cookie → 200 (pass) or 401 (redirect to `/auth/login`)
4. If 401, Traefik returns the 401 + `Location` header to the browser

The hub's own auth middleware (`auth.rs`) runs AFTER the routes, so `/auth/login` and `/auth/callback` are accessible without a session.

## What This Does NOT Include

- User-level permissions (all authenticated users see everything)
- Session revocation beyond cookie expiry
- Refresh tokens (sessions are self-contained, 24h TTL)
- Database-backed sessions (cookie is the session store)
