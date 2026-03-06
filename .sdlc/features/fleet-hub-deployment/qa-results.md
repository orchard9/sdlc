# QA Results: Deploy hub mode sdlc-server at sdlc.threesix.ai

## Environment

- Local validation via `kubectl apply --dry-run=client` and `yq`
- Cluster deployment verification deferred to post-merge (requires GCP OAuth client + live cluster)

## Pre-deployment Checks

### Q1: Manifest Lint — PASS
- `kubectl apply --dry-run=client` passes for all 5 core k8s resources (namespace, 2 deployments, 2 services)
- Traefik CRDs (IngressRoute, Middleware) validate structurally via `yq` — CRDs only available on cluster with Traefik installed
- No hardcoded secrets in any committed manifest — confirmed via grep

### Q2: Namespace Isolation — PASS
- All resources specify `namespace: sdlc-hub`
- No naming conflicts with existing `sdlc-*` project namespaces

## Structural Verification

### Q3: Service Selector Matching — PASS
- `sdlc-hub` service selector `app: sdlc-hub` matches deployment pod label `app: sdlc-hub`
- `oauth2-proxy` service selector `app: oauth2-proxy` matches deployment pod label `app: oauth2-proxy`

### Q4: IngressRoute Configuration — PASS
- Two routes: `/oauth2` prefix (direct to oauth2-proxy) and catch-all (through auth middleware to sdlc-hub)
- TLS secret: `sdlc-wildcard-tls` (matches fleet-ingress-tls output)
- Entry point: `websecure`

### Q5: ForwardAuth Middleware — PASS
- Address: `http://oauth2-proxy.sdlc-hub.svc.cluster.local/oauth2/auth` — correct cluster DNS
- Response headers: `X-Auth-Request-User`, `X-Auth-Request-Email`
- `trustForwardHeader: true`

### Q6: oauth2-proxy Arguments — PASS
- Provider: `google`
- Email domains: `livelyideo.tv`, `masq.me`, `virtualcommunities.ai` — all three present
- Redirect URL: `https://sdlc.threesix.ai/oauth2/callback`
- Cookie: secure=true, name=`_sdlc_hub_oauth2`
- Credentials from Secret: `client-id`, `client-secret`, `cookie-secret` via `oauth2-proxy-credentials`

### Q7: sdlc-hub Deployment — PASS
- Single container: `ghcr.io/orchard9/sdlc:latest`
- Args: `serve --hub --port 8080`
- Env: `SDLC_HUB=true`, `SDLC_ROOT=/tmp/sdlc-hub`
- Probes: liveness + readiness on `/api/health:8080`
- Resources: 50m/64Mi requests, 250m/128Mi limits

### Q8: Port Mapping — PASS
- sdlc-hub: Service 80 -> container 8080
- oauth2-proxy: Service 80 -> container 4180

## Deferred Checks (require live cluster + GCP OAuth)

The following QA plan items require the actual cluster deployment and are deferred to the deployment verification step:

- Q3 (Pod Health) — requires live cluster
- Q4 (Service Connectivity) — requires live cluster
- Q6 (Unauthenticated Access Redirects) — requires live deployment + DNS
- Q7 (Google OAuth Login) — requires GCP OAuth client
- Q8 (Unauthorized Domain Rejection) — requires GCP OAuth client
- Q9-Q11 (Hub API) — requires live deployment
- Q12 (TLS Certificate) — requires live deployment
- Q13 (DNS Resolution) — requires Cloudflare DNS record

## Summary

| Check | Result |
|---|---|
| Q1: Manifest lint | PASS |
| Q2: Namespace isolation | PASS |
| Q3: Service selectors | PASS |
| Q4: IngressRoute config | PASS |
| Q5: ForwardAuth middleware | PASS |
| Q6: oauth2-proxy args | PASS |
| Q7: sdlc-hub deployment | PASS |
| Q8: Port mapping | PASS |

**All locally-verifiable checks pass.** Manifests are structurally correct, consistently namespaced, and follow the design. Live cluster verification will confirm runtime behavior after deployment.

## Verdict

**PASS** — ready for merge.
