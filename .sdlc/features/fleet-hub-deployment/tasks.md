# Tasks: Deploy hub mode sdlc-server at sdlc.threesix.ai

## T1: Create Google OAuth client ID in GCP for sdlc.threesix.ai with allowed domains livelyideo.tv, masq.me, virtualcommunities.ai

**Type:** Manual/External  
**Scope:** GCP Console → APIs & Services → Credentials → OAuth 2.0 Client IDs  
- Application type: Web application
- Authorized redirect URI: `https://sdlc.threesix.ai/oauth2/callback`
- Note the client ID and client secret for T2

## T2: Store OAuth client ID and secret as k8s Secret in hub namespace

**Type:** Infrastructure  
**Depends on:** T1  
**Scope:**
- Create namespace `sdlc-hub` if it does not exist
- Create Secret `oauth2-proxy-credentials` with keys: `client-id`, `client-secret`, `cookie-secret`
- Cookie secret generated via `openssl rand -base64 32`
- Applied via `kubectl create secret generic` (not committed to git — contains credentials)

## T3: Deploy oauth2-proxy — Google provider, allowed email domains, cookie secret

**Type:** Infrastructure manifest  
**Depends on:** T2  
**Scope:**
- Write `k3s-fleet/deployments/hub/oauth2-proxy-deployment.yaml` — Deployment with oauth2-proxy v7.6.0, Google provider, three allowed email domains, env from Secret
- Write `k3s-fleet/deployments/hub/oauth2-proxy-service.yaml` — ClusterIP Service port 80 → 4180
- Apply to cluster and verify pod reaches Running state

## T4: Create Traefik IngressRoute for sdlc.threesix.ai with forward-auth middleware pointing to oauth2-proxy

**Type:** Infrastructure manifest  
**Depends on:** T3  
**Scope:**
- Write `k3s-fleet/deployments/hub/middleware-forward-auth.yaml` — Traefik ForwardAuth middleware pointing to `http://oauth2-proxy.sdlc-hub.svc.cluster.local/oauth2/auth`
- Write `k3s-fleet/deployments/hub/ingressroute.yaml` — IngressRoute with two rules: authenticated main route and unauthenticated `/oauth2/*` pass-through
- TLS via `sdlc-wildcard-tls` secret
- Ensure Cloudflare DNS A record exists for `sdlc.threesix.ai`

## T5: Deploy hub mode sdlc-server (SDLC_HUB=true) as separate deployment from sdlc-sdlc

**Type:** Infrastructure manifest  
**Scope:**
- Write `k3s-fleet/deployments/hub/namespace.yaml`
- Write `k3s-fleet/deployments/hub/sdlc-hub-deployment.yaml` — single container, `SDLC_HUB=true`, no git-sync
- Write `k3s-fleet/deployments/hub/sdlc-hub-service.yaml` — ClusterIP port 80 → 8080
- Liveness/readiness probes on `/api/health`
- Apply and verify pod Running + health check passing

## T6: Helm chart: hub values block with oauth2-proxy config and forward-auth middleware template

**Type:** Code/Template  
**Scope:**
- This task is deprioritized — the hub is a singleton and uses plain YAML manifests, not Helm
- If later needed, add a `hub:` values block to the existing chart with conditional templates
- For now, mark as not-applicable since the design chose plain manifests over Helm for the singleton hub
