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

## Wildcard TLS Certificate

`*.sdlc.threesix.ai` is stored as the `sdlc-wildcard-tls` secret in the `sdlc-tls` namespace. The provision pipeline copies it into each new project namespace.

### Manual renewal

```bash
# Update the cert in the source namespace
kubectl create secret tls sdlc-wildcard-tls \
  --namespace sdlc-tls \
  --cert=fullchain.pem \
  --key=privkey.pem \
  --dry-run=client -o yaml | kubectl apply -f -

# Copy to all project namespaces
for ns in $(kubectl get ns -o name | grep sdlc- | grep -v sdlc-tls | grep -v sdlc-hub | grep -v sdlc-system); do
  ns_name=${ns#namespace/}
  kubectl get secret sdlc-wildcard-tls -n sdlc-tls -o yaml \
    | sed "s/namespace: sdlc-tls/namespace: $ns_name/" \
    | kubectl apply -f -
done
```

### Cert-manager (if configured)

If cert-manager manages the wildcard cert, renewal is automatic. The provision pipeline still copies the secret into new namespaces — cert-manager only manages the source copy.

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

### View project logs

```bash
# Server logs
kubectl logs -n sdlc-<slug> -l app.kubernetes.io/name=sdlc-server -c sdlc-server -f

# Git-sync sidecar logs
kubectl logs -n sdlc-<slug> -l app.kubernetes.io/name=sdlc-server -c git-sync -f
```

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
