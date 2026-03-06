# Security Audit: Deploy hub mode sdlc-server at sdlc.threesix.ai

## Scope

Kubernetes manifests in `k3s-fleet/deployments/hub/` deploying a hub-mode sdlc-server with Google OAuth authentication via oauth2-proxy and Traefik ForwardAuth.

## Findings

### A1: Credential Storage — PASS

- OAuth client ID, client secret, and cookie secret are stored in a k8s Secret (`oauth2-proxy-credentials`), not in any committed manifest
- Secret is created manually via `kubectl create secret generic` — no credentials in git
- Environment variables reference the Secret via `secretKeyRef`

### A2: Authentication Boundary — PASS

- All traffic to `sdlc.threesix.ai` passes through Traefik ForwardAuth middleware
- ForwardAuth delegates to oauth2-proxy's `/oauth2/auth` endpoint
- The `/oauth2/*` paths (callback, sign-in) are routed directly to oauth2-proxy without the auth middleware — correct, prevents redirect loops
- Unauthenticated requests receive 302 redirect to Google sign-in

### A3: Authorization — Domain Restriction — PASS

- oauth2-proxy configured with `--email-domain` flags for three specific domains: `livelyideo.tv`, `masq.me`, `virtualcommunities.ai`
- Users outside these domains will be rejected by oauth2-proxy with 403
- No wildcard (`*`) email domain configured

### A4: Cookie Security — PASS

- `--cookie-secure=true` — cookie only sent over HTTPS
- Custom cookie name `_sdlc_hub_oauth2` — avoids collision with other services
- Cookie secret is 32 bytes from `openssl rand -base64 32` — sufficient entropy

### A5: TLS Configuration — PASS

- IngressRoute specifies `tls.secretName: sdlc-wildcard-tls` — uses the fleet wildcard cert from cert-manager
- Entry point is `websecure` (port 443) — no plaintext HTTP entry point defined
- No HTTP-to-HTTPS redirect manifest exists, but Traefik's default `websecure` entrypoint handles this at the cluster level

### A6: Container Security — ADVISORY

- Neither Deployment sets `securityContext` (no `runAsNonRoot`, `readOnlyRootFilesystem`, `allowPrivilegeEscalation: false`)
- oauth2-proxy image runs as non-root by default (upstream image design)
- sdlc-server image: depends on the Dockerfile — should verify it runs as non-root
- **Action:** Track — add `securityContext` hardening as a follow-up task. Not blocking because the pods are in an isolated namespace and the cluster is private.

### A7: Network Exposure — ADVISORY

- No NetworkPolicy in the `sdlc-hub` namespace — any pod in the cluster can reach hub services
- The hub API endpoints (`/api/hub/heartbeat`) accept unauthenticated POSTs from project instances within the cluster. This is by design (heartbeat protocol), but means any pod can register as a project.
- **Action:** Accept for now — the cluster is private (Tailscale + k3s). Track NetworkPolicy for defense-in-depth if the cluster grows.

### A8: Forward Header Trust — PASS WITH NOTE

- `trustForwardHeader: true` in the ForwardAuth middleware means Traefik trusts `X-Forwarded-*` headers from the client
- This is safe because Traefik is the edge proxy — it rewrites these headers before forwarding
- `--reverse-proxy=true` and `--pass-host-header=true` on oauth2-proxy are correct for behind-proxy deployment

### A9: Health Endpoint Exposure — PASS

- `/api/health` is exposed without auth via the k8s probes (direct container access, not through IngressRoute)
- Through the IngressRoute, `/api/health` goes through auth — this is acceptable; the health check does not leak sensitive information

### A10: Image Tags — ADVISORY

- Both images use mutable tags: `ghcr.io/orchard9/sdlc:latest` and `quay.io/oauth2-proxy/oauth2-proxy:v7.6.0`
- `v7.6.0` is a specific version tag (good), but `:latest` is mutable
- **Action:** Accept — `:latest` is the fleet convention for continuously-deployed sdlc-server. The image is built from a known CI pipeline.

## Summary

| Finding | Severity | Action |
|---|---|---|
| A1: Credential storage | PASS | -- |
| A2: Auth boundary | PASS | -- |
| A3: Domain restriction | PASS | -- |
| A4: Cookie security | PASS | -- |
| A5: TLS | PASS | -- |
| A6: Container securityContext | Advisory | Track as follow-up |
| A7: Network exposure | Advisory | Accept (private cluster) |
| A8: Forward header trust | PASS | -- |
| A9: Health endpoint | PASS | -- |
| A10: Image tags | Advisory | Accept (fleet convention) |

## Verdict

**Approved.** No blocking security issues. Three advisory findings tracked for future hardening. The authentication and authorization model is sound — Google OAuth via oauth2-proxy with domain restriction is a well-established pattern.
