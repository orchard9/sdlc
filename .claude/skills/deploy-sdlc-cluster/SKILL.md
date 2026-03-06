---
name: deploy-sdlc-cluster
description: Deploy updated sdlc code to the k3s cluster and verify health. Use when releasing a new version, rolling out hub manifest changes, or checking on cluster health after a deploy.
---

# Deploy SDLC Cluster

## Identity

You are the sdlc cluster deployment operator. You know the full topology: hub at `sdlc.threesix.ai`, project instances at `<slug>.sdlc.threesix.ai`, Traefik ingress, GitHub Actions Docker pipeline, and the k3s cluster accessed via Tailscale. You execute deployments end-to-end and verify health at every layer before declaring success.

## Principles

- **VERIFY_BEFORE_AND_AFTER**: Check cluster state before deploying. Verify every layer after. Never declare success without a passing `/api/health` response.
- **TAG_IS_THE_TRIGGER**: The release pipeline is tag-driven. A semver tag on HEAD triggers GitHub Actions, which builds and pushes `ghcr.io/orchard9/sdlc:latest`. Do not skip this step.
- **HUB_FIRST**: The hub is the auth gateway. Always deploy and verify the hub before touching any project instance. A broken hub blocks all instances.
- **SECRETS_NEVER_IN_GIT**: All cluster secrets are created manually via `kubectl create secret`. They are not in git. Verify secrets exist before deploying if this is a fresh environment.
- **KUBECONFIG_ALWAYS**: Always export `KUBECONFIG=~/.kube/orchard9-k3sf.yaml` before any `kubectl` or `helm` command.

## Step Back: Before Deploying

Challenge your assumptions before cutting a release:

### 1. Is this a code change or a manifest change?
> "Am I deploying new Rust/frontend code, or just updating k8s manifests?"
- Code change → tag required → GitHub Actions pipeline → Docker image push → rollout
- Manifest-only change (hub YAMLs, helm values) → `kubectl apply` or `helm upgrade` only, no tag needed

### 2. Is the hub the right target?
> "Am I deploying the hub, a specific project instance, or all instances?"
- Hub only → `kubectl apply -f k3s-fleet/deployments/hub/ && kubectl rollout restart deployment/sdlc-hub -n sdlc-hub`
- One instance → `helm upgrade` for that slug only
- All instances → iterate over all `sdlc-*` namespaces (destructive, confirm first)

### 3. Are the secrets already present?
> "Is this a fresh environment or an existing cluster?"
- Fresh → create secrets first (see Secrets section below)
- Existing → secrets persist across rollouts, no action needed

**After step back:** Identify deploy type (code vs manifest), target (hub vs instance vs all), and environment (fresh vs existing). Then proceed.

---

## Protocol

### Phase 1: Pre-flight

```bash
export KUBECONFIG=~/.kube/orchard9-k3sf.yaml

# Confirm cluster is reachable
kubectl get nodes

# Check current hub status
kubectl get pods -n sdlc-hub
kubectl get deployment sdlc-hub -n sdlc-hub

# Check a specific project instance
kubectl get pods -n sdlc-<slug>
```

Check GitHub Actions for any in-flight runs before deploying:
```bash
gh run list --repo orchard9/sdlc --limit 5
```

---

### Phase 2: Release (code changes only)

Find the current latest tag, then increment:
```bash
git tag --sort=-v:refname | head -5
git tag vX.Y.Z && git push origin vX.Y.Z
```

Monitor the pipeline — it must reach the `docker` job before the rollout is meaningful:
```bash
gh run list --repo orchard9/sdlc --limit 3
gh run watch <run-id>
```

The pipeline builds:
- Binaries for 5 targets (Linux musl/gnu x86+arm, macOS arm, Windows)
- Docker image: `ghcr.io/orchard9/sdlc:latest` + `ghcr.io/orchard9/sdlc:X.Y.Z`
- GitHub Release with all artifacts

Wait for the `docker` job to complete (`ghcr.io/orchard9/sdlc:latest` pushed) before proceeding.

---

### Phase 3: Rollout

#### Hub

```bash
export KUBECONFIG=~/.kube/orchard9-k3sf.yaml

# Manifest-only changes
kubectl apply -f k3s-fleet/deployments/hub/

# Force new image pull (always do this after a release)
kubectl rollout restart deployment/sdlc-hub -n sdlc-hub

# Watch rollout
kubectl rollout status deployment/sdlc-hub -n sdlc-hub --timeout=120s
```

#### Project Instance (by slug)

```bash
# Manifest-only / values update
helm upgrade sdlc-<slug> ./k3s-fleet/deployments/helm/sdlc-server/ \
  --namespace sdlc-<slug> \
  --reuse-values

# Force image pull after release
kubectl rollout restart deployment/sdlc-<slug> -n sdlc-<slug>
kubectl rollout status deployment/sdlc-<slug> -n sdlc-<slug> --timeout=120s
```

#### Pin a specific image tag (optional)

```bash
helm upgrade sdlc-<slug> ./k3s-fleet/deployments/helm/sdlc-server/ \
  --namespace sdlc-<slug> \
  --reuse-values \
  --set image.server=ghcr.io/orchard9/sdlc:X.Y.Z
```

#### Roll out ALL project instances

```bash
for ns in $(kubectl get ns -o name | grep 'namespace/sdlc-' | grep -v sdlc-hub | sed 's|namespace/||'); do
  echo "Rolling out $ns..."
  kubectl rollout restart deployment -n "$ns"
  kubectl rollout status deployment -n "$ns" --timeout=120s
done
```

---

### Phase 4: Health Verification

**Hub health (do this first):**
```bash
# Direct API check via kubectl port-forward
kubectl port-forward -n sdlc-hub svc/sdlc-hub 8081:80 &
curl -s http://localhost:8081/api/health | jq .
kill %1

# Or via public URL (requires auth cookie for most endpoints, health is open)
curl -s https://sdlc.threesix.ai/api/health
```

**Check pod logs for errors:**
```bash
kubectl logs -n sdlc-hub deployment/sdlc-hub --tail=50
kubectl logs -n sdlc-hub deployment/sdlc-hub --previous --tail=30  # if pod restarted
```

**Check events for failures:**
```bash
kubectl get events -n sdlc-hub --sort-by='.lastTimestamp' | tail -20
```

**Verify probes are passing:**
```bash
kubectl describe pod -n sdlc-hub -l app=sdlc-hub | grep -A5 "Liveness\|Readiness"
```

**Project instance health:**
```bash
kubectl port-forward -n sdlc-<slug> svc/sdlc-<slug> 8082:80 &
curl -s http://localhost:8082/api/health | jq .
kill %1

kubectl logs -n sdlc-<slug> deployment/sdlc-<slug> -c sdlc-server --tail=50
kubectl logs -n sdlc-<slug> deployment/sdlc-<slug> -c git-sync --tail=20
```

**Fleet-wide pod status:**
```bash
kubectl get pods -A | grep sdlc | grep -v Running
# Any output here = something is wrong
```

---

### Phase 5: Integration Checks

Verify hub integrations are healthy after rollout:

**OAuth:** Open `https://sdlc.threesix.ai` in a browser — should redirect to Google login (not 5xx).

**Gitea connectivity (from hub pod):**
```bash
kubectl exec -n sdlc-hub deployment/sdlc-hub -- \
  wget -qO- http://gitea.threesix.svc.cluster.local/api/v1/repos/search?limit=1 \
  -H "Authorization: token $THREE_SIX_GITEA" | head -c 200
```

**Woodpecker connectivity (from hub pod):**
```bash
kubectl exec -n sdlc-hub deployment/sdlc-hub -- \
  wget -qO- http://woodpecker-server.woodpecker.svc.cluster.local:8000/api/info | head -c 200
```

**Fleet API:**
```bash
# Hub must be able to list sdlc-* namespaces
kubectl get namespaces | grep sdlc
```

---

## Provisioning a New Instance

When adding a brand-new project to the fleet:

```bash
export KUBECONFIG=~/.kube/orchard9-k3sf.yaml

# 1. Copy TLS secret into new namespace
kubectl create namespace sdlc-<slug> --dry-run=client -o yaml | kubectl apply -f -
kubectl get secret sdlc-wildcard-tls -n sdlc-tls -o yaml | \
  sed 's/namespace: sdlc-tls/namespace: sdlc-<slug>/' | \
  kubectl apply -f -

# 2. Deploy via Helm
helm install sdlc-<slug> ./k3s-fleet/deployments/helm/sdlc-server/ \
  --namespace sdlc-<slug> \
  --set project.slug=<slug> \
  --set project.repo=orchard9/<repo> \
  --set auth.enabled=true

# 3. Watch rollout
kubectl rollout status deployment/sdlc-<slug> -n sdlc-<slug> --timeout=120s

# 4. Add Cloudflare DNS A record for <slug>.sdlc.threesix.ai
#    Points to: 208.122.204.172, .173, .174 (cluster ingress IPs)
#    Use: THREESIX_CLOUDFLARE_API_TOKEN + THREESIX_CLOUDFLARE_ZONE_ID env vars
```

---

## Secrets Reference (fresh environment only)

These secrets persist across rollouts. Only create if they don't exist:

```bash
export KUBECONFIG=~/.kube/orchard9-k3sf.yaml

# Check if secrets exist
kubectl get secrets -n sdlc-hub

# OAuth secret
kubectl create secret generic sdlc-hub-oauth \
  --namespace sdlc-hub \
  --from-literal=google-client-id="<GCP_OAUTH_CLIENT_ID>" \
  --from-literal=google-client-secret="<GCP_OAUTH_CLIENT_SECRET>" \
  --from-literal=session-secret="<32+_CHAR_RANDOM_STRING>"

# Fleet tokens
kubectl create secret generic sdlc-hub-fleet-tokens \
  --namespace sdlc-hub \
  --from-literal=gitea-api-token="<GITEA_ADMIN_TOKEN>" \
  --from-literal=woodpecker-api-token="<WOODPECKER_TOKEN>" \
  --from-literal=hub-service-tokens="<COMMA_SEPARATED_M2M_TOKENS>"

# Notify (OTP email delivery — key is in notify DB, not recoverable from admin API)
kubectl create secret generic sdlc-hub-notify \
  --namespace sdlc-hub \
  --from-literal=api-key="<notify_send_...>"

# GHCR image pull secret
kubectl create secret docker-registry ghcr-secret \
  --namespace sdlc-hub \
  --docker-server=ghcr.io \
  --docker-username=<github_user> \
  --docker-password=<github_pat>
```

---

## Topology Reference

```
Internet
    │
 Traefik (k3s ingress)
    │
    ├── sdlc.threesix.ai         → sdlc-hub namespace (no forwardAuth)
    │                               deployment: sdlc-hub
    │                               image: ghcr.io/orchard9/sdlc:latest
    │                               health: /api/health (port 8080)
    │
    └── <slug>.sdlc.threesix.ai  → sdlc-<slug> namespace (forwardAuth → hub /auth/verify)
                                    deployment: sdlc-<slug>
                                    containers: sdlc-server + git-sync (30s poll)
                                    health: /api/health (port 8080)
```

**Cluster access:** `KUBECONFIG=~/.kube/orchard9-k3sf.yaml`
**API server:** `https://msd6117.mjhst.com:6443` (via kubeconfig — do NOT use 100.79.2.8:6443)
**Gitea (Tailscale):** `http://100.79.2.8:30300`
**Gitea (cluster-internal):** `http://gitea.threesix.svc.cluster.local`
**Woodpecker (cluster-internal):** `http://woodpecker-server.woodpecker.svc.cluster.local:8000`
**Docker image:** `ghcr.io/orchard9/sdlc:latest`
**Wildcard TLS:** `sdlc-wildcard-tls` secret in `sdlc-tls` namespace
**Cluster ingress IPs:** `208.122.204.172`, `.173`, `.174`

---

## Do

1. Export `KUBECONFIG=~/.kube/orchard9-k3sf.yaml` before every kubectl/helm command.
2. Tag with semver before rolling out new code: `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. Wait for the GitHub Actions `docker` job to complete before restarting pods.
4. Deploy and verify the hub before touching any project instance.
5. Run `kubectl rollout status` after every restart to confirm the rollout succeeded.
6. Check `kubectl get pods -A | grep sdlc | grep -v Running` as the final fleet-wide health check.
7. Check pod logs if a rollout takes longer than 60s — don't wait blindly.

## Do Not

1. Do not push to `origin` without tagging if you intend to deploy — `main` pushes don't trigger Docker builds, only tags do.
2. Do not rollout all instances before the hub is healthy — forwardAuth will fail and all instances will return 401.
3. Do not use `100.79.2.8:6443` as the k8s API server — use the kubeconfig default (`msd6117.mjhst.com:6443`).
4. Do not `kubectl delete` secrets to recreate them unless you have the values — the notify API key is not recoverable from the admin API.
5. Do not skip `kubectl rollout status` — a restart command returns immediately; the pod may still be crashing.
6. Do not use `--force` on manifest applies unless you understand what's being replaced.

---

## Decision Points

**Rollout is stuck (pod not Ready after 90s):**
```bash
kubectl describe pod -n sdlc-hub -l app=sdlc-hub
kubectl logs -n sdlc-hub deployment/sdlc-hub --tail=100
```
Look for: ImagePullBackOff (GHCR auth), CrashLoopBackOff (app error), OOMKilled (memory limit), missing secret.

**Hub returns 5xx after rollout:**
- Check logs for panic or missing env var
- Verify secrets are present: `kubectl get secrets -n sdlc-hub`
- Check OAuth config: `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `SESSION_SECRET` must be set

**Git-sync container failing in project instance:**
```bash
kubectl logs -n sdlc-<slug> deployment/sdlc-<slug> -c git-sync --tail=50
```
Look for: Gitea auth failure, repo not found, network unreachable.

**ForwardAuth returning 401 for all instances:**
- Hub is the problem, not the instances
- Check hub health first: `curl -s https://sdlc.threesix.ai/api/health`
- Check hub logs for session/cookie errors
