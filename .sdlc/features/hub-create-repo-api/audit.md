# Security Audit: hub-create-repo-api

## Surface

New endpoint `POST /api/hub/create-repo` on the sdlc-server, hub mode only. Creates a Gitea repo and returns authenticated push credentials.

## Findings

### F1 — Admin token in push URL response [ACCEPTED]

**Finding:** The `push_url` in the response contains the Gitea admin token in HTTP basic auth: `http://claude-agent:<GITEA_TOKEN>@host/org/repo.git`

**Risk:** Anyone who receives this response can push to any repo in the `orchard9/` org using the admin token. The token is not scoped to a single repo.

**Accepted because:**
- This endpoint is behind the hub's Google OAuth auth gate (same as all `/api/hub/*` endpoints)
- The hub is a single-operator tool (Jordan's personal cluster)
- The response is served over HTTPS in production (Traefik TLS termination)
- The token is not logged (no tracing call on the push_url value)
- Documented in spec as a known limitation with upgrade path to per-repo deploy keys

**Action:** None required for current use case. Track as future work if multi-user access is added.

### F2 — Name validation [PASS]

Input `name` is validated against `^[a-z0-9][a-z0-9-]*$` (max 100 chars) before being sent to the Gitea API. No injection surface — name is passed as a JSON body field, not interpolated into a URL path. Shell/SQL injection not applicable.

### F3 — Hub mode guard [PASS]

Handler returns 503 if `app.hub_registry.is_none()` — not exploitable in project mode. Gitea config guard also returns 503 before any Gitea calls if credentials are absent.

### F4 — No SSRF surface [PASS]

`create_gitea_repo` and `get_gitea_username` call fixed Gitea endpoints using the server-configured `app.gitea_url`. No user-supplied URL is fetched.

### F5 — Token not logged [PASS]

`gitea_token` is used in the Authorization header and push URL but is not passed to any `tracing::` macro. The warning log on provision failure uses `%e` (error display) which does not contain the token.

## Verdict

APPROVED. One accepted finding (admin token in response) with documented rationale. All other surface areas are clean.
