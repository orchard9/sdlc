# Spec: fleet-management-api

## Summary

Hub API endpoints for fleet listing, provisioning, and repo import. When the sdlc server runs in hub mode (`sdlc.threesix.ai`), these endpoints let the fleet management UI (and machine callers) see all running instances, discover available repos, provision new instances, and import external repos.

## Context

The hub mode server already has a heartbeat-based project registry (`hub.rs`) that tracks project instances sending periodic heartbeats. This feature adds the infrastructure-aware layer: querying the k8s API for actual deployment state, querying Gitea for available repos, and triggering provisioning via Woodpecker CI pipelines.

## Endpoints

### GET /api/hub/fleet

Returns all sdlc project deployments from the k8s cluster, merged with heartbeat data from the HubRegistry.

- Queries the k8s API (in-cluster ServiceAccount) to list namespaces matching `sdlc-*`
- Excludes non-instance namespaces: `sdlc-tls`, `sdlc-hub`, and any namespace without a `sdlc-server` deployment
- For each namespace, reads deployment status (available replicas, conditions) and pod health
- Merges with HubRegistry heartbeat data (active milestone, feature count, agent running)
- Returns `FleetInstance[]` sorted by name

```json
{
  "instances": [
    {
      "slug": "sdlc",
      "namespace": "sdlc-sdlc",
      "url": "https://sdlc.sdlc.threesix.ai",
      "deployment_status": "running",
      "pod_healthy": true,
      "created_at": "2026-03-01T00:00:00Z",
      "active_milestone": "v42-fleet-control-plane",
      "feature_count": 12,
      "agent_running": true
    }
  ],
  "total": 1
}
```

`deployment_status`: `running` | `pending` | `failed` | `unknown`

### GET /api/hub/repos

Lists all repos in the `orchard9` Gitea org.

- Calls Gitea API: `GET /api/v1/orgs/orchard9/repos?limit=50&page=N` (paginated)
- Returns `GiteaRepo[]` with slug, full name, description, created_at, clone URL

### GET /api/hub/available

Returns repos from Gitea that do NOT have a running instance in the fleet.

- Calls `/api/hub/fleet` and `/api/hub/repos` internally
- Diffs: repos whose slug does not match any fleet namespace (`sdlc-{slug}`)
- Returns `AvailableRepo[]` with the same shape as repos, plus a `can_provision: bool` flag

### POST /api/hub/provision

Triggers provisioning of a new sdlc instance for a Gitea repo.

- Request: `{ "repo_slug": "my-project" }`
- Validates the repo exists in the orchard9 Gitea org
- Triggers the fleet-reconcile Woodpecker pipeline via API: `POST /api/repos/{owner}/{repo}/pipelines` with the repo slug as a parameter
- Returns `{ "status": "provisioning", "repo_slug": "my-project" }`
- Emits `FleetProvisionStarted { slug }` SSE event

### POST /api/hub/import

Imports an external git repo into Gitea, then triggers provisioning.

- Request: `{ "clone_url": "https://github.com/org/repo", "repo_name": "my-repo", "auth_token": null }`
- Calls Gitea migrate API: `POST /api/v1/repos/migrate` with `{ clone_addr, repo_name, repo_owner: "orchard9", mirror: false }`
- On success, triggers provision for the imported repo
- Returns `{ "status": "importing", "repo_name": "my-repo", "gitea_url": "..." }`

### GET /api/hub/agents

Returns aggregate active agent run counts across all fleet instances.

- Iterates HubRegistry entries where `agent_running == true`
- Returns `{ "active_count": 3, "active_projects": ["sdlc", "payments"] }`
- Used by the fleet dashboard header for "N agents running across M projects"

## Authentication

All `/api/hub/*` endpoints require authentication. In production, oauth2-proxy handles browser sessions. For machine-to-machine access, a bearer token in the `Authorization` header is checked against a configured service token list. The auth middleware (existing `auth.rs`) is extended to accept service tokens alongside oauth2-proxy cookies.

## Infrastructure Requirements

### ServiceAccount + RBAC

The hub pod needs a k8s ServiceAccount with read-only access:
- `namespaces`: list
- `deployments` (apps/v1): list, get (all namespaces)
- `pods` (v1): list (all namespaces)

ClusterRole + ClusterRoleBinding, not namespace-scoped.

### Environment Variables

The hub deployment needs these secrets as env vars:
- `GITEA_API_TOKEN` — from `THREE_SIX_GITEA` secret
- `WOODPECKER_API_TOKEN` — from `THREE_SIX_WOODPECKER` secret
- `GITEA_URL` — `http://gitea.threesix.svc.cluster.local` (internal cluster URL)
- `WOODPECKER_URL` — Woodpecker CI API base URL

### k8s Client

Uses the `kube` crate with in-cluster config (`Config::incluster()`). Falls back gracefully: if not running in a k8s pod, fleet endpoints return empty results with a warning (not 500).

## Error Handling

- Gitea API failures: return 502 with `{ "error": "gitea_unavailable", "detail": "..." }`
- k8s API failures: return 502 with `{ "error": "k8s_unavailable", "detail": "..." }`
- Woodpecker API failures on provision: return 502 with `{ "error": "woodpecker_unavailable" }`
- Repo not found on provision: return 404
- Import with invalid URL: return 400

## Non-goals

- Per-user access control (all authenticated users see all projects)
- Instance deletion or scaling from the API
- Real-time pod log streaming
- Health check probing of individual instances (heartbeat is sufficient)
