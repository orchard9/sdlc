# Troubleshooting Fleet

Common failure modes and their resolutions for the sdlc fleet.

---

## Instance won't come online

**Symptom:** Hub shows instance as "Provisioning" or "Failed" indefinitely.

**Check the provision pipeline:**

```bash
# Find the Woodpecker build for the slug
curl -s -H "Authorization: Bearer $WOODPECKER_API_TOKEN" \
  "$WOODPECKER_URL/api/repos/orchard9/sdlc/pipelines?page=1&per_page=5" | jq
```

**Check the deployment:**

```bash
kubectl get pods -n sdlc-<slug>
kubectl describe deployment sdlc-server -n sdlc-<slug>
kubectl get events -n sdlc-<slug> --sort-by='.lastTimestamp'
```

**Common causes:**

| Cause | Fix |
|-------|-----|
| Image pull failure (`ErrImagePull`) | Check `ghcr-secret` exists in namespace, verify image tag |
| `gitea-credentials` secret missing | Create the secret (see [cluster-operations.md](cluster-operations.md#project-instance-secrets-sdlc-slug-namespace)) |
| Git-sync init container stuck | Check Gitea repo exists and is accessible from the cluster |
| CrashLoopBackOff on sdlc-server | Check logs: `kubectl logs -n sdlc-<slug> -c sdlc-server --previous` |
| TLS secret missing | Copy wildcard cert: see [cluster-operations.md](cluster-operations.md#wildcard-tls-certificate) |

---

## Heartbeat not reaching hub

**Symptom:** Instance pod is running but hub shows it as offline or missing from fleet view.

**Check from the instance pod:**

```bash
kubectl exec -n sdlc-<slug> deploy/sdlc-server -c sdlc-server -- \
  env | grep -E 'SDLC_HUB_URL|SDLC_BASE_URL'
```

Both must be set. `SDLC_HUB_URL` should be `https://sdlc.threesix.ai` (or the hub's internal cluster URL). `SDLC_BASE_URL` should be `https://<slug>.sdlc.threesix.ai`.

**Check connectivity:**

```bash
kubectl exec -n sdlc-<slug> deploy/sdlc-server -c sdlc-server -- \
  curl -sf -o /dev/null -w '%{http_code}' "$SDLC_HUB_URL/api/health"
```

If this fails, check network policies or DNS resolution within the cluster.

**Check server logs for heartbeat warnings:**

```bash
kubectl logs -n sdlc-<slug> -l app.kubernetes.io/name=sdlc-server -c sdlc-server \
  | grep -i heartbeat
```

---

## Agent runs failing

**Symptom:** Agent runs start but fail immediately or produce no output.

### No credentials available

```bash
# Check pool status
curl -s https://sdlc.threesix.ai/api/credential-pool | jq
```

If `active: 0`, add credentials (see [cluster-operations.md](cluster-operations.md#managing-credentials)). If the pool is disabled, check that `DATABASE_URL` is set in the instance's environment.

### Credential exhaustion

All credentials locked by concurrent runs. The `SKIP LOCKED` query returns `None` and the agent falls back to ambient auth. If ambient auth is also absent, the run fails.

**Resolution:** Add more credentials to the pool, or reduce concurrent agent runs.

### Agent binary missing

The `sdlc-base` image must include the Claude Code CLI. Check the image:

```bash
kubectl exec -n sdlc-<slug> deploy/sdlc-server -c sdlc-server -- which claude
```

If missing, rebuild and push `sdlc-base:latest` and restart pods.

---

## Git-sync sidecar issues

**Symptom:** Workspace is stale or empty.

**Check sidecar logs:**

```bash
kubectl logs -n sdlc-<slug> -l app.kubernetes.io/name=sdlc-server -c git-sync -f
```

**Common causes:**

| Cause | Fix |
|-------|-----|
| Auth failure (401/403) | Recreate `gitea-credentials` secret with valid token |
| Repo not found (404) | Verify repo exists in Gitea `orchard9` org |
| Symlink broken | Delete pod to trigger fresh init clone |
| Merge conflict in `.sdlc/` | Resolve in Gitea, then delete pod |

---

## Hub pod issues

**Symptom:** Hub UI is unreachable or fleet view is empty.

**Check hub health:**

```bash
kubectl get pods -n sdlc-hub
kubectl logs -n sdlc-hub deploy/sdlc-hub -f
curl -sf https://sdlc.threesix.ai/api/health
```

**Fleet view empty but instances are running:**

The hub queries the k8s API for `sdlc-*` namespaces. Check RBAC:

```bash
kubectl auth can-i list namespaces --as=system:serviceaccount:sdlc-hub:sdlc-hub
kubectl auth can-i list deployments --as=system:serviceaccount:sdlc-hub:sdlc-hub --all-namespaces
```

Both should return `yes`. If not, reapply `k3s-fleet/deployments/hub/rbac.yaml`.

**OAuth login fails:**

1. Check `sdlc-hub-oauth` secret exists and has correct values
2. Verify the GCP OAuth client's authorized redirect URI includes `https://sdlc.threesix.ai/auth/callback`
3. Check `OAUTH_ALLOWED_DOMAINS` includes the user's email domain

---

## Provision stuck in "Requested"

**Symptom:** Clicking "Start" in the hub shows "Requested" but never progresses.

The hub triggers a Woodpecker pipeline with the `PROVISION_SLUG` variable. Check:

1. **Woodpecker reachable:** `kubectl exec -n sdlc-hub deploy/sdlc-hub -- curl -sf $WOODPECKER_URL/api/health`
2. **Woodpecker token valid:** Tokens are invalidated on Woodpecker server restart. Re-generate if needed.
3. **Repo activated in Woodpecker:** The `orchard9/sdlc` repo must be activated in the Woodpecker UI for pipelines to run.
4. **Pipeline exists:** `.woodpecker/provision.yaml` must be present in the repo.

---

## DNS record not created

**Symptom:** Instance deploys but `<slug>.sdlc.threesix.ai` doesn't resolve.

The provision pipeline creates Cloudflare DNS records. Check:

```bash
# Verify Cloudflare credentials are available to the pipeline
# These should be in the Woodpecker secrets for the repo:
# - CLOUDFLARE_API_TOKEN
# - CLOUDFLARE_ZONE_ID
```

**Manual DNS creation:**

```bash
curl -X POST "https://api.cloudflare.com/client/v4/zones/$CLOUDFLARE_ZONE_ID/dns_records" \
  -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "A",
    "name": "<slug>.sdlc.threesix.ai",
    "content": "208.122.204.172",
    "proxied": false,
    "ttl": 300
  }'
```

Repeat for each ingress IP (`.173`, `.174`).
