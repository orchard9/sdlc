# Audit: fleet-management-api

## Scope

Security audit of hub fleet management API: 6 new HTTP endpoints, k8s RBAC, Gitea/Woodpecker API token handling, service token authentication.

## Findings

### A1: API tokens in environment variables [ACCEPTABLE]

**Risk:** Gitea, Woodpecker, and hub service tokens are passed via env vars (`GITEA_API_TOKEN`, `WOODPECKER_API_TOKEN`, `HUB_SERVICE_TOKENS`). These are read from k8s Secrets in the deployment manifest.

**Assessment:** Standard k8s pattern. Secrets are referenced via `secretKeyRef`, not hardcoded. The tokens are never logged or included in API responses. No action needed.

### A2: k8s RBAC scope [PASS]

**Risk:** Overly broad cluster permissions could allow the hub pod to access sensitive resources.

**Assessment:** ClusterRole `sdlc-hub-reader` grants only:
- `namespaces`: list
- `pods`: list
- `deployments`: list, get

No write access, no access to secrets, configmaps, or other sensitive resources. Follows least privilege.

### A3: Service token auth bypass [ACCEPTABLE]

**Risk:** `HUB_SERVICE_TOKENS` allows machine-to-machine API access without browser OAuth.

**Assessment:** Tokens are loaded into the same auth token list as named tokens from `auth.yaml`. They go through the existing `auth_middleware` Bearer token check. No new auth code path introduced. Tokens should be rotated periodically by updating the k8s Secret. Tracked as a future operational concern, not a code issue.

### A4: Gitea API token forwarded in requests [PASS]

**Risk:** Token could leak through error messages or logs.

**Assessment:** The token is sent only in the `Authorization` header to the Gitea API. Error responses from Gitea are wrapped in `FleetError` with a generic message; raw response bodies are included in the `detail` field but this is internal error context, not user-facing token data. No token values are ever serialized into responses.

### A5: Import endpoint accepts arbitrary URLs [LOW RISK]

**Risk:** `POST /api/hub/import` accepts a `clone_url` that is forwarded to Gitea's migrate API. A malicious URL could cause SSRF through Gitea.

**Assessment:** The URL is forwarded to Gitea, which itself performs the git clone. Gitea has its own SSRF protections. Our validation checks `http://` or `https://` prefix, blocking `file://`, `ssh://`, etc. The endpoint is behind auth (OAuth or service token), so unauthenticated access is impossible. Risk is low and mitigated by auth + URL scheme validation.

**Action:** Accept — Gitea's own SSRF protections are the correct layer for this.

### A6: No rate limiting on provision/import [LOW RISK]

**Risk:** An authenticated user could spam `POST /api/hub/provision` to create many instances.

**Assessment:** Both endpoints are behind auth. Provisioning goes through Woodpecker, which has its own pipeline queuing. Import goes through Gitea, which has its own rate limits. No additional rate limiting needed at the hub level for v1.

**Action:** Accept for v1. Track as future improvement if abuse is observed.

### A7: Namespace injection via repo_slug [PASS]

**Risk:** A malicious `repo_slug` in provision request could target arbitrary namespaces.

**Assessment:** The repo_slug is passed as a Woodpecker pipeline variable (`PROVISION_SLUG`), not used directly in k8s API calls. The fleet-reconcile pipeline controls namespace creation. The slug is also validated against Gitea repos when Gitea is reachable. No injection vector exists.

## Verdict

No blockers. All security concerns are addressed through existing infrastructure (k8s RBAC, auth middleware, Gitea protections) or are acceptable low-risk items for v1.
