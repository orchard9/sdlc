# Review: Deploy hub mode sdlc-server at sdlc.threesix.ai

## Files Changed

All new files in `k3s-fleet/deployments/hub/`:

| File | Purpose |
|---|---|
| `namespace.yaml` | Creates `sdlc-hub` namespace with fleet labels |
| `sdlc-hub-deployment.yaml` | Hub-mode sdlc-server (single container, no git-sync) |
| `sdlc-hub-service.yaml` | ClusterIP service for sdlc-hub (80 -> 8080) |
| `oauth2-proxy-deployment.yaml` | Google OAuth proxy with allowed email domains |
| `oauth2-proxy-service.yaml` | ClusterIP service for oauth2-proxy (80 -> 4180) |
| `middleware-forward-auth.yaml` | Traefik ForwardAuth middleware pointing to oauth2-proxy |
| `ingressroute.yaml` | Traefik IngressRoute for sdlc.threesix.ai with TLS |

## Review Checklist

### Correctness

- [x] All YAML validates (`kubectl apply --dry-run=client` passes for core resources; Traefik CRDs validate via yq)
- [x] Namespace `sdlc-hub` is consistent across all resources
- [x] Selector labels match between Deployments and Services
- [x] oauth2-proxy env vars reference the correct Secret name (`oauth2-proxy-credentials`) and keys
- [x] IngressRoute routes `/oauth2/*` directly to oauth2-proxy without forward-auth (prevents redirect loop)
- [x] IngressRoute routes all other traffic through `sdlc-hub-auth` middleware to `sdlc-hub` service
- [x] TLS references `sdlc-wildcard-tls` (provided by fleet-ingress-tls)

### Security

- [x] No credentials or secrets committed to git
- [x] oauth2-proxy credentials loaded from k8s Secret via `secretKeyRef`
- [x] Cookie set to `secure=true`
- [x] Forward-auth passes `X-Auth-Request-User` and `X-Auth-Request-Email` headers
- [x] Allowed email domains restricted to three specific domains

### Resource Limits

- [x] sdlc-hub: 50m/64Mi requests, 250m/128Mi limits — appropriate for hub (no agent runs)
- [x] oauth2-proxy: 10m/32Mi requests, 100m/64Mi limits — lightweight proxy
- [x] Both have liveness and readiness probes

### Findings

1. **SDLC_ROOT directory**: Hub deployment uses `SDLC_ROOT=/tmp/sdlc-hub`. The hub mode creates a minimal `.sdlc/` directory structure on startup even though it does not serve project state. Using `/tmp` is acceptable since the hub state file (`~/.sdlc/hub-state.yaml`) is the only persistent data, and it's ephemeral by design (rebuilt from heartbeats).
   - **Action:** Accept — intentional design choice documented in the spec.

2. **Image pull policy**: No `imagePullPolicy` specified, defaulting to `Always` for `:latest` tag. This is correct for a continuously-deployed service.
   - **Action:** Accept — default behavior is appropriate.

3. **Secret creation is manual**: The `oauth2-proxy-credentials` Secret is created via `kubectl create secret generic`, not via a manifest. This is intentional — credentials should not be in git.
   - **Action:** Accept — documented in T2.

4. **IngressRoute route ordering**: The `/oauth2` PathPrefix route is listed before the catch-all `Host` route. Traefik evaluates routes by specificity (longest match), so ordering in the YAML is not strictly important, but having it first improves readability.
   - **Action:** Accept — correct and readable.

5. **No NetworkPolicy**: There is no NetworkPolicy restricting traffic to the hub namespace. This means any pod in the cluster can reach the hub services directly.
   - **Action:** Track — create a follow-up task for NetworkPolicy if needed for defense-in-depth.

## Verdict

**Approved.** All manifests are correct, well-structured, and follow existing fleet conventions. The oauth2-proxy + Traefik forward-auth pattern is a standard, well-documented approach. No blocking issues found.
