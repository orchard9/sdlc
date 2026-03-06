# Fleet Deployment Model

How sdlc instances run in the k3s cluster and how the hub coordinates them.

---

## Topology

```
                    Internet
                       │
                   ┌───┴───┐
                   │Traefik│  (k3s ingress controller)
                   └───┬───┘
                       │
          ┌────────────┼────────────┐
          │            │            │
   sdlc.threesix.ai   │    *.sdlc.threesix.ai
          │            │            │
    ┌─────┴─────┐      │    ┌──────┴──────┐
    │  sdlc-hub │      │    │ sdlc-<slug> │  (one per project)
    │ namespace │      │    │  namespace  │
    └───────────┘      │    └─────────────┘
                       │
              forwardAuth to hub
              /auth/verify
```

Every sdlc project gets its own namespace (`sdlc-<slug>`) with a helm-deployed instance.
The hub is a single dedicated deployment at `sdlc.threesix.ai` that provides:
- Google OAuth login/session management
- ForwardAuth verification for all project instances
- Fleet discovery (k8s API), repo listing (Gitea), provisioning (Woodpecker)

---

## Two Deployment Modes

### 1. Project instances (Helm chart)

**Path:** `k3s-fleet/deployments/helm/sdlc-server/`

Each project instance is deployed via `helm install` with project-specific values:

```bash
helm install sdlc-<slug> ./k3s-fleet/deployments/helm/sdlc-server/ \
  --namespace sdlc-<slug> --create-namespace \
  --set project.slug=<slug> \
  --set project.repo=orchard9/<repo> \
  --set auth.enabled=true
```

Instance pods run two containers:
- **sdlc-server** — serves the UI and API for one project
- **git-sync** — keeps `/workspace/<slug>` in sync with the Gitea repo (30s poll)

An init container does a one-time `git clone` so the server starts with data immediately.

**Ingress:** `<slug>.sdlc.threesix.ai` via Traefik IngressRoute. When `auth.enabled=true`,
requests pass through the `sdlc-google-auth` forwardAuth middleware before reaching the server.

### 2. Hub (dedicated manifests)

**Path:** `k3s-fleet/deployments/hub/`

The hub is a single-container deployment with no git-sync (it doesn't serve a project).
It runs in hub mode (`SDLC_HUB=true`) which activates:
- `/auth/login`, `/auth/callback`, `/auth/verify`, `/auth/logout` — native Google OAuth
- `/api/hub/fleet` — k8s namespace/deployment discovery
- `/api/hub/repos` — Gitea org repo listing
- `/api/hub/provision` — Woodpecker pipeline trigger for new instances
- `/api/hub/import` — Gitea repo mirror + auto-provision
- Hub UI — fleet dashboard with running/available views

The hub's IngressRoute has **no forwardAuth middleware** — it handles auth internally.
Project IngressRoutes point their forwardAuth to the hub's `/auth/verify` endpoint.

---

## Authentication Flow

Native Google OAuth built into sdlc-server (no oauth2-proxy).

### Browser sessions

```
Browser → Traefik → forwardAuth → Hub /auth/verify
                                    ├─ cookie valid? → 200 + X-Auth-User → Traefik proxies to instance
                                    └─ no cookie?    → 401 → browser redirects to /auth/login
                                                              → 302 to Google → callback → set cookie
```

Session cookie:
- Name: `sdlc_session`
- Domain: `.sdlc.threesix.ai` (covers hub + all instances = SSO)
- Format: `base64(json_payload).hmac_sha256_signature`
- Attributes: `HttpOnly; Secure; SameSite=Lax; Max-Age=86400`
- Payload: `{ email, name, exp }`

### Machine-to-machine

`Authorization: Bearer <token>` checked against `HUB_SERVICE_TOKENS` env var.
Existing `auth.rs` Bearer token logic in project instances is unchanged.

### Allowed domains

`OAUTH_ALLOWED_DOMAINS=livelyideo.tv,masq.me,virtualcommunities.ai`

The callback handler fetches the user's email from Google userinfo and rejects
any domain not in this list with a 403.

---

## Manifests

### Hub (`k3s-fleet/deployments/hub/`)

| File | Purpose |
|------|---------|
| `namespace.yaml` | `sdlc-hub` namespace |
| `rbac.yaml` | ServiceAccount + ClusterRole (read-only: namespaces, pods, deployments) |
| `sdlc-hub-deployment.yaml` | Hub server — OAuth env vars, fleet integration tokens |
| `sdlc-hub-service.yaml` | ClusterIP service (port 80 → 8080) |
| `ingressroute.yaml` | `sdlc.threesix.ai` → hub service (no forwardAuth) |
| `middleware-forward-auth.yaml` | Traefik forwardAuth middleware pointing to hub `/auth/verify` |

### Secrets (created manually, not in git)

**`sdlc-hub-oauth`** (namespace: `sdlc-hub`)
- `google-client-id` — GCP OAuth client ID
- `google-client-secret` — GCP OAuth client secret
- `session-secret` — 32+ char HMAC signing key

**`sdlc-hub-fleet-tokens`** (namespace: `sdlc-hub`)
- `gitea-api-token` — Gitea admin token for repo listing/import
- `woodpecker-api-token` — Woodpecker API token for provisioning
- `hub-service-tokens` — comma-separated M2M bearer tokens (optional)

**`sdlc-hub-notify`** (namespace: `sdlc-hub`)
- `api-key` — notify send key for `sdlc-hub` account (scope: `mail.sdlc.threesix.ai`)

```bash
# Recreate if lost (key is in notify DB, not recoverable from admin API):
kubectl create secret generic sdlc-hub-notify \
  --namespace sdlc-hub \
  --from-literal=api-key="<notify_send_...>"
```

### Helm chart (`k3s-fleet/deployments/helm/sdlc-server/`)

| File | Purpose |
|------|---------|
| `values.yaml` | Project slug, repo, auth toggle, image refs, resources |
| `templates/deployment.yaml` | Two-container pod (sdlc-server + git-sync) |
| `templates/service.yaml` | ClusterIP service |
| `templates/ingressroute.yaml` | `<slug>.sdlc.threesix.ai` with optional forwardAuth middleware |
| `templates/middleware-google-auth.yaml` | ForwardAuth → hub `/auth/verify` (created when `auth.enabled`) |
| `templates/external-secret-postgres.yaml` | Optional postgres credential from GCP Secret Manager |

---

## DNS & TLS

- **Wildcard cert:** `*.sdlc.threesix.ai` stored as `sdlc-wildcard-tls` secret
  - Lives in `sdlc-tls` namespace, copied into each project namespace
  - Managed via cert-manager or manual renewal
- **DNS:** Cloudflare — each `<slug>.sdlc.threesix.ai` is an A record pointing to the cluster ingress IPs (`208.122.204.172`, `.173`, `.174`)
- **Hub DNS:** `sdlc.threesix.ai` (same A record, no wildcard — explicit record needed)

---

## Provisioning a New Instance

1. Repo exists in Gitea `orchard9` org (or gets imported via `/api/hub/import`)
2. Woodpecker pipeline runs `helm install` with the project's values
3. Namespace `sdlc-<slug>` created, git-sync clones the repo
4. Cloudflare DNS A record created for `<slug>.sdlc.threesix.ai`
5. TLS secret copied from `sdlc-tls` namespace
6. Instance starts sending heartbeats to the hub

---

## Environment Variables

### Hub-only

| Variable | Purpose |
|----------|---------|
| `SDLC_HUB` | `true` — activates hub mode |
| `GOOGLE_CLIENT_ID` | GCP OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | GCP OAuth client secret |
| `SESSION_SECRET` | HMAC signing key for session cookies |
| `OAUTH_ALLOWED_DOMAINS` | Comma-separated allowed email domains |
| `INGRESS_DOMAIN` | `sdlc.threesix.ai` — used for fleet discovery |
| `GITEA_URL` | Internal Gitea URL for repo listing |
| `GITEA_API_TOKEN` | Gitea admin token |
| `WOODPECKER_URL` | Woodpecker server URL for provisioning |
| `WOODPECKER_API_TOKEN` | Woodpecker API token |
| `HUB_SERVICE_TOKENS` | M2M bearer tokens (optional) |
| `NOTIFY_URL` | `https://notify.orchard9.ai` — OTP email delivery |
| `NOTIFY_API_KEY` | notify send key (`sdlc-hub-notify` secret) |
| `NOTIFY_HOST` | `mail.sdlc.threesix.ai` |
| `NOTIFY_FROM` | `noreply@mail.sdlc.threesix.ai` (display: "Ponder") |

### Instance-only

| Variable | Purpose |
|----------|---------|
| `SDLC_ROOT` | Path to project workspace (set by git-sync) |
| `SDLC_HUB_URL` | Hub URL for heartbeat reporting |
| `DATABASE_URL` | Postgres connection string (optional, enables cluster storage) |
| `ANTHROPIC_API_KEY` | For agent runs |
