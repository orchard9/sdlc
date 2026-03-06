# Design: fleet-management-api

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│  Hub Mode sdlc-server (sdlc.threesix.ai)            │
│                                                     │
│  ┌──────────────┐  ┌─────────────┐  ┌────────────┐ │
│  │ routes/hub.rs│  │  hub.rs     │  │ fleet.rs   │ │
│  │ (HTTP layer) │→ │ (registry)  │  │ (k8s+gitea)│ │
│  └──────────────┘  └─────────────┘  └────────────┘ │
│         │                                  ↕        │
│         └──────────────────────────────────┘        │
└──────────────┬────────────┬────────────┬────────────┘
               │            │            │
        ┌──────▼──────┐ ┌──▼────────┐ ┌▼──────────────┐
        │ k8s API     │ │ Gitea API │ │ Woodpecker API │
        │ (in-cluster)│ │ (internal)│ │ (CI trigger)   │
        └─────────────┘ └───────────┘ └────────────────┘
```

## Module Layout

### `crates/sdlc-server/src/fleet.rs` (new)

Core fleet data-fetching logic. Pure functions that take client references and return data types. No axum dependency.

```rust
pub struct FleetInstance {
    pub slug: String,
    pub namespace: String,
    pub url: String,
    pub deployment_status: DeploymentStatus,  // Running, Pending, Failed, Unknown
    pub pod_healthy: bool,
    pub created_at: DateTime<Utc>,
    // Merged from HubRegistry (optional — may not have heartbeat data)
    pub active_milestone: Option<String>,
    pub feature_count: Option<u32>,
    pub agent_running: Option<bool>,
}

pub enum DeploymentStatus { Running, Pending, Failed, Unknown }

pub struct GiteaRepo {
    pub slug: String,
    pub full_name: String,
    pub description: Option<String>,
    pub clone_url: String,
    pub created_at: DateTime<Utc>,
}

pub struct AvailableRepo {
    pub repo: GiteaRepo,
    pub can_provision: bool,
}
```

**Functions:**

- `list_fleet_instances(kube_client, hub_registry) -> Result<Vec<FleetInstance>>`
  - Lists namespaces with label selector or prefix `sdlc-`
  - Filters out excluded namespaces (`sdlc-tls`, `sdlc-hub`)
  - Gets deployment status from `apps/v1/deployments` in each namespace
  - Merges heartbeat data from HubRegistry by matching `sdlc-{slug}` namespace to registry URL
  
- `list_gitea_repos(http_client, gitea_url, token) -> Result<Vec<GiteaRepo>>`
  - Paginated fetch: `GET {gitea_url}/api/v1/orgs/orchard9/repos?limit=50&page=N`
  - Stops when page returns fewer than limit
  
- `list_available_repos(instances, repos) -> Vec<AvailableRepo>`
  - Pure diff: repos where no instance has matching `slug`
  - `can_provision: true` unless repo is archived

- `trigger_provision(http_client, woodpecker_url, token, repo_slug) -> Result<()>`
  - `POST {woodpecker_url}/api/repos/{owner}/{repo}/pipelines` with variables
  
- `import_repo(http_client, gitea_url, token, clone_url, repo_name, auth_token) -> Result<GiteaRepo>`
  - `POST {gitea_url}/api/v1/repos/migrate`
  - Body: `{ clone_addr, repo_name, repo_owner: "orchard9", service: "git", mirror: false }`

### `crates/sdlc-server/src/routes/hub.rs` (extended)

New route handlers added to existing hub routes module:

- `fleet()` — GET /api/hub/fleet
- `repos()` — GET /api/hub/repos
- `available()` — GET /api/hub/available
- `provision()` — POST /api/hub/provision
- `import()` — POST /api/hub/import
- `agents()` — GET /api/hub/agents

All handlers check `app.hub_registry.is_some()` and return 503 if not in hub mode (same pattern as existing handlers).

### `crates/sdlc-server/src/lib.rs` (extended)

Register new routes in `build_router_from_state`:

```rust
// Hub fleet management (alongside existing hub routes)
.route("/api/hub/fleet", get(routes::hub::fleet))
.route("/api/hub/repos", get(routes::hub::repos))
.route("/api/hub/available", get(routes::hub::available))
.route("/api/hub/provision", post(routes::hub::provision))
.route("/api/hub/import", post(routes::hub::import))
.route("/api/hub/agents", get(routes::hub::agents))
```

### AppState Extensions

```rust
// In state.rs — add to AppState
pub kube_client: Option<kube::Client>,     // None when not in k8s
pub gitea_url: Option<String>,              // From GITEA_URL env
pub gitea_token: Option<String>,            // From GITEA_API_TOKEN env
pub woodpecker_url: Option<String>,         // From WOODPECKER_URL env
pub woodpecker_token: Option<String>,       // From WOODPECKER_API_TOKEN env
```

Initialized in `new_with_port_hub()`:
- `kube::Client::try_default()` — succeeds in-cluster, returns `None` outside k8s
- Env vars read with `std::env::var().ok()` — missing is not fatal

## k8s Client Strategy

Use the `kube` crate (already available in the Rust ecosystem, well-maintained):

```toml
# Cargo.toml additions for sdlc-server
kube = { version = "0.98", features = ["client", "runtime"] }
k8s-openapi = { version = "0.23", features = ["latest"] }
```

In-cluster auth via ServiceAccount token mounted at `/var/run/secrets/kubernetes.io/serviceaccount/token`. The `kube::Client::try_default()` detects this automatically.

**Graceful fallback:** When running locally (dev), `kube::Client::try_default()` fails. The `kube_client` field is `None`, and fleet endpoints return `{ "instances": [], "total": 0, "warning": "k8s not available" }`.

## Namespace Filtering

Excluded namespaces (hardcoded constant):
```rust
const EXCLUDED_NAMESPACES: &[&str] = &["sdlc-tls", "sdlc-hub", "sdlc-system"];
```

Detection logic: namespace starts with `sdlc-` AND is not in the exclusion list AND has at least one deployment with label `app.kubernetes.io/name: sdlc-server`.

## HTTP Client for Gitea/Woodpecker

Use `reqwest::Client` (already a dependency of sdlc-server). Store a shared `reqwest::Client` in AppState for connection pooling.

## Service Token Auth

Extend `auth.rs` to check for `Authorization: Bearer <token>` header. If present and matches a configured service token (env `HUB_SERVICE_TOKENS`, comma-separated), the request is authenticated without oauth2-proxy cookies. This allows machine-to-machine API access.

## SSE Events

New hub SSE event variants:
- `FleetProvisionStarted { slug: String }` — emitted when provision is triggered
- `FleetImportStarted { repo_name: String }` — emitted when import is triggered

Added to existing `HubSseMessage` enum in `hub.rs`.

## Error Handling Strategy

All external API calls (k8s, Gitea, Woodpecker) use `?` with a custom `FleetError` that maps to appropriate HTTP status codes:

| Source | HTTP Status | Error Code |
|--------|-------------|------------|
| k8s API unreachable | 502 | `k8s_unavailable` |
| Gitea API unreachable | 502 | `gitea_unavailable` |
| Woodpecker API unreachable | 502 | `woodpecker_unavailable` |
| Repo not found | 404 | `repo_not_found` |
| Invalid import URL | 400 | `invalid_url` |
| Not in hub mode | 503 | `not_hub_mode` |

## RBAC Manifest

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: sdlc-hub
  namespace: sdlc-hub
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: sdlc-hub-reader
rules:
  - apiGroups: [""]
    resources: ["namespaces", "pods"]
    verbs: ["list"]
  - apiGroups: ["apps"]
    resources: ["deployments"]
    verbs: ["list", "get"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: sdlc-hub-reader
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: sdlc-hub-reader
subjects:
  - kind: ServiceAccount
    name: sdlc-hub
    namespace: sdlc-hub
```

## Testing Strategy

- **Unit tests** in `fleet.rs`: mock k8s/Gitea responses, test namespace filtering, diff logic
- **Integration tests**: use `build_router_for_test` with hub mode, verify 503 in project mode, verify JSON shapes
- No live k8s/Gitea tests in CI — those are covered by milestone UAT
