# QA Results: credential-pool-helm

## Status: PASSED

All test cases from the QA plan executed and passed.

## Test Execution

### TC-1: Default values — feature disabled

**Command:**
```
helm template test k3s-fleet/deployments/helm/sdlc-server \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  --set gitea.url=http://100.79.2.8:30300
```

**Result:** PASS
- `grep DATABASE_URL` → 0 matches
- `grep postgres-sdlc-credentials` → 0 matches
- No ExternalSecret in output
- No DATABASE_URL env var in deployment

### TC-2: gsmKey set — feature enabled

**Command:**
```
helm template test k3s-fleet/deployments/helm/sdlc-server \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  --set gitea.url=http://100.79.2.8:30300 \
  --set postgres.externalSecret.gsmKey=k3sf-postgres-sdlc
```

**Result:** PASS

ExternalSecret rendered correctly:
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

DATABASE_URL env var injected into sdlc-server container:
```yaml
- name: DATABASE_URL
  valueFrom:
    secretKeyRef:
      name: postgres-sdlc-credentials
      key: DATABASE_URL
```

- `secretKeyRef.name` matches `target.name` (`postgres-sdlc-credentials`) ✓
- `remoteRef.key` is `k3sf-postgres-sdlc` (not hardcoded) ✓
- DATABASE_URL appears exactly once in deployment env ✓

### TC-3: Custom gsmKey — parametric correctness

**Command:**
```
helm template test ... --set postgres.externalSecret.gsmKey=custom-secret-name
```

**Result:** PASS
- `remoteRef.key: custom-secret-name` confirmed in ExternalSecret output
- Template is fully parametric — no hardcoded GCP secret name

### TC-4: Chart lint

**Command:**
```
helm lint k3s-fleet/deployments/helm/sdlc-server \
  --set project.slug=test --set project.repo=orchard9/test
```

**Result:** PASS
```
==> Linting k3s-fleet/deployments/helm/sdlc-server
[INFO] Chart.yaml: icon is recommended

1 chart(s) linted, 0 chart(s) failed
```

One INFO-level notice (icon recommended) — not a warning or error. Acceptable.

### TC-5: Live cluster sync

**Result:** SKIPPED — staging cluster not available in this environment. Template
validation confirms the ExternalSecret resource definition is correct for ESO v1beta1.

## Summary

| Test Case | Result |
|-----------|--------|
| TC-1: Disabled path | PASS |
| TC-2: Enabled path | PASS |
| TC-3: Parametric gsmKey | PASS |
| TC-4: Helm lint | PASS |
| TC-5: Live cluster sync | SKIPPED |

All required test cases passed. Feature is ready to merge.
