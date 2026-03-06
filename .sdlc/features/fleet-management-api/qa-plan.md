# QA Plan: fleet-management-api

## Build Verification

1. `SDLC_NO_NPM=1 cargo test --all` passes — no regressions from new `kube`/`k8s-openapi` dependencies
2. `cargo clippy --all -- -D warnings` passes — no new warnings
3. `fleet.rs` compiles without `unwrap()` in library paths

## Unit Tests (fleet.rs)

### Namespace filtering
- **T-FILTER-1**: `list_available_repos()` with 5 repos and 2 instances returns 3 available
- **T-FILTER-2**: Excluded namespaces (`sdlc-tls`, `sdlc-hub`) are filtered out even when present
- **T-FILTER-3**: Namespace without `sdlc-server` deployment is excluded
- **T-FILTER-4**: Archived repos have `can_provision: false`

### Data merging
- **T-MERGE-1**: Fleet instance with matching HubRegistry entry gets heartbeat fields merged
- **T-MERGE-2**: Fleet instance without HubRegistry entry has `None` for heartbeat fields
- **T-MERGE-3**: HubRegistry entry without k8s namespace is not included in fleet listing

### Agent aggregation
- **T-AGENT-1**: Registry with 3 entries (2 with `agent_running: true`) returns `active_count: 2`
- **T-AGENT-2**: Empty registry returns `active_count: 0, active_projects: []`

## Integration Tests (routes)

### Hub mode gating
- **T-HUB-1**: All new endpoints return 503 when not in hub mode (project mode server)
- **T-HUB-2**: `/api/hub/fleet` returns 200 with empty `instances` array in hub mode without k8s

### Route registration
- **T-ROUTE-1**: GET `/api/hub/fleet` is routable (not 404)
- **T-ROUTE-2**: GET `/api/hub/repos` is routable
- **T-ROUTE-3**: GET `/api/hub/available` is routable
- **T-ROUTE-4**: POST `/api/hub/provision` is routable
- **T-ROUTE-5**: POST `/api/hub/import` is routable
- **T-ROUTE-6**: GET `/api/hub/agents` is routable

### Input validation
- **T-VAL-1**: POST `/api/hub/provision` with empty `repo_slug` returns 400
- **T-VAL-2**: POST `/api/hub/import` with invalid `clone_url` returns 400
- **T-VAL-3**: POST `/api/hub/import` with empty `repo_name` returns 400

### Error responses
- **T-ERR-1**: When Gitea is unreachable, `/api/hub/repos` returns 502 with `gitea_unavailable`
- **T-ERR-2**: JSON error responses have consistent shape: `{ "error": "code", "detail": "message" }`

## Service Token Auth

- **T-AUTH-1**: Request with valid `Authorization: Bearer <token>` is authenticated
- **T-AUTH-2**: Request with invalid bearer token returns 401
- **T-AUTH-3**: Request with no auth header and no oauth2-proxy cookie returns 401 (when tunnel auth is active)

## Manual / UAT Verification

These are covered by the milestone acceptance test (v42-fleet-control-plane) and are NOT automated in this feature's QA:
- Live k8s namespace listing against real cluster
- Live Gitea repo listing
- Live Woodpecker pipeline triggering
- OAuth2-proxy integration
- End-to-end provisioning flow
