# Exploration: Fleet Control Plane with Google Auth

## Problem

Right now the sdlc cluster has one live instance at `sdlc.sdlc.threesix.ai` with no
authentication. There is no way to:

- Log in as an authorized user
- See what sdlc project instances are running across the fleet
- Start a new instance for a repo that doesn't have one yet
- Import an external git repo into the fleet

The hub mode (v37) was built for local navigation but deployed at `sdlc.sdlc.threesix.ai`
it's a project workspace, not a control plane. There is nothing at `sdlc.threesix.ai`.

## What we want

A control plane at `sdlc.threesix.ai` that:

1. **Google OAuth gate** — log in with a `@livelyideo.tv`, `@masq.me`, or
   `@virtualcommunities.ai` Google account. No anonymous access.

2. **Running instances view** — see all live `sdlc-*` deployments in the cluster
   with their status (healthy, starting, offline). Click to navigate to that instance.

3. **Startable instances view** — see all repos in the `orchard9` Gitea org that
   don't have a running instance. A "Start" action provisions one via the v18
   fleet-reconcile pipeline.

4. **Import external git** — paste a GitHub/GitLab URL + optional PAT. The hub
   calls Gitea's migrate API, imports the repo into `orchard9`, and the fleet
   reconcile picks it up automatically.

## Key technical decisions

### Auth: oauth2-proxy, not in-app

The cluster already runs oauth2-proxy in the `auth` namespace for `masq-ops.orchard9.ai`.
The same pattern applies: Traefik middleware routes all traffic through oauth2-proxy
first, which validates Google session cookies, then forwards to the hub. The hub app
itself does not handle auth — it's a Kubernetes-level concern.

Allowed Google domains: `livelyideo.tv`, `masq.me`, `virtualcommunities.ai`

This also means individual project instances (`sdlc-<project>.threesix.ai`) need the
same gate — otherwise someone who guesses a URL bypasses auth.

### Control plane surface: hub mode extended

The v37 hub mode (heartbeat registry, project cards, SSE) is the right foundation.
The hub needs two additions:

1. **Fleet API** — talk to the k8s API (or Woodpecker API) to list deployments and
   trigger new ones. The hub pod needs a ServiceAccount with read access to namespaces
   and deployments, plus the ability to POST to Woodpecker to trigger reconcile.

2. **Gitea integration** — the hub calls `GET /api/v1/orgs/orchard9/repos` to get all
   repos, diffs against running deployments to compute the "startable" list.

### Starting an instance: Woodpecker webhook trigger

The v18 fleet-reconcile pipeline (`reconcile-projects.yaml`) already knows how to
provision a single deployment given a repo slug. The hub sends a webhook POST to
Woodpecker's API to trigger that pipeline with the repo slug as a parameter.
This is async — hub shows "provisioning" status and the SSE stream updates when it
comes online (heartbeat from new instance).

Simpler than the hub calling k8s directly — avoids needing cluster-admin RBAC in
the hub pod.

### Import flow: Gitea migrate API

```
POST /api/v1/repos/migrate
{
  "repo_name": "<slug>",
  "owner": "orchard9",
  "clone_addr": "<source URL>",
  "auth_token": "<user PAT, optional>"
}
```

Once imported, the repo appears in the Gitea org and the reconcile pipeline picks it
up on the next scheduled run (or the hub triggers an immediate reconcile).

### Domain layout

```
sdlc.threesix.ai              — hub / control plane (new, needs deployment)
sdlc-<project>.threesix.ai   — individual project instances (v18 pattern)
sdlc.sdlc.threesix.ai        — the sdlc project's own instance (existing)
```

`sdlc.threesix.ai` is a new ingress — hub mode ponder binary with `SDLC_HUB=true`,
behind oauth2-proxy middleware.

## What exists today

| Component | Status |
|---|---|
| Hub mode binary (v37) | Shipped — heartbeat registry, project cards, SSE |
| Fleet provisioning pipeline (v18) | Code done, UAT not passed |
| Wildcard TLS `*.sdlc.threesix.ai` (v18) | Code done, not deployed |
| Credential pool (v38) | In progress — needed for agent runs in fleet |
| oauth2-proxy for Google auth | Pattern exists in `auth` ns, not wired to sdlc |
| `sdlc.threesix.ai` ingress | Does not exist |
| Fleet management UI | Does not exist |
| Import flow | Does not exist |

## Scope boundary

This initiative does NOT include:
- Per-user workspaces or multi-tenancy (all authenticated users see all projects)
- Billing or usage metering
- Stopping/deleting instances from the UI (too destructive for v1)
- Per-project access control

## Dependencies

- v18 must UAT-pass before fleet management UI is meaningful
- v38 credential-pool-helm must ship for agent runs in fleet instances to work
- The Google OAuth client ID/secret must be provisioned in GCP and stored as a cluster secret
