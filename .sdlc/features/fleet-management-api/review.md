# Review: fleet-management-api

## Summary

Implemented 6 new hub API endpoints for fleet management (`/api/hub/fleet`, `/api/hub/repos`, `/api/hub/available`, `/api/hub/provision`, `/api/hub/import`, `/api/hub/agents`) plus supporting infrastructure (kube client, env var wiring, RBAC manifests, service token auth).

## Files Changed

| File | Change |
|------|--------|
| `crates/sdlc-server/src/fleet.rs` | **New** — Core fleet types and logic: k8s namespace discovery, Gitea API, provisioning, import, error types |
| `crates/sdlc-server/src/routes/hub.rs` | **Extended** — 6 new route handlers appended to existing hub routes |
| `crates/sdlc-server/src/state.rs` | **Extended** — 8 new AppState fields (kube_client, gitea/woodpecker URLs/tokens, hub_service_tokens, ingress_domain) |
| `crates/sdlc-server/src/lib.rs` | **Extended** — `pub mod fleet;` added, 6 new routes wired |
| `crates/sdlc-server/Cargo.toml` | **Extended** — `kube`, `k8s-openapi`, `reqwest` json feature added |
| `k3s-fleet/deployments/hub/rbac.yaml` | **New** — ServiceAccount, ClusterRole, ClusterRoleBinding |
| `k3s-fleet/deployments/hub/sdlc-hub-deployment.yaml` | **Updated** — serviceAccountName, env vars for Gitea/Woodpecker/service tokens |

## Findings

### F1: No `unwrap()` in library code [PASS]
All error paths use `?`, `match`, or graceful fallback. The `kube::Config::incluster()` failure is handled by setting `kube_client = None`.

### F2: Graceful degradation outside k8s [PASS]
When running outside k8s (dev environment), `kube_client` is `None`. Fleet endpoint returns empty instances with a warning field. No 500 errors.

### F3: Namespace filtering [PASS]
`EXCLUDED_NAMESPACES` constant excludes `sdlc-tls`, `sdlc-hub`, `sdlc-system`. Additionally, namespaces without a `sdlc-server` deployment are filtered via label selector. Unit test covers this.

### F4: Paginated Gitea fetching [PASS]
`list_gitea_repos` paginates with `limit=50&page=N`, stopping when a page returns fewer than limit items.

### F5: Input validation [PASS]
- `provision`: Checks `repo_slug` non-empty, validates repo exists in Gitea (graceful if Gitea is down)
- `import`: Checks `clone_url` and `repo_name` non-empty, validates URL starts with `http://` or `https://`

### F6: Service token auth [PASS]
`HUB_SERVICE_TOKENS` env var is loaded into the auth token list during `build_base_state`. Bearer auth already existed in `auth.rs`; no middleware changes needed.

### F7: Error response consistency [PASS]
All fleet errors map through `FleetError` with consistent `{ "error": "<code>", "detail": "<message>" }` JSON shape and appropriate HTTP status codes (400, 404, 502, 503).

### F8: Hub mode gating [PASS]
All new endpoints check `app.hub_registry.is_none()` and return 503 in project mode, consistent with existing hub endpoints.

### F9: RBAC scope [PASS]
ClusterRole grants read-only access (list namespaces/pods, list/get deployments). No write permissions. Follows principle of least privilege.

### F10: Potential improvement — k8s client init is synchronous [NOTE]
`kube::Config::incluster()` and `Client::try_from` are called synchronously in `new_with_port_hub`. This is fast (reads a file from ServiceAccount mount), but could be moved to an async init if it ever becomes a concern. Acceptable for now.

## Verdict

All findings pass. Code is clean, well-structured, and follows existing patterns. No blockers.
