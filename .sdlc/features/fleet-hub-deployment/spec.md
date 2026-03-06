# Spec: Deploy hub mode sdlc-server at sdlc.threesix.ai with Google OAuth via oauth2-proxy

## Problem

The sdlc fleet has per-project instances (e.g. `sdlc.sdlc.threesix.ai`) deployed via the existing Helm chart, but there is no central hub instance running at the apex domain `sdlc.threesix.ai`. The hub mode (`--hub` flag / `SDLC_HUB=true`) already exists in the codebase — it enables a `HubRegistry` that tracks project heartbeats, exposes `/api/hub/projects`, `/api/hub/heartbeat`, and `/api/hub/events` SSE — but it has never been deployed to the cluster.

Additionally, the hub needs Google OAuth authentication so only authorized users (from allowed email domains) can access the control plane UI. The existing per-project instances use tunnel auth, which is not appropriate for a shared control plane.

## Solution

Deploy a dedicated hub-mode sdlc-server instance at `sdlc.threesix.ai` with:

1. **oauth2-proxy** as a sidecar/separate deployment handling Google OAuth
2. **Traefik IngressRoute** with forward-auth middleware pointing to oauth2-proxy
3. **Hub-mode sdlc-server** deployment (`SDLC_HUB=true`) — no git-sync sidecar needed since the hub does not serve a project workspace

## Scope

This feature covers:
- Kubernetes manifests for the hub deployment (Deployment, Service, IngressRoute)
- oauth2-proxy deployment with Google provider configuration
- Traefik forward-auth middleware wiring
- Secrets (Google OAuth client credentials, cookie secret) stored as k8s Secrets
- Cloudflare DNS A record for `sdlc.threesix.ai`

This feature does **not** cover:
- Creating the Google OAuth client ID in GCP (T1 — manual/external step)
- The shared auth gate for per-project instances (fleet-auth-gate)
- Hub API endpoints beyond what already exists (fleet-management-api)
- Hub UI beyond what already exists (fleet-management-ui)

## Architecture

```
Browser → sdlc.threesix.ai
         │
         ▼
   Traefik IngressRoute
         │
         │  ForwardAuth middleware
         ▼
   oauth2-proxy (Google provider)
         │  ✓ authenticated
         ▼
   sdlc-server (hub mode)
     - /api/hub/projects
     - /api/hub/heartbeat  
     - /api/hub/events (SSE)
     - / (embedded frontend)
```

### Deployment Topology

All resources live in namespace `sdlc-hub`:

- **Deployment: sdlc-hub** — single container running `ghcr.io/orchard9/sdlc:latest` with `SDLC_HUB=true`
- **Deployment: oauth2-proxy** — single container running `quay.io/oauth2-proxy/oauth2-proxy:v7.6.0`
- **Service: sdlc-hub** — ClusterIP, port 80 → 8080
- **Service: oauth2-proxy** — ClusterIP, port 80 → 4180
- **Middleware: sdlc-hub-auth** — Traefik ForwardAuth pointing to `http://oauth2-proxy.sdlc-hub.svc.cluster.local/oauth2/auth`
- **IngressRoute: sdlc-hub** — host `sdlc.threesix.ai`, middleware chain `[sdlc-hub-auth]`, backend `sdlc-hub:80`

### Secrets

| Secret Name | Namespace | Keys | Source |
|---|---|---|---|
| `oauth2-proxy-credentials` | `sdlc-hub` | `client-id`, `client-secret`, `cookie-secret` | Manual creation from GCP OAuth client + `openssl rand -base64 32` |

### oauth2-proxy Configuration

```
--provider=google
--email-domain=livelyideo.tv
--email-domain=masq.me
--email-domain=virtualcommunities.ai
--upstream=static://200
--http-address=0.0.0.0:4180
--redirect-url=https://sdlc.threesix.ai/oauth2/callback
--cookie-secure=true
--cookie-name=_sdlc_hub_oauth2
--set-xauthrequest=true
```

### DNS

- Cloudflare A record: `sdlc.threesix.ai` → cluster ingress IP (same as existing `*.sdlc.threesix.ai`)

## Manifest Layout

```
k3s-fleet/deployments/hub/
├── namespace.yaml
├── oauth2-proxy-deployment.yaml
├── oauth2-proxy-service.yaml
├── sdlc-hub-deployment.yaml
├── sdlc-hub-service.yaml
├── middleware-forward-auth.yaml
├── ingressroute.yaml
└── README.md
```

All manifests are plain YAML (not Helm) since the hub is a singleton — there is no parameterization need.

## Non-Goals

- Persistent storage for the hub (the `HubRegistry` persists to `~/.sdlc/hub-state.yaml` inside the container — ephemeral and rebuilt from heartbeats)
- High availability / multi-replica (single pod is sufficient)
- Database for the hub instance (hub mode does not need PostgreSQL)
- Automated Google OAuth client creation
- Per-project auth (covered by fleet-auth-gate)

## Acceptance Criteria

1. `kubectl get pods -n sdlc-hub` shows two running pods: `sdlc-hub-*` and `oauth2-proxy-*`
2. Unauthenticated `curl -I https://sdlc.threesix.ai` returns 302 redirect to Google sign-in
3. After Google OAuth login with an allowed-domain email, the hub UI loads at `https://sdlc.threesix.ai`
4. `GET https://sdlc.threesix.ai/api/hub/projects` returns a JSON array (empty or with registered projects)
5. `GET https://sdlc.threesix.ai/api/health` returns 200 (health endpoint bypasses auth)
6. The IngressRoute uses the `sdlc-wildcard-tls` certificate for TLS termination
7. All manifests are committed to `k3s-fleet/deployments/hub/` in this repository

## Dependencies

- Wildcard TLS cert for `*.sdlc.threesix.ai` must exist (fleet-ingress-tls — already deployed)
- Google OAuth client ID must be created manually (T1) before deployment
- Traefik ingress controller must be running in the cluster (already present)
