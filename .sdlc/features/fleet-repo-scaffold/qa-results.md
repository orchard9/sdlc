# QA Results: Scaffold orchard9/sdlc-cluster Repo

## Result: PASS

All 7 test cases passed. No failures.

---

## TC-1: Repository exists on Gitea — PASS

```
GET /api/v1/repos/orchard9/sdlc-cluster → 200
full_name: orchard9/sdlc-cluster
private: false
```

---

## TC-2: All required files present — PASS

| File | HTTP |
|---|---|
| `.woodpecker.yml` | 200 |
| `README.md` | 200 |
| `helm/sdlc-server/Chart.yaml` | 200 |
| `helm/sdlc-server/values.yaml` | 200 |
| `helm/sdlc-server/templates/.gitkeep` | 200 |
| `pipelines/deploy-project.yaml.stub` | 200 |
| `pipelines/reconcile-projects.yaml.stub` | 200 |
| `external-secrets/cluster-secret-store.yaml.stub` | 200 |

---

## TC-3: Chart.yaml is valid — PASS

```yaml
apiVersion: v2
name: sdlc-server
version: 0.1.0
description: Helm chart for deploying sdlc-server instances
```

Contains all required fields: `apiVersion: v2`, `name: sdlc-server`, `version: 0.1.0`.

---

## TC-4: helm lint passes — PASS

```
==> Linting helm/sdlc-server
[INFO] Chart.yaml: icon is recommended

1 chart(s) linted, 0 chart(s) failed
Exit code: 0
```

No errors or failures. INFO message about icon is non-blocking.

---

## TC-5: .woodpecker.yml contains helm lint step — PASS

```yaml
pipeline:
  lint-helm:
    image: alpine/helm:latest
    commands:
      - helm lint helm/sdlc-server
    when:
      branch: main
```

Contains `helm lint helm/sdlc-server`, `alpine/helm` image, and `branch: main` trigger.

---

## TC-6: Initial commit message is correct — PASS

```
Latest commit: chore: scaffold sdlc-cluster repo
```

Matches spec exactly.

---

## TC-7: README.md describes purpose — PASS

```
# sdlc-cluster

Manages sdlc-server deployments for the orchard9 fleet.
```

Contains reference to sdlc-server deployments and orchard9 fleet.

---

## Summary

| TC | Result |
|---|---|
| TC-1: Repository exists | PASS |
| TC-2: All required files | PASS |
| TC-3: Chart.yaml valid | PASS |
| TC-4: helm lint passes | PASS |
| TC-5: .woodpecker.yml correct | PASS |
| TC-6: Commit message correct | PASS |
| TC-7: README describes purpose | PASS |

**Overall: 7/7 PASS**
