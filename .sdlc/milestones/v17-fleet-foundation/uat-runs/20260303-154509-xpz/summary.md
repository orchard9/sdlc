# UAT Summary: v17-fleet-foundation

**Run ID:** 20260303-154509-xpz
**Date:** 2026-03-03
**Milestone:** sdlc-cluster: Fleet deployment foundation
**Verdict:** PASS_WITH_TASKS

---

## Vision

> One sdlc-server pod per orchard9 project, zero Longhorn PVCs, managed by Woodpecker — deploy the fleet without touching the sdlc Rust codebase. A developer runs one Woodpecker pipeline and gets a live sdlc UI at `<slug>.sdlc.threesix.ai`.

---

## Feature Status

All 4 features are in `released` phase at time of UAT.

---

## Checklist Results

### Fleet Repo Scaffold (`fleet-repo-scaffold`)

| # | Check | Result | Evidence |
|---|---|---|---|
| 1 | `orchard9/sdlc-cluster` exists on Gitea | ✅ PASS | `full_name: orchard9/sdlc-cluster`, not empty |
| 2 | `.woodpecker.yml` present | ✅ PASS | HTTP 200 |
| 3 | `README.md` present with correct content | ✅ PASS | "Manages sdlc-server deployments for the orchard9 fleet." |
| 4 | `helm/sdlc-server/Chart.yaml` present | ✅ PASS | HTTP 200; apiVersion: v2, name: sdlc-server, version: 0.1.0 |
| 5 | `helm/sdlc-server/values.yaml` present | ✅ PASS | HTTP 200 |
| 6 | `helm/sdlc-server/templates/.gitkeep` present | ✅ PASS | HTTP 200 |
| 7 | `pipelines/deploy-project.yaml.stub` present | ✅ PASS | HTTP 200 |
| 8 | `external-secrets/cluster-secret-store.yaml.stub` present | ✅ PASS | HTTP 200 |
| 9 | `.woodpecker.yml` runs `helm lint helm/sdlc-server` on `main` push | ✅ PASS | Content confirmed |
| 10 | Initial commit message `chore: scaffold sdlc-cluster repo` present | ✅ PASS | Found in git log |

**Scaffold result: 10/10 PASS**

---

### Helm Chart (`fleet-helm-chart`)

| # | Check | Result | Evidence |
|---|---|---|---|
| 1 | Chart exists at `k3s-fleet/deployments/helm/sdlc-server/` | ✅ PASS | Local files present |
| 2 | `helm lint` passes (0 failures) | ✅ PASS | "1 chart(s) linted, 0 chart(s) failed" |
| 3 | 5+ resource kinds rendered by `helm template` | ✅ PASS | Namespace, Service, Deployment, Ingress, ExternalSecret, ClusterSecretStore (6 total) |
| 4 | Deployment has 2 containers: `sdlc-server` + `git-sync` | ✅ PASS | Confirmed in rendered template |
| 5 | ExternalSecret references `gcp-secret-store` secretStore | ⚠️ PARTIAL | References `gcp-secret-store` but cluster has `gcp-secret-manager` — **name mismatch** |
| 6 | ExternalSecret references `sdlc-fleet-gitea` GSM key | ✅ PASS | Confirmed |
| 7 | Ingress host is `test.sdlc.threesix.ai` | ✅ PASS | Confirmed in rendered template |
| 8 | Namespace is `sdlc-test` | ✅ PASS | Confirmed |
| 9 | Chart committed to a git repo | ❌ FAIL | `deployments/helm/sdlc-server/` is UNTRACKED in k3s-fleet — never committed |

**Helm chart result: 7/9 PASS, 1 PARTIAL, 1 FAIL**

**Task created:** Commit Helm chart templates to k3s-fleet repo and fix ClusterSecretStore name (`gcp-secret-manager` not `gcp-secret-store`).

---

### Secrets Infrastructure (`fleet-secrets-infra`)

| # | Check | Result | Evidence |
|---|---|---|---|
| 1 | GCP Secret `sdlc-fleet-gitea` exists | ✅ PASS | `gcloud secrets list` confirmed |
| 2 | Secret has all 4 fields: token, url, user, org | ✅ PASS | `{"org","token","url","user"}` all present |
| 3 | `url` = `http://100.79.2.8:30300` | ✅ PASS | Confirmed |
| 4 | `user` = `claude-agent` | ✅ PASS | Confirmed |
| 5 | ExternalSecret `sdlc-fleet-gitea-credentials` synced | ✅ PASS | `status: True`, `reason: SecretSynced` in `projects` namespace |
| 6 | k8s Secret keys present: GITEA_TOKEN, GITEA_URL, GITEA_USER, GITEA_ORG | ✅ PASS | All 4 keys confirmed |
| 7 | `ClusterSecretStore` named `gcp-secret-store` exists | ❌ FAIL | Cluster has `gcp-secret-manager` — wrong name; Helm chart references `gcp-secret-store` |
| 8 | `cluster-secret-store.yaml` committed to sdlc-cluster | ❌ FAIL | Only `.stub` placeholder in repo |

**Secrets result: 6/8 PASS, 2 FAIL**

**Note:** The GCP secret and ESO syncing work correctly. The gap is that `ClusterSecretStore` name in the Helm chart (`gcp-secret-store`) doesn't match the actual cluster resource (`gcp-secret-manager`), and the ESO manifest wasn't committed to sdlc-cluster.

---

### Deploy Pipeline (`fleet-deploy-pipeline`)

| # | Check | Result | Evidence |
|---|---|---|---|
| 1 | `pipelines/deploy-project.yaml` exists in sdlc-cluster | ❌ FAIL | Only `deploy-project.yaml.stub` found; actual file is 404 |
| 2 | Pipeline YAML has `when: event: custom` | N/A | File not present |
| 3 | Pipeline uses `helm upgrade --install` with `--create-namespace` | N/A | File not present |
| 4 | Namespace constructed as `sdlc-${SLUG}` | N/A | File not present |
| 5 | Ingress host constructed as `${SLUG}.sdlc.threesix.ai` | N/A | File not present |
| 6 | Guard clauses for missing SDLC_PROJECT_SLUG | N/A | File not present |
| 7 | Woodpecker server running in cluster | ✅ PASS | `woodpecker-server-5f59789ccb-vwpj9` Running in `threesix` namespace |
| 8 | No sdlc-server pods currently running | 📋 NOTE | No `sdlc-*` namespaces with pods; only `sdlc-tls` namespace exists |

**Deploy pipeline result: 1/2 PASS, 1 FAIL (other checks N/A due to missing file)**

**Note:** Review doc for fleet-deploy-pipeline claimed commit `86399af4` added `deploy-project.yaml`, but this commit SHA does not exist in `orchard9/sdlc-cluster`. The file was never pushed.

---

## Gap Summary

| Gap | Impact | Task |
|---|---|---|
| Helm chart templates untracked in k3s-fleet | Pipeline can't deploy — chart not in version control | Commit `deployments/helm/sdlc-server/` to k3s-fleet |
| `deploy-project.yaml` missing from sdlc-cluster | Woodpecker can't trigger deployment | Create and push `pipelines/deploy-project.yaml` |
| ClusterSecretStore name mismatch | Per-project ExternalSecrets would fail | Fix chart to reference `gcp-secret-manager` not `gcp-secret-store` |
| No actual sdlc-server instances running | Vision not yet validated end-to-end | Run first deployment after above gaps are closed |

---

## Tasks Created

1. Commit `deployments/helm/sdlc-server/` templates to k3s-fleet repo (fleet-helm-chart)
2. Add `pipelines/deploy-project.yaml` to `orchard9/sdlc-cluster` (fleet-deploy-pipeline)
3. Fix ClusterSecretStore reference in Helm chart from `gcp-secret-store` to `gcp-secret-manager` (fleet-helm-chart)
4. Run first integration deploy to validate end-to-end: `sdlc` project as pilot (v17 integration test)

---

## Overall Assessment

The milestone's foundational scaffolding is substantially complete:
- The `orchard9/sdlc-cluster` repo exists with correct structure
- The Helm chart logic is correct and renders valid Kubernetes manifests
- GCP secrets are provisioned and ESO syncing works
- Woodpecker is operational in the cluster

However, three critical artifacts were never committed to their target repos, making the pipeline non-functional as a complete system. The milestone can advance with tracked tasks; a follow-up integration run will close the remaining gaps.

**Verdict: PASS_WITH_TASKS** (4 tasks)
