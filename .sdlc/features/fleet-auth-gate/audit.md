# Security Audit: fleet-auth-gate

## Scope

This feature introduces Google OAuth authentication gating for all `*.sdlc.threesix.ai` project instances via Traefik forward-auth middleware and oauth2-proxy. Files audited:

- `k3s-fleet/deployments/helm/sdlc-server/templates/middleware-google-auth.yaml`
- `k3s-fleet/deployments/helm/sdlc-server/templates/ingressroute.yaml`
- `k3s-fleet/deployments/helm/sdlc-server/templates/service.yaml`
- `k3s-fleet/deployments/helm/sdlc-server/values.yaml`
- `k3s-fleet/deployments/hub/oauth2-proxy-deployment.yaml`

## Findings

### A1: Cookie domain allows cross-subdomain access (INFORMATIONAL)

`--cookie-domain=.sdlc.threesix.ai` means any subdomain under `sdlc.threesix.ai` can read the session cookie. This is intentional for SSO but means a compromised project instance could read session tokens from other instances.

**Risk:** Low. All project instances run the same sdlc-server binary in the same cluster. A compromised instance already implies cluster access.

**Action:** Accepted. Cross-subdomain SSO is a design requirement.

### A2: Email domain allowlist is the only access control (INFORMATIONAL)

Access is gated by Google email domains: `livelyideo.tv`, `masq.me`, `virtualcommunities.ai`. Any user with an account on these domains can access all project instances. There is no per-project or per-user access control.

**Risk:** Low for current team size. Would need revisiting if the team grows or external collaborators are added.

**Action:** Accepted. Per-user access control is explicitly out of scope (documented in the exploration).

### A3: Bearer token passthrough bypasses Google OAuth (MEDIUM)

`--pass-authorization-header=true` forwards bearer tokens through oauth2-proxy to sdlc-server. The sdlc-server `auth.rs` validates bearer tokens against `auth.yaml` tokens. If `auth.yaml` has no tokens configured (open mode), bearer auth is effectively disabled and oauth2-proxy is the sole gate.

**Risk:** Medium. In open mode (no `auth.yaml` tokens), requests with any `Authorization: Bearer` header would be passed through oauth2-proxy but sdlc-server would accept all requests. However, oauth2-proxy still validates the request first -- `--pass-authorization-header` only forwards the header, it does not skip auth validation.

**Action:** Verified. oauth2-proxy still requires a valid Google session OR the forward-auth subrequest to pass. The `--pass-authorization-header` flag only controls header forwarding, not auth bypass. Bearer-only access without a Google session requires the forward-auth middleware to be configured to allow it (which it is not -- it always checks). The sdlc-server bearer check is a second layer, not a bypass. No issue.

### A4: oauth2-proxy credentials in Kubernetes Secret (INFORMATIONAL)

`client-id`, `client-secret`, and `cookie-secret` are stored in a Kubernetes Secret `oauth2-proxy-credentials`. These are not committed to the repo -- they must be created manually or via ExternalSecrets.

**Risk:** Low. Standard Kubernetes secret management pattern.

**Action:** Accepted.

### A5: TLS termination at Traefik (INFORMATIONAL)

The IngressRoute uses `tls.secretName: sdlc-wildcard-tls`. Traffic between Traefik and the sdlc-server pod is unencrypted within the cluster (port 80 -> 8080).

**Risk:** Low. In-cluster traffic between pods in the same cluster is standard. mTLS (via service mesh) would add encryption but is not required for this threat model.

**Action:** Accepted. Standard k8s pattern.

### A6: cookie-samesite=lax prevents CSRF (POSITIVE)

`SameSite=Lax` prevents the session cookie from being sent on cross-origin POST requests, mitigating CSRF attacks. Combined with `cookie-secure=true` (HTTPS only), the cookie configuration follows security best practices.

**Action:** No issue. Positive finding.

### A7: Session refresh before expiry prevents session fixation (POSITIVE)

`--cookie-refresh=1h0m0s` with `--cookie-expire=24h0m0s` means the session cookie is silently refreshed every hour. This limits the window for session theft and prevents stale sessions from persisting indefinitely.

**Action:** No issue. Positive finding.

## Verdict

No critical or high-severity findings. The auth gate implementation follows established patterns (Traefik forward-auth + oauth2-proxy) with appropriate cookie security settings. The medium-severity bearer passthrough finding (A3) was investigated and found to be a non-issue -- oauth2-proxy does not bypass auth validation when forwarding headers.
