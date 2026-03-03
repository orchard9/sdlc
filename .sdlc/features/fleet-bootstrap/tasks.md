# Tasks: fleet-bootstrap

## Task List

### T1: Write bootstrap.py script

Create `pipelines/bootstrap.py` in the `orchard9/sdlc-cluster` repo.

The script must:
- Read `GITEA_URL`, `GITEA_TOKEN` (or `gitea_token` env), `WOODPECKER_URL`, `WOODPECKER_TOKEN` (or `woodpecker_token` env), `CONCURRENCY`, `SKIP_REPOS`, `DRY_RUN` from environment
- Page through all repos in the `orchard9` Gitea org via `GET /api/v1/orgs/orchard9/repos?limit=50&page=N` until an empty page is returned
- Filter out repos listed in `SKIP_REPOS` (comma-separated, default `sdlc-cluster`)
- If `DRY_RUN=true`, print list of repos that would be triggered and exit 0
- Otherwise, trigger `deploy-project` pipeline for each qualifying repo via Woodpecker API `POST /api/repos/orchard9/sdlc-cluster/pipelines` with `variables`: `SDLC_PROJECT_SLUG`, `SDLC_REPO`, `SDLC_BRANCH=main`
- Use `ThreadPoolExecutor(max_workers=CONCURRENCY)` for bounded parallelism
- Print `[OK]` or `[FAIL]` per repo
- Print final summary: N triggered, K skipped, F failed
- Exit 1 if any triggers failed

Dependencies: Python stdlib only (`os`, `json`, `urllib.request`, `concurrent.futures`, `sys`)

---

### T2: Write bootstrap.yaml pipeline definition

Create `pipelines/bootstrap.yaml` in the `orchard9/sdlc-cluster` repo.

Pipeline spec:
- Step name: `bootstrap`
- Image: `python:3.12-alpine`
- Commands: `apk add --no-cache curl`, then `python3 pipelines/bootstrap.py`
- Environment variables declared: `GITEA_URL`, `GITEA_TOKEN`, `WOODPECKER_URL`, `WOODPECKER_TOKEN`, `CONCURRENCY`, `SKIP_REPOS`, `DRY_RUN`
- Secrets: `gitea_token`, `woodpecker_token`
- Trigger: `event: custom` (never fires on push)

---

### T3: Commit and push to orchard9/sdlc-cluster

Commit both new files to the `orchard9/sdlc-cluster` repo on threesix/gitea:

- `pipelines/bootstrap.py`
- `pipelines/bootstrap.yaml`

Commit message: `feat: add bootstrap pipeline for initial fleet provisioning`

Push to `origin main`.

Verify via Gitea API that the files appear in the repo at the expected paths.

---

### T4: Register Woodpecker secrets

Ensure the following secrets are registered in Woodpecker for the `orchard9/sdlc-cluster` repo:
- `gitea_token` — the `claude-agent` Gitea token (from `sdlc secrets env export gitea`)
- `woodpecker_token` — the Woodpecker admin API token

If they already exist (set by `fleet-deploy-pipeline` or `fleet-reconcile-pipeline`), verify they are correct. If absent, create them via the Woodpecker API:
```
POST /api/repos/{owner}/{repo}/secrets
```

---

### T5: Validate dry run in Woodpecker

Trigger the bootstrap pipeline in the Woodpecker UI (or via API) with `DRY_RUN=true`.

Verify:
- Pipeline completes with exit code 0
- Pipeline output lists all orchard9 repos (except `sdlc-cluster`)
- No `deploy-project` pipelines were created (check Woodpecker pipeline list)

This validates connectivity (Gitea API reachable) and skip-list logic without affecting the cluster.

---

### T6: (Optional) Run full bootstrap

Trigger the bootstrap pipeline without `DRY_RUN` to provision all existing orchard9 repos.

Verify:
- All `deploy-project` triggers fired (check Woodpecker pipeline list)
- Final summary shows 0 failures
- Spot-check 3-5 namespaces: `kubectl get pods -n sdlc-<repo-name>`

This task is marked optional because it depends on `fleet-deploy-pipeline`, `fleet-helm-chart`, `fleet-secrets-infra`, and `fleet-ingress-tls` all being delivered and verified first. If those features are not yet released, this task is deferred — the pipeline file itself (T1–T3) can be merged and tested with dry-run (T5) independently.
