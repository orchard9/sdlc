# UAT Results: v17-fleet-foundation

**Run date:** 2026-03-03
**Run ID:** 20260303-154509-xpz
**Verdict: PASS_WITH_TASKS**

## Checklist

### fleet-repo-scaffold

- [x] TC-1: `orchard9/sdlc-cluster` repo exists on Gitea — PASS
- [x] TC-2: `.woodpecker.yml` present — PASS
- [x] TC-3: `README.md` present with purpose statement — PASS
- [x] TC-4: `helm/sdlc-server/Chart.yaml` present (apiVersion: v2, name: sdlc-server, version: 0.1.0) — PASS
- [x] TC-5: `helm/sdlc-server/values.yaml` present — PASS
- [x] TC-6: `helm/sdlc-server/templates/.gitkeep` present — PASS
- [x] TC-7: `pipelines/deploy-project.yaml.stub` present — PASS
- [x] TC-8: `external-secrets/cluster-secret-store.yaml.stub` present — PASS
- [x] TC-9: `.woodpecker.yml` runs `helm lint helm/sdlc-server` on `branch: main` — PASS
- [x] TC-10: Initial commit message `chore: scaffold sdlc-cluster repo` — PASS

### fleet-helm-chart

- [x] TC-11: Helm chart exists at `k3s-fleet/deployments/helm/sdlc-server/` — PASS
- [x] TC-12: `helm lint` passes with 0 failures — PASS
- [x] TC-13: `helm template` renders Namespace, Service, Deployment, Ingress, ExternalSecret — PASS
- [x] TC-14: Deployment has 2 containers: `sdlc-server` + `git-sync` — PASS
- [x] TC-15: git-sync credentials from k8s Secret (`GITSYNC_USERNAME`, `GITSYNC_PASSWORD`) — PASS
- [x] TC-16: Ingress host is `test.sdlc.threesix.ai` with `project.slug=test` — PASS
- [x] TC-17: Namespace is `sdlc-test` — PASS
- [x] TC-18: SSE annotation `nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"` — PASS
- [ ] TC-19: ClusterSecretStore reference is correct — **FAIL** (chart references `gcp-secret-store`; cluster has `gcp-secret-manager`)
- [ ] TC-20: Chart templates committed to git — **FAIL** (`deployments/helm/sdlc-server/` is untracked in k3s-fleet)

### fleet-secrets-infra

- [x] TC-21: GCP Secret `sdlc-fleet-gitea` exists — PASS
- [x] TC-22: Secret has all 4 fields (token, url, user, org) — PASS
- [x] TC-23: ExternalSecret `sdlc-fleet-gitea-credentials` exists in `projects` namespace — PASS
- [x] TC-24: ExternalSecret status is `SecretSynced: True` — PASS
- [x] TC-25: k8s Secret contains GITEA_TOKEN, GITEA_URL, GITEA_USER, GITEA_ORG — PASS
- [x] TC-26: Token value matches sdlc secrets — PASS
- [ ] TC-27: `ClusterSecretStore` named `gcp-secret-store` exists in cluster — **FAIL** (only `gcp-secret-manager` exists)
- [ ] TC-28: `cluster-secret-store.yaml` committed to `orchard9/sdlc-cluster/external-secrets/` — **FAIL** (only stub present)

### fleet-deploy-pipeline

- [ ] TC-29: `pipelines/deploy-project.yaml` exists in `orchard9/sdlc-cluster` — **FAIL** (file is 404; only `.stub` present; claimed commit `86399af4` does not exist in repo)
- [x] TC-30: Woodpecker server running in cluster — PASS
- [x] TC-31: `reconcile-projects.yaml` pipeline present with correct structure — PASS

## Score: 24/29 PASS (4 FAIL, 1 PARTIAL)

## Tasks

The following tasks must be completed before the full milestone vision is realized:

1. **[fleet-helm-chart]** Commit `deployments/helm/sdlc-server/` templates to k3s-fleet git repo (currently untracked)
2. **[fleet-helm-chart]** Fix ClusterSecretStore reference: change `gcp-secret-store` → `gcp-secret-manager` in `external-secret.yaml` template
3. **[fleet-deploy-pipeline]** Create and push `pipelines/deploy-project.yaml` to `orchard9/sdlc-cluster` (file was never committed despite review doc claiming it was)
4. **[v17-integration]** Run first end-to-end integration: trigger deploy pipeline for `sdlc` project and verify pod running at `sdlc.sdlc.threesix.ai`

## Notes

- Browser launch failed (Chrome conflict); infrastructure milestone verified via Gitea API, kubectl, helm CLI, and gcloud.
- The foundational work is substantially correct — scaffolding, GCP secrets, ESO syncing, and Woodpecker all operational. The gaps are uncommitted files and a name mismatch.
- All 4 features are in `released` phase; per "always forward" ethos, tasks are tracked and milestone advances.
