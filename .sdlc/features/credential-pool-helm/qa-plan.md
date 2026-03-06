# QA Plan: credential-pool-helm

## Approach

This is a Helm chart configuration feature. QA is performed via `helm template` output
diffing (no Kubernetes cluster required) and optionally a dry-run against the staging cluster.

## Test Cases

### TC-1: Default values — feature disabled

**Command:**
```bash
cd k3s-fleet/deployments/helm/sdlc-server
helm template test . \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  --set gitea.url=http://100.79.2.8:30300
```

**Expected:**
- No `ExternalSecret` resource in output
- `Deployment` spec does NOT contain `DATABASE_URL` in any env block
- `Deployment` spec does NOT reference secret `postgres-sdlc-credentials`

**Pass criteria:** `grep DATABASE_URL` and `grep postgres-sdlc-credentials` return no output.

### TC-2: gsmKey set — feature enabled

**Command:**
```bash
cd k3s-fleet/deployments/helm/sdlc-server
helm template test . \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  --set gitea.url=http://100.79.2.8:30300 \
  --set postgres.externalSecret.gsmKey=k3sf-postgres-sdlc
```

**Expected ExternalSecret:**
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: postgres-sdlc-credentials
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: gcp-secret-manager
    kind: ClusterSecretStore
  target:
    name: postgres-sdlc-credentials
    creationPolicy: Owner
  data:
    - secretKey: DATABASE_URL
      remoteRef:
        key: k3sf-postgres-sdlc
        property: database_url
```

**Expected Deployment env (sdlc-server container):**
```yaml
- name: DATABASE_URL
  valueFrom:
    secretKeyRef:
      name: postgres-sdlc-credentials
      key: DATABASE_URL
```

**Pass criteria:**
- ExternalSecret present with `remoteRef.key: k3sf-postgres-sdlc`
- `secretKeyRef.name` matches `target.name` (`postgres-sdlc-credentials`)
- `DATABASE_URL` appears exactly once in the deployment env block

### TC-3: Custom gsmKey — parametric correctness

**Command:** same as TC-2 but `--set postgres.externalSecret.gsmKey=custom-secret-name`

**Expected:** `remoteRef.key: custom-secret-name` in ExternalSecret output.

**Pass criteria:** The gsmKey value propagates correctly — no hardcoded key name in template.

### TC-4: Chart lint

**Command:**
```bash
cd k3s-fleet/deployments/helm/sdlc-server
helm lint . --set project.slug=test --set project.repo=orchard9/test
```

**Pass criteria:** No errors; warnings acceptable only for missing required values that are
intentionally left empty in defaults.

### TC-5 (optional, staging cluster): Live ExternalSecret sync

**Prerequisites:** Access to k3s cluster with `ClusterSecretStore gcp-secret-manager` installed
and GCP secret `k3sf-postgres-sdlc` populated.

**Steps:**
1. Deploy a Helm release with `gsmKey: k3sf-postgres-sdlc` in a test namespace.
2. Check ExternalSecret status: `kubectl get externalsecret postgres-sdlc-credentials -n <ns>`
3. Verify `Ready` condition is `True`.
4. Check Secret exists: `kubectl get secret postgres-sdlc-credentials -n <ns>`
5. Verify pod starts: `kubectl get pods -n <ns>` — no `CreateContainerConfigError`.

**Pass criteria:** ExternalSecret `Ready=True`, Secret has `DATABASE_URL` key, pod Running.

## Regression Check

After applying changes, re-run TC-1 to confirm the disabled path is unaffected — existing
deployments without `gsmKey` must not be modified.
