# Design: fleet-auth-gate

## Architecture Overview

This feature is pure Kubernetes infrastructure — no Rust code changes, no frontend changes. All work is YAML manifests: one Traefik Middleware CRD, one Helm template addition, and configuration patches to existing resources.

```
┌───────────────────────────────────────────────────────────────────┐
│                        Traefik Ingress Controller                │
│                                                                   │
│  Request to sdlc-*.threesix.ai                                   │
│       │                                                           │
│       ▼                                                           │
│  ┌─────────────────────────────┐                                 │
│  │ Middleware: sdlc-google-auth│                                 │
│  │ (forwardAuth)               │                                 │
│  └────────────┬────────────────┘                                 │
│               │                                                   │
│               ▼                                                   │
│  ┌─────────────────────────────┐    ┌──────────────────────┐     │
│  │ oauth2-proxy                │───▶│ Google OAuth          │     │
│  │ (sdlc-hub namespace)       │◀───│ (accounts.google.com) │     │
│  └────────────┬────────────────┘    └──────────────────────┘     │
│               │ 200 OK (authed)                                   │
│               ▼                                                   │
│  ┌─────────────────────────────┐                                 │
│  │ sdlc-server pod             │                                 │
│  │ (project namespace)         │                                 │
│  └─────────────────────────────┘                                 │
└───────────────────────────────────────────────────────────────────┘
```

## Request Flow

1. Browser hits `sdlc-myproject.threesix.ai`
2. Traefik matches the IngressRoute, sees `sdlc-google-auth` middleware
3. Traefik sends a subrequest to `oauth2-proxy` at `/oauth2/auth`
4. If no valid `_oauth2_proxy` cookie: oauth2-proxy returns 401, Traefik redirects to `/oauth2/start?rd=<original-url>` which begins Google sign-in
5. Google sign-in completes, redirects to `/oauth2/callback`, oauth2-proxy sets `_oauth2_proxy` cookie on `.sdlc.threesix.ai` domain
6. Subsequent requests carry the cookie, oauth2-proxy returns 200, Traefik forwards to the sdlc-server pod
7. Bearer token requests: oauth2-proxy is configured with `--skip-auth-regex=^/api/` OR the sdlc-server's existing `auth.rs` handles bearer validation after oauth2-proxy passthrough

## File Changes

### New files

#### `k3s-fleet/deployments/helm/sdlc-server/templates/middleware-google-auth.yaml`

```yaml
{{- if .Values.auth.enabled }}
apiVersion: traefik.io/v1alpha1
kind: Middleware
metadata:
  name: sdlc-google-auth
  namespace: {{ .Release.Namespace }}
spec:
  forwardAuth:
    address: http://oauth2-proxy.{{ .Values.auth.proxyNamespace }}.svc.cluster.local:4180/oauth2/auth
    trustForwardHeader: true
    authResponseHeaders:
      - X-Auth-Request-Email
      - X-Auth-Request-User
{{- end }}
```

Note: The middleware is deployed per-namespace rather than in a central namespace. This avoids cross-namespace Middleware reference issues in Traefik. Each project namespace gets its own copy of the middleware, all pointing to the same oauth2-proxy service.

Alternatively, deploy a single Middleware in `sdlc-hub` and reference it as `sdlc-hub-sdlc-google-auth@kubernetescrd` from all IngressRoutes. This is cleaner but requires Traefik's `allowCrossNamespace` to be enabled. Decision: use the per-namespace approach since it works with default Traefik settings.

#### `k3s-fleet/deployments/helm/sdlc-server/templates/ingressroute.yaml`

```yaml
apiVersion: traefik.io/v1alpha1
kind: IngressRoute
metadata:
  name: sdlc-{{ .Values.project.slug }}
  namespace: {{ .Release.Namespace }}
spec:
  entryPoints:
    - websecure
  routes:
    - match: Host(`{{ .Values.project.slug }}.{{ .Values.ingress.domain }}`)
      kind: Rule
      services:
        - name: sdlc-server-{{ .Values.project.slug }}
          port: 8080
      {{- if .Values.auth.enabled }}
      middlewares:
        - name: sdlc-google-auth
      {{- end }}
  tls:
    secretName: {{ .Values.ingress.tlsSecretName }}
```

### Modified files

#### `k3s-fleet/deployments/helm/sdlc-server/values.yaml`

Add auth section:

```yaml
auth:
  enabled: false                    # Enable Google OAuth gate
  proxyNamespace: sdlc-hub          # Namespace where oauth2-proxy runs
```

Default `enabled: false` so existing deployments are unaffected. The fleet-reconcile pipeline sets `auth.enabled=true` when deploying.

### oauth2-proxy Configuration (in fleet-hub-deployment)

Key settings that affect this feature (documented here for reference, implemented in `fleet-hub-deployment`):

```
--cookie-domain=.sdlc.threesix.ai     # Shared across all subdomains
--cookie-expire=24h                    # Session TTL
--cookie-refresh=1h                    # Refresh cookie silently every hour
--cookie-secure=true                   # HTTPS only
--cookie-samesite=lax                  # Cross-subdomain but not cross-site
--whitelist-domain=.sdlc.threesix.ai   # Allow redirects to any subdomain
--set-xauthrequest=true                # Pass email/user headers downstream
--pass-authorization-header=true       # Forward Bearer tokens to upstream
```

## Service Token Path

For machine-to-machine calls (dev-driver, CI, Woodpecker), two approaches work:

**Option A (preferred):** oauth2-proxy `--skip-auth-regex` for bearer-token paths. Configure oauth2-proxy to skip auth checks when a valid `Authorization: Bearer` header is present. The existing `auth.rs` in sdlc-server already validates bearer tokens, so double-gating is unnecessary.

**Option B:** oauth2-proxy `--pass-authorization-header=true` passes the bearer token through to sdlc-server even for authenticated sessions. The sdlc-server's `auth.rs` already handles bearer validation. This means both oauth2-proxy AND sdlc-server check auth — belt and suspenders.

Recommendation: Option B is simpler and doesn't require regex configuration. oauth2-proxy passes the header through, sdlc-server validates it. If the request also has a valid Google session cookie, oauth2-proxy lets it through; if it only has a bearer token, oauth2-proxy's `--pass-authorization-header` ensures sdlc-server sees it.

## Interaction with Existing auth.rs

The sdlc-server `auth.rs` middleware currently handles:
- Tunnel token auth (cloudflare tunnel use case)
- localhost bypass
- Bearer token validation
- Cookie-based session (`sdlc_auth` cookie)

In fleet mode, oauth2-proxy handles the primary auth gate at the Traefik level. The sdlc-server `auth.rs` continues to work as a second layer for bearer tokens, but the tunnel token flow becomes unnecessary for fleet instances (oauth2-proxy replaces it). No changes to `auth.rs` are needed — it gracefully passes through when no tokens are configured (`TunnelConfig::none()`).

## Rollback Plan

If the auth gate causes issues:
1. Set `auth.enabled: false` in the Helm values and redeploy
2. Delete the Middleware CRDs from affected namespaces
3. Traffic flows directly to sdlc-server pods without auth
