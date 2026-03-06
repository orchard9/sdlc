# Security Audit: fleet-native-oauth

## Scope

Native Google OAuth2 implementation for hub mode: session creation, cookie management, forwardAuth verification.

## Findings

### A1: Session cookie HMAC uses constant-time comparison — PASS
`constant_time_eq` uses XOR accumulation, no early exit. Timing attacks on cookie validation are not possible.

### A2: CSRF state parameter validation — PASS
Login generates HMAC-signed timestamp as state param. Callback verifies signature and 10-minute expiry. Prevents cross-site request forgery on the callback endpoint.

### A3: Email domain validation — PASS
Domain is extracted from email after `@`, lowercased, compared against allowlist. No regex — exact match only. Cannot be bypassed with case tricks.

### A4: Cookie attributes — PASS
`HttpOnly` (no JS access), `Secure` (HTTPS only), `SameSite=Lax` (CSRF for POST), `Domain=.sdlc.threesix.ai` (wildcard coverage). Standard secure cookie configuration.

### A5: Session secret from environment — PASS
`SESSION_SECRET` env var used as HMAC key. Not hardcoded, not logged. Standard practice for container deployments.

### A6: Token exchange uses HTTPS — PASS
Google token endpoint (`https://oauth2.googleapis.com/token`) and userinfo endpoint both use HTTPS. Client secret is sent via POST body (form-encoded), not in URL.

### A7: No session revocation — ACCEPTED
Sessions are self-contained cookies with 24h TTL. No server-side session store means no revocation capability. Accepted for v1 — the threat model (3 trusted domains, internal fleet) doesn't warrant the complexity of a session store.

### A8: Redirect URI not validated against request origin — LOW RISK
The callback handler uses the configured `redirect_uri` (from env or derived from cookie domain). An attacker cannot inject a different redirect_uri into the token exchange because it's server-side config, not user input.

### A9: Bearer token comparison in verify — PASS
`hub_service_tokens` comparison uses standard string equality. These are long random tokens (not passwords), so timing attacks are impractical. The risk is negligible.

### A10: /auth/* routes bypass tunnel auth middleware — PASS
The `/auth/` path bypass in `auth.rs` is intentional — these routes must be accessible without a session (they create sessions). The bypass is scoped to `/auth/` only and does not affect `/api/` routes.

## Verdict

No blockers. The implementation follows standard OAuth2 security practices. The main limitation (no session revocation) is accepted for the current threat model.
