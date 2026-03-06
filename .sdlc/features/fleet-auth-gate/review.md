# Review: fleet-auth-gate

## Files Changed

### New files (Helm chart templates)

1. **`k3s-fleet/deployments/helm/sdlc-server/templates/middleware-google-auth.yaml`** — Traefik `Middleware` CRD with `forwardAuth` pointing to `oauth2-proxy.<proxyNamespace>.svc.cluster.local`. Gated by `auth.enabled`. Passes `X-Auth-Request-Email` and `X-Auth-Request-User` downstream.

2. **`k3s-fleet/deployments/helm/sdlc-server/templates/ingressroute.yaml`** — Traefik `IngressRoute` CRD routing `<slug>.<domain>` to `sdlc-server-<slug>` service on port 80. Conditionally applies `sdlc-google-auth` middleware when `auth.enabled` is true.

3. **`k3s-fleet/deployments/helm/sdlc-server/templates/service.yaml`** — ClusterIP `Service` mapping port 80 to container port 8080. Selector matches the deployment labels.

### Modified files

4. **`k3s-fleet/deployments/helm/sdlc-server/values.yaml`** — Added `auth.enabled` (default `false`) and `auth.proxyNamespace` (default `sdlc-hub`). Backward compatible — existing deployments unaffected.

5. **`k3s-fleet/deployments/hub/oauth2-proxy-deployment.yaml`** — Added 6 args for cross-subdomain SSO and bearer passthrough:
   - `--cookie-domain=.sdlc.threesix.ai` (shared cookie across subdomains)
   - `--cookie-expire=24h0m0s` (session TTL)
   - `--cookie-refresh=1h0m0s` (silent refresh before expiry)
   - `--cookie-samesite=lax` (cross-subdomain but not cross-site)
   - `--whitelist-domain=.sdlc.threesix.ai` (allow redirects to subdomains)
   - `--pass-authorization-header=true` (forward bearer tokens to upstream)

## Findings

### F1: No Rust code changes needed

The existing `auth.rs` in sdlc-server handles bearer token validation and gracefully passes through when no tokens are configured. The auth gate is entirely at the Kubernetes/Traefik layer, which is the correct architectural choice — auth is an infrastructure concern, not an application concern.

**Action:** Accepted (by design).

### F2: IngressRoute service port matches Service definition

The IngressRoute references `port: 80` on the service, and the Service maps `port: 80 -> targetPort: 8080`. The deployment exposes `containerPort: 8080`. The chain is correct.

**Action:** Verified, no issue.

### F3: Middleware uses ClusterIP service address (port 80 via Service, not 4180 direct)

The middleware address uses `oauth2-proxy.<ns>.svc.cluster.local` without a port, which defaults to port 80. The hub's `oauth2-proxy` Service maps port 80 to targetPort 4180. This matches the hub's own middleware (`middleware-forward-auth.yaml`) which also omits the port. Consistent.

**Action:** Verified, no issue.

### F4: Missing `--cookie-domain` trailing dot convention

oauth2-proxy interprets `--cookie-domain=.sdlc.threesix.ai` (leading dot) as "set cookie for this domain and all subdomains". This is the correct format for cross-subdomain SSO. Verified against oauth2-proxy documentation.

**Action:** Verified, correct.

### F5: auth.enabled defaults to false — safe rollout

Existing Helm releases that don't specify `auth.enabled` will not get the middleware or the forward-auth on their IngressRoutes. This is the correct default for backward compatibility.

**Action:** Accepted.

## Verdict

All files are correct, consistent with the hub's existing patterns, and backward compatible. No issues found.
