# Tasks: fleet-management-api

## T1: GET /api/hub/fleet — k8s namespace listing

Create `fleet.rs` with `list_fleet_instances()`. Query k8s API via `kube` crate to list `sdlc-*` namespaces, get deployment status for each, merge with HubRegistry heartbeat data. Add the `fleet()` route handler in `routes/hub.rs`. Wire route in `lib.rs`.

## T2: GET /api/hub/repos — Gitea org repo listing

Add `list_gitea_repos()` to `fleet.rs`. Paginated fetch from Gitea API (`/api/v1/orgs/orchard9/repos`). Add `repos()` route handler. Uses `reqwest::Client` with `GITEA_API_TOKEN` bearer auth.

## T3: GET /api/hub/available — diff fleet vs repos

Add `list_available_repos()` pure function to `fleet.rs`. Diffs fleet instances against Gitea repos to find repos without running instances. Add `available()` route handler.

## T4: POST /api/hub/provision — trigger Woodpecker pipeline

Add `trigger_provision()` to `fleet.rs`. Calls Woodpecker API to start fleet-reconcile pipeline for a specific repo slug. Add `provision()` route handler with request validation. Emit `FleetProvisionStarted` SSE event.

## T5: POST /api/hub/import — Gitea migrate + provision

Add `import_repo()` to `fleet.rs`. Calls Gitea migrate API to import external repo into orchard9 org, then triggers provision. Add `import()` route handler with URL validation.

## T6: Wire API tokens into hub deployment

Add `GITEA_API_TOKEN`, `GITEA_URL`, `WOODPECKER_API_TOKEN`, `WOODPECKER_URL` env vars to hub deployment. Create k8s Secret manifest or reference existing secrets. Update AppState initialization in `new_with_port_hub()` to read these env vars.

## T7: ServiceAccount + RBAC for hub pod

Create ClusterRole `sdlc-hub-reader` with read-only access to namespaces, deployments, and pods. Create ServiceAccount `sdlc-hub` and ClusterRoleBinding. Add `serviceAccountName` to hub deployment template.

## T8: [user-gap] Service token auth bypass

Extend `auth.rs` to accept `Authorization: Bearer <token>` for machine-to-machine API access. Check against `HUB_SERVICE_TOKENS` env var (comma-separated). Allows programmatic fleet management without browser session.

## T9: [user-gap] Namespace filter

Implement `EXCLUDED_NAMESPACES` constant in `fleet.rs`. Filter out `sdlc-tls`, `sdlc-hub`, `sdlc-system` and any namespace without a `sdlc-server` deployment. Ensure only actual project instances appear in fleet listing.

## T10: [user-gap] Fleet-wide agent status endpoint

Add `GET /api/hub/agents` returning aggregate active agent run count across all instances from HubRegistry. Add `agents()` route handler. Returns `{ active_count, active_projects }`.
