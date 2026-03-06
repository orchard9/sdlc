# Spec: fleet-deploy-hub

## Problem

The fleet control plane code exists (fleet.rs, hub.rs, HubPage.tsx) but nothing is deployed. There's no hub instance, no DNS, no auth gate, no way to reach the fleet dashboard.

## Solution

Deploy the hub mode sdlc-server in its own namespace with native OAuth2 auth, DNS, and Traefik ingress. Wire all existing sdlc-* instances behind the same auth gate.

## Deployment Architecture

```
sdlc.threesix.ai (hub)
├── Traefik IngressRoute → sdlc-hub service (port 8080)
├── Native OAuth2 (/auth/login, /auth/callback, /auth/verify, /auth/logout)
├── Fleet API (/api/hub/fleet, /api/hub/repos, /api/hub/provision, /api/hub/import)
└── Hub UI (React dashboard)

*.sdlc.threesix.ai (project instances)
├── Traefik IngressRoute with forwardAuth middleware → sdlc.threesix.ai/auth/verify
└── Per-instance sdlc-server (unchanged)
```

## Prerequisites (Manual Steps)

1. **GCP OAuth Client** — Create in Google Cloud Console:
   - Authorized redirect URIs: `https://sdlc.threesix.ai/auth/callback`
   - Authorized domains: `sdlc.threesix.ai`

2. **Generate SESSION_SECRET** — `openssl rand -hex 32`

## Namespace: sdlc-hub

Deployment (no git-sync sidecar — hub has no project repo):
- Image: `ghcr.io/orchard9/sdlc:latest`
- Env: `SDLC_HUB=true`, `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `OAUTH_ALLOWED_DOMAINS=livelyideo.tv,masq.me,virtualcommunities.ai`, `SESSION_SECRET`
- Env: `GITEA_URL`, `GITEA_API_TOKEN`, `WOODPECKER_URL`, `WOODPECKER_API_TOKEN`
- Env: `INGRESS_DOMAIN=sdlc.threesix.ai`
- ServiceAccount with ClusterRole: read namespaces, read deployments (fleet discovery)

## DNS

Cloudflare A record: `sdlc.threesix.ai` → `208.122.204.172` (same IP as wildcard)

## Traefik Configuration

Hub IngressRoute:
```yaml
spec:
  routes:
    - match: Host(`sdlc.threesix.ai`)
      kind: Rule
      services:
        - name: sdlc-hub
          port: 8080
```

ForwardAuth middleware (shared across all project IngressRoutes):
```yaml
spec:
  forwardAuth:
    address: http://sdlc-hub.sdlc-hub.svc.cluster.local:8080/auth/verify
    trustForwardHeader: true
    authResponseHeaders:
      - X-Auth-User
```

## What This Does NOT Include

- oauth2-proxy (replaced by native OAuth2)
- Per-project auth tokens (existing bearer token system handles M2M)
- Hub mode git-sync (hub has no project checkout)
