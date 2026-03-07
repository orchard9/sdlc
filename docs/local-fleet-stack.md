# Local Fleet Stack

Run the full hub + fleet pipeline from your laptop. Provisions real pods on the k3s cluster.

---

## Architecture

```
localhost                                          k3s cluster
─────────────────────────────────────              ──────────────────────────
:7781  Postgres (sdlc + gitea DBs)
:7782  Gitea    (orchard9 org, repos)
:7783  Woodpecker server ──── agent ──────────►    kubectl / helm install
:7778  Hub server (ponder --hub)                      │
:7779  Vite dev server (optional)                     ▼
                                                   sdlc-<slug> namespace
                                                     ├─ sdlc-server pod
                                                     ├─ git-sync sidecar
                                                     └─ <slug>.sdlc.threesix.ai
```

Hub and Woodpecker run locally in Docker. The hub server runs natively via `cargo watch` for hot reload. Woodpecker pipelines execute against the real k3s cluster using your kubeconfig.

## Port Allocation

| Port | Service |
|------|---------|
| 7777 | sdlc UI (project instance, local dev) |
| 7778 | Hub server (`ponder --hub`) |
| 7779 | Vite dev server |
| 7781 | Postgres |
| 7782 | Gitea |
| 7783 | Woodpecker server |

---

## Setup

### 1. Start infrastructure

```bash
docker compose up -d
```

Starts Postgres (7781) and Gitea (7782). First run pulls images.

### 2. Bootstrap Gitea

```bash
./dev/setup-gitea.sh
```

Creates admin user (`sdlc-admin`/`sdlc-admin`), `orchard9` org, and generates an API token. Idempotent.

### 3. Create Gitea OAuth app (for Woodpecker)

```bash
curl -sf -X POST "http://localhost:7782/api/v1/user/applications/oauth2" \
  -u "sdlc-admin:sdlc-admin" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "woodpecker",
    "redirect_uris": ["http://localhost:7783/authorize"]
  }'
```

Save the `client_id` and `client_secret` from the response.

### 4. Enable Woodpecker

Edit `docker-compose.yml` — uncomment the `woodpecker-server` and `woodpecker-agent` services and fill in the OAuth credentials from step 3. Or use a `docker-compose.override.yml`:

```yaml
services:
  woodpecker-server:
    image: woodpeckerci/woodpecker-server:latest
    ports: ["7783:8000"]
    environment:
      WOODPECKER_HOST: http://localhost:7783
      WOODPECKER_GITEA: "true"
      WOODPECKER_GITEA_URL: http://gitea:3000
      WOODPECKER_GITEA_CLIENT: "<client_id>"
      WOODPECKER_GITEA_SECRET: "<client_secret>"
      WOODPECKER_ADMIN: sdlc-admin
      WOODPECKER_SECRET: "local-dev-secret"
    depends_on: [gitea]
    volumes:
      - woodpecker-data:/var/lib/woodpecker

  woodpecker-agent:
    image: woodpeckerci/woodpecker-agent:latest
    environment:
      WOODPECKER_SERVER: woodpecker-server:9000
      WOODPECKER_SECRET: "local-dev-secret"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ${HOME}/.kube/orchard9-k3sf.yaml:/root/.kube/config:ro
    depends_on: [woodpecker-server]

volumes:
  woodpecker-data:
```

Then restart:

```bash
docker compose up -d
```

### 5. Activate the sdlc repo in Woodpecker

Open `http://localhost:7783`, log in with `sdlc-admin`, and activate the `orchard9/sdlc` repo. This registers the webhook so Woodpecker runs `provision.yaml` when triggered.

### 6. Start the hub server

```bash
mkdir -p /tmp/sdlc-hub/.sdlc

SDLC_ROOT=/tmp/sdlc-hub \
SDLC_HUB=true \
GITEA_URL=http://localhost:7782 \
GITEA_API_TOKEN=<from setup-gitea.sh> \
DATABASE_URL=postgres://sdlc:sdlc@localhost:7781/sdlc \
WOODPECKER_URL=http://localhost:7783 \
WOODPECKER_API_TOKEN=<from woodpecker UI> \
cargo watch -x 'run --bin ponder -- ui start --port 7778 --no-open --no-tunnel --hub'
```

### 7. Start Vite (optional, for frontend hot reload)

```bash
cd frontend && npm run dev   # serves on :7779, proxies /api to :7778
```

---

## User Flow

### Add a project

Three ways to get a repo into the system — all from the hub UI at `http://localhost:7778` (or `:7779` via Vite):

1. **Create** — enter a repo name → get an authenticated push URL → add as git remote and push:
   ```bash
   cd /path/to/your-project
   git remote add gitea <push_url_from_hub>
   git push gitea main
   ```

2. **Import** — paste an external clone URL (GitHub, GitLab) → hub mirrors it into Gitea. One-time copy, not a live sync. Gitea becomes the source of truth.

3. **Already in Gitea** — if the repo exists in the `orchard9` org, it shows up in the available list automatically.

### Provision a pod

Click **Start** on an available repo. The hub triggers Woodpecker, which:

1. Clones the repo from Gitea
2. Builds a custom image if `Dockerfile.sdlc` exists, otherwise uses `sdlc-base:latest`
3. Runs `helm install` against the k3s cluster → creates `sdlc-<slug>` namespace
4. Creates Cloudflare DNS record for `<slug>.sdlc.threesix.ai`
5. Copies wildcard TLS cert into the namespace

The pod is live at `https://<slug>.sdlc.threesix.ai` within ~2 minutes.

### Test a project locally (without provisioning)

Run a second ponder instance pointed at the project:

```bash
SDLC_ROOT=/path/to/your-project \
SDLC_HUB_URL=http://localhost:7778 \
cargo run --bin ponder -- ui start --port 7777 --no-open --no-tunnel
```

The project sends heartbeats to the hub and appears in the fleet view.

---

## Tear Down

```bash
docker compose down -v   # -v removes volumes (pgdata, gitea-data, woodpecker-data)
```

Provisioned pods in the k3s cluster are not affected — tear those down separately:

```bash
helm uninstall sdlc-<slug> -n sdlc-<slug>
kubectl delete namespace sdlc-<slug>
```
