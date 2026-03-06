# Plan: Fleet Control Plane with Google Auth

## Milestone: v42-fleet-control-plane

**Vision:** Go to `sdlc.threesix.ai`, log in with Google, and see every sdlc project
instance in the fleet. Start new instances for any repo in Gitea. Import external git
repos and have them automatically provisioned. One authenticated entry point for the
entire sdlc fleet.

### Feature 1: `fleet-hub-deployment`

Deploy the hub mode ponder binary at `sdlc.threesix.ai` with Google OAuth via oauth2-proxy.

**Tasks:**
1. Create Google OAuth client ID in GCP for `sdlc.threesix.ai` — allowed redirect URIs, allowed domains (`livelyideo.tv`, `masq.me`, `virtualcommunities.ai`)
2. Store OAuth client ID + secret as k8s Secret in `sdlc-sdlc` namespace (or a new `sdlc-hub` namespace)
3. Deploy oauth2-proxy in the hub namespace — configured for Google provider, allowed email domains, cookie secret
4. Create Traefik IngressRoute for `sdlc.threesix.ai` with oauth2-proxy forward-auth middleware
5. Deploy hub mode sdlc-server (`SDLC_HUB=true`) at `sdlc.threesix.ai` — new deployment, not the existing `sdlc-sdlc` project instance
6. Add oauth2-proxy forward-auth middleware to all `sdlc-*.threesix.ai` ingresses — individual project instances are also auth-gated
7. Helm chart updates: new `hub` values block with oauth2-proxy config, forward-auth middleware template

### Feature 2: `fleet-management-api`

Server-side API in hub mode that lists running instances and startable repos, and triggers provisioning.

**Tasks:**
1. Add `GET /api/hub/fleet` endpoint — queries k8s API to list all `sdlc-*` namespaces with deployment status (running, pending, failed), pod health, and creation timestamp
2. Add `GET /api/hub/repos` endpoint — calls Gitea API (`GET /api/v1/orgs/orchard9/repos`) to list all repos in the org
3. Add `GET /api/hub/available` endpoint — diffs fleet (running) against repos (all) to return repos without a running instance
4. Add `POST /api/hub/provision` endpoint — accepts `{ repo_slug }`, triggers the fleet-reconcile Woodpecker pipeline via API (`POST /api/repos/{id}/pipelines`) with the repo slug as parameter
5. Add `POST /api/hub/import` endpoint — accepts `{ clone_url, repo_name, auth_token? }`, calls Gitea migrate API (`POST /api/v1/repos/migrate`) to import into `orchard9` org, then triggers provision
6. Wire Gitea API token (`THREE_SIX_GITEA`) and Woodpecker API token (`THREE_SIX_WOODPECKER`) into hub deployment as env vars from k8s secrets
7. Add ServiceAccount + RBAC for hub pod — read-only access to namespaces and deployments across the cluster (no cluster-admin needed)

### Feature 3: `fleet-management-ui`

React UI in hub mode for fleet management — the control plane surface.

**Tasks:**
1. Fleet dashboard page — replaces current hub page with three sections: running instances, available repos, import
2. Running instances section — cards showing project name, URL (clickable), pod status (healthy/starting/offline), active milestone, feature count, agent running badge. Data from `GET /api/hub/fleet` merged with heartbeat registry
3. Available repos section — cards for repos without instances, "Start" button triggers `POST /api/hub/provision`. Shows provisioning status via SSE updates
4. Import section — form with URL input, optional PAT input, "Import" button calls `POST /api/hub/import`. Shows progress (importing → provisioning → ready)
5. SSE integration — `FleetUpdated` event emitted when provisioning completes or heartbeat changes status. UI updates live without polling
6. Search/filter — client-side text filter across all sections (running + available)

### Feature 4: `fleet-auth-gate`

Ensure all sdlc project instances behind `*.sdlc.threesix.ai` are gated by the same Google OAuth — not just the hub.

**Tasks:**
1. Create shared Traefik middleware `sdlc-google-auth` that points to the hub's oauth2-proxy
2. Apply the middleware to the v18 fleet-reconcile Helm template so every new project ingress automatically gets auth
3. Retrofit the existing `sdlc-sdlc` ingress to use the shared middleware
4. Test: unauthenticated request to `sdlc-sdlc.threesix.ai` redirects to Google sign-in, then passes through after auth

## Dependencies

- v18 (fleet-automation) must pass UAT — the provisioning pipeline is the engine
- v38 (credential-pool-helm) should ship for agent runs in provisioned instances
- GCP OAuth client ID must be created manually (ops prerequisite)

## Wave plan

- **Wave 1:** `fleet-hub-deployment` + `fleet-auth-gate` (ops + auth — can deploy and test without fleet UI)
- **Wave 2:** `fleet-management-api` (server endpoints, needs hub deployed)
- **Wave 3:** `fleet-management-ui` (React UI, needs API endpoints)
