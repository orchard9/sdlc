# Tasks: fleet-deploy-pipeline

## T1 — Create `pipelines/` directory in `orchard9/sdlc-cluster`

Create the `pipelines/` directory in the `orchard9/sdlc-cluster` repository (depends on `fleet-repo-scaffold` having initialized the repo).

**Done when:** `pipelines/` directory exists at the repo root.

---

## T2 — Write `pipelines/deploy-project.yaml`

Author the Woodpecker pipeline file exactly as designed:
- `when: event: custom` trigger
- Single `deploy` step using `alpine/helm:3.14`
- Accepts `SDLC_PROJECT_SLUG`, `SDLC_REPO`, `SDLC_BRANCH` (default `main`)
- Constructs namespace as `sdlc-${SLUG}` and release as `sdlc-${SLUG}`
- Runs `helm upgrade --install` with `--create-namespace`, `--atomic`, `--timeout 5m`, `--wait`
- Sets `project.slug`, `project.repo`, `project.branch`, `ingress.host` via `--set`

**Done when:** File exists at `pipelines/deploy-project.yaml` with correct content.

---

## T3 — Validate YAML syntax

Run `yamllint` or equivalent on `deploy-project.yaml` to confirm no syntax errors.

**Done when:** YAML parses cleanly with no errors.

---

## T4 — Commit and push to `orchard9/sdlc-cluster`

Commit `pipelines/deploy-project.yaml` with message `feat: add deploy-project pipeline` and push to the `main` branch.

**Done when:** File is visible in the Gitea UI at `orchard9/sdlc-cluster/pipelines/deploy-project.yaml`.

---

## T5 — Verify pipeline appears in Woodpecker UI

Open the Woodpecker dashboard for `orchard9/sdlc-cluster` and confirm the `deploy-project` pipeline is listed and activatable.

**Done when:** Pipeline visible in Woodpecker UI for the repo.

---

## T6 — Manual trigger smoke test

Trigger the pipeline manually from Woodpecker UI (or API) with:
- `SDLC_PROJECT_SLUG=test-project`
- `SDLC_REPO=orchard9/test-project`

Verify:
- Pipeline run completes green
- `kubectl get ns sdlc-test-project` returns the namespace
- `kubectl get deployment -n sdlc-test-project` shows Running state
- Ingress exists for `test-project.sdlc.threesix.ai`

**Done when:** All kubectl checks pass after the first trigger.

---

## T7 — Idempotency re-run test

Trigger the pipeline a second time with the same parameters (`SDLC_PROJECT_SLUG=test-project`, `SDLC_REPO=orchard9/test-project`).

**Done when:** Second run completes green with no errors (helm reports no-op or fast reapply).

---

## T8 — Clean up test namespace

Remove the `sdlc-test-project` namespace and Helm release created during smoke testing.

```bash
helm uninstall sdlc-test-project -n sdlc-test-project
kubectl delete ns sdlc-test-project
```

**Done when:** Namespace no longer exists in the cluster.
