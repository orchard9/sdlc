# Spec: fleet-auth-gate

## Summary

Create a shared Traefik forward-auth middleware that routes all `*.sdlc.threesix.ai` traffic through the hub's oauth2-proxy before reaching individual sdlc-server instances. This ensures every project instance in the fleet requires Google sign-in — not just the hub at `sdlc.threesix.ai`.

## Problem

Individual sdlc project instances (`sdlc-<project>.threesix.ai`) are currently accessible without authentication. Anyone who knows or guesses a project URL can access the full UI and API. The hub deployment (fleet-hub-deployment) will have its own oauth2-proxy, but that only gates `sdlc.threesix.ai` — project subdomains bypass it entirely.

## Solution

### Shared Traefik Middleware

Create a Traefik `Middleware` CRD of type `forwardAuth` in the hub namespace that points to the oauth2-proxy instance deployed by `fleet-hub-deployment`. This middleware:

- Forwards the request to `http://oauth2-proxy.sdlc-hub.svc.cluster.local:4180/oauth2/auth`
- Passes `X-Forwarded-*` headers so oauth2-proxy can redirect back to the original URL after Google sign-in
- Sets `authResponseHeaders` to forward `X-Auth-Request-Email` and `X-Auth-Request-User` downstream

```yaml
apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: sdlc-google-auth
  namespace: sdlc-hub
spec:
  forwardAuth:
    address: http://oauth2-proxy.sdlc-hub.svc.cluster.local:4180/oauth2/auth
    trustForwardHeader: true
    authResponseHeaders:
      - X-Auth-Request-Email
      - X-Auth-Request-User
```

### Helm Template Integration

Add the middleware annotation to the sdlc-server Helm chart's IngressRoute template so every project instance deployed via fleet-reconcile automatically gets the auth gate:

```yaml
traefik.ingress.kubernetes.io/router.middlewares: sdlc-hub-sdlc-google-auth@kubernetescrd
```

The middleware reference format is `<namespace>-<name>@kubernetescrd` — Traefik requires the full namespaced reference when the middleware lives in a different namespace than the IngressRoute.

### Retrofit Existing Ingress

The existing `sdlc-sdlc` instance ingress must be updated to include the same middleware annotation. This is a one-time manual update or a Helm upgrade if sdlc-sdlc is managed by the chart.

### Service Token Bypass

Machine-to-machine API calls (dev-driver, CI agents, Woodpecker pipelines) use `Authorization: Bearer <token>` headers. The oauth2-proxy must be configured to pass through requests with valid bearer tokens without requiring a Google session. oauth2-proxy supports this via `--skip-auth-route` for specific paths or `--pass-authorization-header=true` combined with the existing sdlc-server `auth.rs` bearer validation.

### Session Management

oauth2-proxy handles session cookies (`_oauth2_proxy` cookie). Configure:
- Cookie TTL: 24 hours (reasonable for a dev tool)
- Cookie domain: `.sdlc.threesix.ai` (shared across all subdomains so one login covers hub + all projects)
- Logout: oauth2-proxy exposes `/oauth2/sign_out` — the hub UI can link to it

## Scope

### In scope
- Traefik Middleware CRD manifest for shared forward-auth
- Helm chart template update to apply middleware to all project IngressRoutes
- Retrofit of existing sdlc-sdlc ingress
- oauth2-proxy cookie domain configuration for cross-subdomain SSO
- Session expiry (24h TTL) and logout endpoint configuration
- Service token path for machine-to-machine API calls

### Out of scope
- Deploying oauth2-proxy itself (that is `fleet-hub-deployment`)
- Creating the Google OAuth client ID (ops prerequisite)
- Per-user or per-project access control
- Revoking individual sessions

## Dependencies

- `fleet-hub-deployment` must deploy oauth2-proxy in `sdlc-hub` namespace first
- Wildcard TLS cert for `*.sdlc.threesix.ai` must be deployed (v18)
- Google OAuth client ID must be provisioned in GCP

## Acceptance Criteria

1. Unauthenticated request to `sdlc-sdlc.threesix.ai` returns a 302 redirect to Google sign-in
2. After Google sign-in, the user is redirected back to the original URL and can access the app
3. A single Google sign-in covers both `sdlc.threesix.ai` and `sdlc-<project>.threesix.ai` (shared cookie domain)
4. `Authorization: Bearer <valid-token>` requests to `/api/*` endpoints bypass Google OAuth
5. New project instances deployed via fleet-reconcile automatically get the auth gate (no manual configuration)
6. Clicking logout at the hub invalidates the session across all subdomains
