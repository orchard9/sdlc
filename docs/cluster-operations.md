# Cluster Operations

Day-to-day operations for the sdlc fleet running on k3s. Covers secrets, credentials, cert management, and hub state.

---

## Secrets

All secrets are created manually via `kubectl` — they are not checked into git.

### Hub secrets (`sdlc-hub` namespace)

**`sdlc-hub-oauth`** — Google OAuth for SSO:

```bash
kubectl create secret generic sdlc-hub-oauth \
  --namespace sdlc-hub \
  --from-literal=google-client-id="<GCP_CLIENT_ID>" \
  --from-literal=google-client-secret="<GCP_CLIENT_SECRET>" \
  --from-literal=session-secret="$(openssl rand -hex 32)"
```

**`sdlc-hub-fleet-tokens`** — Gitea + Woodpecker integration:

```bash
kubectl create secret generic sdlc-hub-fleet-tokens \
  --namespace sdlc-hub \
  --from-literal=gitea-api-token="<GITEA_ADMIN_TOKEN>" \
  --from-literal=woodpecker-api-token="<WOODPECKER_API_TOKEN>" \
  --from-literal=hub-service-tokens="<COMMA_SEPARATED_M2M_TOKENS>"
```

The `hub-service-tokens` field is optional. When set, external services can call hub API endpoints with `Authorization: Bearer <token>`.

**`sdlc-hub-notify`** — OTP email delivery:

```bash
kubectl create secret generic sdlc-hub-notify \
  --namespace sdlc-hub \
  --from-literal=api-key="<NOTIFY_SEND_KEY>"
```

### Project instance secrets (`sdlc-<slug>` namespace)

**`gitea-credentials`** — git-sync sidecar authentication (required):

```bash
kubectl create secret generic gitea-credentials \
  --namespace sdlc-<slug> \
  --from-literal=username="claude-agent" \
  --from-literal=token="<GITEA_TOKEN>"
```

**`anthropic-credentials`** — fallback agent key (optional, credential pool preferred):

```bash
kubectl create secret generic anthropic-credentials \
  --namespace sdlc-<slug> \
  --from-literal=api_key="<ANTHROPIC_API_KEY>"
```

**`postgres-sdlc-credentials`** — cluster storage (optional, via ExternalSecret):

Created automatically when `postgres.externalSecret.gsmKey` is set in Helm values. The ExternalSecret operator fetches the connection string from GCP Secret Manager.

**`sdlc-hub-token`** — shared fleet token for authenticated heartbeats (optional but recommended):

```bash
kubectl create secret generic sdlc-hub-token \
  --namespace sdlc-hub \
  --from-literal=token="$(openssl rand -hex 32)"
```

This token is copied by the provision pipeline into every project namespace as `sdlc-hub-token`. Project pods inject it as `SDLC_HUB_TOKEN`, which is sent as a Bearer token on heartbeat calls. On the hub side, add the same value to `HUB_SERVICE_TOKENS` in the `sdlc-hub-fleet-tokens` secret (comma-separated list). When `HUB_SERVICE_TOKENS` is empty, the hub accepts all heartbeats without auth.

### Rotating a secret

```bash
# Delete and recreate
kubectl delete secret <name> --namespace <ns>
kubectl create secret generic <name> --namespace <ns> --from-literal=...

# Restart the deployment to pick up the new value
kubectl rollout restart deployment/<deployment-name> --namespace <ns>
```

---

## Credential Pool

The credential pool stores Claude OAuth tokens in PostgreSQL so multiple project pods can run agents concurrently without overloading a single account. It is optional — when `DATABASE_URL` is not set, agents fall back to ambient credentials (`~/.claude/`).

### Schema

Created automatically on first connection (idempotent):

```sql
CREATE TABLE IF NOT EXISTS claude_credentials (
    id           BIGSERIAL   PRIMARY KEY,
    account_name TEXT        NOT NULL,
    token        TEXT        NOT NULL,
    is_active    BOOLEAN     NOT NULL DEFAULT true,
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01',
    use_count    BIGINT      NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS claude_credentials_lru_idx
ON claude_credentials (last_used_at ASC)
WHERE is_active;
```

### Checkout mechanism

When `spawn_agent_run` starts an agent:

1. `SELECT ... FOR UPDATE SKIP LOCKED` picks the least-recently-used active credential
2. `last_used_at` and `use_count` are updated atomically in the same transaction
3. The token is injected as `CLAUDE_CODE_OAUTH_TOKEN` (or the provider's equivalent env var) into the subprocess
4. If no active credentials exist or all are locked, the agent runs with ambient auth

This is round-robin LRU — credentials distribute evenly across concurrent runs.

### Managing credentials

All write endpoints require `Authorization: Bearer {SDLC_AGENT_TOKEN}`.

```bash
# Check pool status
curl -s https://sdlc.threesix.ai/api/credential-pool | jq

# List credentials (tokens are never returned)
curl -s https://sdlc.threesix.ai/api/credential-pool/credentials | jq

# Add a credential
curl -s -X POST https://sdlc.threesix.ai/api/credential-pool/credentials \
  -H "Authorization: Bearer $SDLC_AGENT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"account_name": "account@example.com", "token": "cot_..."}'

# Disable a credential (stops checkout, preserves history)
curl -s -X PATCH https://sdlc.threesix.ai/api/credential-pool/credentials/3 \
  -H "Authorization: Bearer $SDLC_AGENT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"is_active": false}'

# Delete a credential
curl -s -X DELETE https://sdlc.threesix.ai/api/credential-pool/credentials/3 \
  -H "Authorization: Bearer $SDLC_AGENT_TOKEN"
```

### Graceful degradation

| Condition | Behavior |
|-----------|----------|
| `DATABASE_URL` not set | Pool disabled, agents use ambient auth |
| Database unreachable at startup | Pool disabled, warning logged |
| Schema init fails | Pool disabled, warning logged |
| No active credentials in table | `checkout()` returns `None`, agents use ambient auth |
| All credentials locked (concurrent runs) | `SKIP LOCKED` returns `None`, agents use ambient auth |

---

## Heartbeat Protocol

Project instances report to the hub every 30 seconds via `POST {SDLC_HUB_URL}/api/hub/heartbeat`.

### Payload

```json
{
  "name": "my-project",
  "url": "https://my-project.sdlc.threesix.ai",
  "active_milestone": "v2-auth",
  "feature_count": 5,
  "agent_running": true
}
```

| Field | Type | Source |
|-------|------|--------|
| `name` | `string` | Basename of `SDLC_ROOT` directory |
| `url` | `string` | `SDLC_BASE_URL` env var, or `http://localhost:{port}` |
| `active_milestone` | `string?` | `active_milestone` field from `.sdlc/state.yaml` |
| `feature_count` | `u32?` | Count of subdirectories in `.sdlc/features/` |
| `agent_running` | `bool?` | `true` if any entries exist in the agent runs map |

### Status transitions

The hub classifies instances by time since last heartbeat:

| Status | Age | Behavior |
|--------|-----|----------|
| `online` | < 30s | Healthy, shown in fleet view |
| `stale` | 30–90s | Shown with warning indicator |
| `offline` | 90s–5min | Shown as offline |
| *(removed)* | > 5min | Entry swept from registry |

The sweep task runs every 15 seconds, recomputes statuses, and emits `ProjectRemoved` SSE events for entries that age out.

### Activation

- **Instance side:** Set `SDLC_HUB_URL` — the heartbeat task spawns automatically in `new_with_port()`. If unset, no task is spawned.
- **Hub side:** Always active when `SDLC_HUB=true`. Heartbeats are accepted at `POST /api/hub/heartbeat`.
- **Timeout:** Each heartbeat POST has a 5-second timeout. Failures are logged as warnings and retried on the next 30-second tick.

---

## Hub State Persistence

The hub registry persists to `~/.sdlc/hub-state.yaml` (or `{SDLC_ROOT}/.sdlc/hub-state.yaml`). This file contains:

- **projects** — last-seen project entries (keyed by URL)
- **provisions** — provision lifecycle records (keyed by slug)
- **activity** — recent activity log entries (max 100)

The file is written after every state mutation (heartbeat, provision status change, sweep). It survives hub restarts.

### Inspecting hub state

```bash
# On the hub pod
kubectl exec -n sdlc-hub deploy/sdlc-hub -- cat /tmp/sdlc-hub/.sdlc/hub-state.yaml

# Via API
curl -s https://sdlc.threesix.ai/api/hub/projects | jq    # project registry
curl -s https://sdlc.threesix.ai/api/hub/fleet | jq       # fleet instances (k8s + registry merge)
curl -s https://sdlc.threesix.ai/api/hub/summary | jq     # aggregate health
curl -s https://sdlc.threesix.ai/api/hub/attention | jq   # instances needing intervention
curl -s https://sdlc.threesix.ai/api/hub/activity | jq    # recent 30 activity entries
```

---

## Fleet Reconciliation

The hub runs a fleet poll task every 15 seconds that:

1. Queries the k8s API for all `sdlc-*` namespaces (excluding `sdlc-tls`, `sdlc-hub`, `sdlc-system`)
2. Checks each namespace for a deployment with label `app.kubernetes.io/name=sdlc-server`
3. Extracts deployment status: `Running`, `Pending`, `Failed`, or `Unknown`
4. Merges heartbeat data from the hub registry (active milestone, agent status, last seen)
5. Reconciles provision records — a provisioning instance transitions to `Ready` when its deployment is healthy

If the k8s API is unavailable, the hub falls back to the provision registry to show provisioning-in-progress instances.

---

## Automated Hub Rollout

Every push to `main` builds and pushes `ghcr.io/orchard9/sdlc:latest`, then the CI workflow automatically restarts the hub pod to pick up the new image.

This requires a `KUBE_CONFIG` secret in the GitHub repo (base64-encoded kubeconfig). If absent, the build step logs a warning and skips the rollout.

### One-time setup

```bash
# Generate a kubeconfig scoped to the sdlc-hub namespace
# (use the existing kubeconfig — or create a restricted ServiceAccount for CI)
cat ~/.kube/orchard9-k3sf.yaml | base64 | tr -d '\n' > /tmp/kubeconfig-b64.txt

# Add to GitHub repo secrets
gh secret set KUBE_CONFIG --repo orchard9/sdlc < /tmp/kubeconfig-b64.txt
rm /tmp/kubeconfig-b64.txt
```

After adding the secret, every `main` push will: build image → push → restart hub → wait for rollout.

To roll back to a specific SHA tag manually:
```bash
kubectl set image deployment/sdlc-hub sdlc-server=ghcr.io/orchard9/sdlc:sha-<7chars> -n sdlc-hub
kubectl rollout status deployment/sdlc-hub -n sdlc-hub
```

---

## Alerting

SDLC-specific Prometheus alert rules are defined in `k3s-fleet/deployments/hub/alerting-rules-sdlc.yaml`. They are merged into the cluster-wide `prometheus-alerting-rules` configmap in the `observability` namespace. Alerts fire to Discord via Alertmanager.

### Active alert groups

| Group | Alerts |
|-------|--------|
| `sdlc-hub-alerts` | `SdlcHubDown` (critical, 2m), `SdlcHubCrashLooping` (critical, 5m) |
| `sdlc-project-alerts` | `SdlcProjectCrashLooping` (warning, 5m), `SdlcWorkspacePVCFull` (warning, 10m), `SdlcProvisionStuck` (warning, 20m) |

### Updating alert rules

Edit `k3s-fleet/deployments/hub/alerting-rules-sdlc.yaml`, then apply:

```bash
# Fetch current rules, merge in updated sdlc groups, apply
kubectl get configmap prometheus-alerting-rules -n observability \
  -o jsonpath='{.data.alerting-rules\.yaml}' > /tmp/current-rules.yaml

# Append the rules: block from alerting-rules-sdlc.yaml, then apply
# (or edit /tmp/current-rules.yaml directly and apply below)

kubectl create configmap prometheus-alerting-rules \
  --namespace observability \
  --from-file=alerting-rules.yaml=/tmp/current-rules.yaml \
  --dry-run=client -o yaml | kubectl apply -f -

# Reload Prometheus
kubectl exec -n observability deploy/prometheus -- wget -qO- --post-data='' http://localhost:9090/-/reload
```

---

## Wildcard TLS Certificate

`*.sdlc.threesix.ai` is managed by cert-manager using the `letsencrypt-prod` ClusterIssuer. The `Certificate` resource lives in the `sdlc-tls` namespace and auto-renews 30 days before expiry (renew-before: 720h).

The secret is automatically mirrored to all `sdlc-*` namespaces via the Reflector operator (annotations on the Certificate resource). The provision pipeline also copies it as a belt-and-suspenders fallback.

### Checking cert status

```bash
# Certificate health + renewal time
kubectl describe certificate sdlc-wildcard-tls -n sdlc-tls

# Verify the secret is present in a project namespace
kubectl get secret sdlc-wildcard-tls -n sdlc-<slug>
```

### Manual emergency renewal

Only needed if cert-manager is broken. Otherwise renewal is automatic.

```bash
# Force immediate renewal
kubectl annotate certificate sdlc-wildcard-tls -n sdlc-tls \
  cert-manager.io/issuer-kind=ClusterIssuer \
  cert-manager.io/force-renewal="$(date +%s)"

# If cert-manager is unavailable, renew manually and apply
kubectl create secret tls sdlc-wildcard-tls \
  --namespace sdlc-tls \
  --cert=fullchain.pem \
  --key=privkey.pem \
  --dry-run=client -o yaml | kubectl apply -f -
```

---

## Workspace Storage

Each project pod has a `PersistentVolumeClaim` (`workspace-<slug>`) that persists across pod restarts. This preserves agent work-in-progress, `.sdlc/` state, and embedded databases (redb) even if the pod is killed.

**Default:** 5Gi Longhorn (replicated, survives node failure). Configurable via `storage.size` and `storage.storageClass` in Helm values.

### Inspecting a project's PVC

```bash
kubectl get pvc workspace-<slug> -n sdlc-<slug>
kubectl describe pvc workspace-<slug> -n sdlc-<slug>
```

### Resizing a PVC

Longhorn supports online volume expansion:

```bash
kubectl patch pvc workspace-<slug> -n sdlc-<slug> \
  --type merge -p '{"spec":{"resources":{"requests":{"storage":"20Gi"}}}}'
```

### Deleting a project (including storage)

The Helm chart's PVC has `helm.sh/resource-policy: keep` by default — `helm uninstall` does **not** delete the PVC, preserving workspace data. Delete manually when you're sure the data is no longer needed:

```bash
helm uninstall sdlc-<slug> -n sdlc-<slug>
kubectl delete pvc workspace-<slug> -n sdlc-<slug>
kubectl delete namespace sdlc-<slug>
```

---

## Image Registry

Project images are stored at `registry.threesix.ai` (Zot, no auth). The `ghcr-secret` imagePullSecret in each namespace provides access to GitHub Container Registry for any private base images.

### Creating the pull secret

```bash
kubectl create secret docker-registry ghcr-secret \
  --namespace sdlc-<slug> \
  --docker-server=ghcr.io \
  --docker-username=<GITHUB_USER> \
  --docker-password=<GITHUB_PAT>
```

The provision pipeline handles this automatically for new namespaces.

---

## Common Operations

### Restart a project instance

```bash
kubectl rollout restart deployment/sdlc-server -n sdlc-<slug>
kubectl rollout status deployment/sdlc-server -n sdlc-<slug> --timeout=120s
```

### View logs

**Step 1 — Citadel (structured, searchable, persistent)**

The cluster-wide citadel-agent DaemonSet tails `/var/log/containers/*.log` on every node, which includes all `sdlc-hub` and `sdlc-*` namespace pods. Logs are available in the Citadel dashboard at `https://citadel.orchard9.ai`.

No extra setup is required for sdlc — the wildcard glob covers all namespaces automatically.

**Step 2 — kubectl direct (fallback when Citadel is unavailable)**

```bash
# --- Hub ---
# Live tail hub logs
kubectl logs deployment/sdlc-hub -n sdlc-hub -f

# Last N lines (no streaming)
kubectl logs deployment/sdlc-hub -n sdlc-hub --tail=100

# Logs from the previous container (after a crash/restart)
kubectl logs deployment/sdlc-hub -n sdlc-hub --previous

# --- Project pod (2 containers: sdlc-server + git-sync) ---
# Server logs
kubectl logs deployment/sdlc-server-<slug> -n sdlc-<slug> -c sdlc-server -f

# Git-sync sidecar logs
kubectl logs deployment/sdlc-server-<slug> -n sdlc-<slug> -c git-sync --tail=50

# Previous container logs (after crash)
kubectl logs deployment/sdlc-server-<slug> -n sdlc-<slug> -c sdlc-server --previous

# --- Cross-cutting ---
# All sdlc namespaces at a glance
kubectl get ns | grep sdlc

# Recent events (OOM kills, restarts, scheduling failures) — k8s only keeps ~1hr
kubectl get events -n sdlc-hub --sort-by='.lastTimestamp'
kubectl get events -n sdlc-<slug> --sort-by='.lastTimestamp'

# Pod status + restart count (restart count > 0 = previous crashes)
kubectl get pods -n sdlc-hub -o wide
kubectl get pods -n sdlc-<slug> -o wide
```

All commands assume `KUBECONFIG=~/.kube/orchard9-k3sf.yaml` is set (or exported).

> **Note:** `kubectl logs --previous` only works if the container has restarted at least once within the current pod. If the pod was deleted and recreated (e.g., rollout restart), previous logs are gone — use Citadel.

### Remove a project instance

```bash
helm uninstall sdlc-<slug> -n sdlc-<slug>
kubectl delete namespace sdlc-<slug>
```

The provision record persists in hub-state.yaml as historical data.

### Force-sync a project repo

```bash
# Kill the git-sync sidecar to trigger a fresh clone
kubectl delete pod -n sdlc-<slug> -l app.kubernetes.io/name=sdlc-server
```

The pod restarts with the init container doing a fresh `git clone` before the server starts.

---

## Citadel Telemetry

Ponder can ship structured tracing events to Citadel for centralized log aggregation. This is optional — when the env vars are absent, Ponder uses standard stderr logging with zero overhead.

### How it works

A custom `tracing::Layer` captures all INFO+ events, serializes them as JSON, and ships them to Citadel's batch ingest endpoint (`POST /api/v1/ingest`) every 5 seconds. Events are buffered in a bounded channel (4096 capacity) and dropped if the channel fills. Failed batches are logged as warnings and discarded (no retry queue).

Each event includes:
- `message`, `level`, `timestamp` — standard tracing fields
- `service` — `ponder-hub` or `ponder-<slug>` (derived from `SDLC_BASE_URL`)
- `environment` — `local` or `production`
- `target` — Rust module path (e.g. `sdlc_server::heartbeat`)
- `fields` — all structured key-value fields from the tracing event

### Env vars

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `PONDER_CITADEL_URL` | Yes | — | Citadel ingest URL (e.g. `https://citadel-staging.orchard9.ai`) |
| `PONDER_CITADEL_API_KEY` | Yes | — | Citadel API key (`ck_live_*`) |
| `PONDER_CITADEL_TENANT_ID` | Yes | — | Citadel tenant UUID |
| `PONDER_ENVIRONMENT` | No | `production` if `SDLC_HUB=true`, else `local` | Environment tag |

All three required vars must be set for the layer to activate.

### Local development

Set the env vars when starting the server:

```bash
PONDER_CITADEL_URL=https://citadel-staging.orchard9.ai \
PONDER_CITADEL_API_KEY=ck_live_... \
PONDER_CITADEL_TENANT_ID=b1692b26-... \
PONDER_ENVIRONMENT=local \
cargo run --bin ponder -- ui start --port 7777 --no-open --no-tunnel
```

Verify logs are flowing:

```bash
# Wait 10 seconds for the first batch, then query
curl -s "https://citadel-staging.orchard9.ai/api/v1/ingest/event" \
  -H "Authorization: Bearer $PONDER_CITADEL_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"message": "test from ponder", "level": "info", "attributes": {"service": "ponder-test"}}'
```

### Production (k3s cluster)

**Hub** — create the secret:

```bash
kubectl create secret generic sdlc-hub-citadel \
  --namespace sdlc-hub \
  --from-literal=url="https://citadel-staging.orchard9.ai" \
  --from-literal=api-key="ck_live_..." \
  --from-literal=tenant-id="b1692b26-..."
```

The hub deployment manifest (`sdlc-hub-deployment.yaml`) reads from this secret with `optional: true` — if the secret doesn't exist, the hub runs without Citadel.

**Project instances** — enable in Helm values:

```yaml
citadel:
  enabled: true
  url: https://citadel-staging.orchard9.ai
  tenantId: b1692b26-...
```

Then create the API key secret in the project namespace:

```bash
kubectl create secret generic sdlc-citadel-credentials \
  --namespace sdlc-<slug> \
  --from-literal=api-key="ck_live_..."
```

### Graceful degradation

| Condition | Behavior |
|-----------|----------|
| Any `PONDER_CITADEL_*` var missing | Layer not created, zero overhead |
| Citadel unreachable | Batch dropped, warning logged to stderr |
| Channel full (burst > 4096 events) | Oldest events dropped silently |
| Citadel returns non-2xx | Batch dropped, warning logged to stderr |
