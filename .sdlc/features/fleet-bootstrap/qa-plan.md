# QA Plan: fleet-bootstrap

## Scope

The fleet-bootstrap feature delivers two files to `orchard9/sdlc-cluster`:
- `pipelines/bootstrap.py` — the bootstrap logic
- `pipelines/bootstrap.yaml` — the Woodpecker pipeline definition

QA validates correctness, idempotency, skip-list logic, and Woodpecker integration without requiring a full cluster run until the optional T6 task is executed.

---

## Test Cases

### TC-1: Repository files exist and are correct

**What to verify:** Both pipeline files are present in `orchard9/sdlc-cluster` on threesix/gitea.

**How:**
```bash
# Export Gitea credentials
eval $(sdlc secrets env export gitea)

# Check bootstrap.py exists
curl -s -H "Authorization: token $GITEA_TOKEN" \
  "http://100.79.2.8:30300/api/v1/repos/orchard9/sdlc-cluster/contents/pipelines/bootstrap.py" \
  | jq '.name'
# Expected: "bootstrap.py"

# Check bootstrap.yaml exists
curl -s -H "Authorization: token $GITEA_TOKEN" \
  "http://100.79.2.8:30300/api/v1/repos/orchard9/sdlc-cluster/contents/pipelines/bootstrap.yaml" \
  | jq '.name'
# Expected: "bootstrap.yaml"
```

**Pass criteria:** Both files return HTTP 200 with the correct filename.

---

### TC-2: bootstrap.yaml has correct Woodpecker event trigger

**What to verify:** The pipeline is configured as `event: custom` and does not trigger on push.

**How:** Read `pipelines/bootstrap.yaml` content (via Gitea API or local clone) and confirm:
```yaml
when:
  event: custom
```

**Pass criteria:** `grep "event: custom" pipelines/bootstrap.yaml` returns a match.

---

### TC-3: bootstrap.py handles pagination correctly

**What to verify:** The script pages through all Gitea results when more than 50 repos exist.

**How:** Unit-test the pagination logic by mocking the Gitea API responses:

```python
# Simulate two pages: page 1 returns 50 repos, page 2 returns 30, page 3 returns empty
# Verify list_repos() returns 80 items

from unittest.mock import patch, MagicMock
import json, sys, os
sys.path.insert(0, "pipelines")
os.environ.update({"GITEA_URL": "http://mock", "GITEA_TOKEN": "t",
                   "WOODPECKER_URL": "http://mock", "WOODPECKER_TOKEN": "t"})

import bootstrap
# Verify list_repos collects all pages
```

**Pass criteria:** `list_repos()` returns the correct total count across all pages.

---

### TC-4: Skip list filters out sdlc-cluster

**What to verify:** `sdlc-cluster` is excluded from the trigger list.

**How:**
1. With `DRY_RUN=true`, run the pipeline or test locally:
```bash
GITEA_URL=... GITEA_TOKEN=... WOODPECKER_URL=http://mock WOODPECKER_TOKEN=t \
  DRY_RUN=true SKIP_REPOS=sdlc-cluster python3 pipelines/bootstrap.py
```
2. Inspect output — `sdlc-cluster` must NOT appear in the trigger list.

**Pass criteria:** Output does not contain `sdlc-cluster`; count of triggered repos = total repos minus skipped.

---

### TC-5: DRY_RUN mode exits 0 without triggering any pipelines

**What to verify:** With `DRY_RUN=true`, the script prints repos and exits 0 without calling the Woodpecker trigger API.

**How:** Trigger via Woodpecker UI with `DRY_RUN=true` (or run locally with mocked Gitea returning a small list).

Verify:
- Pipeline exits with code 0
- No new `deploy-project` pipeline runs appear in Woodpecker after the bootstrap run
- Output includes "DRY_RUN=true" message and repo listing

**Pass criteria:** No new pipeline runs created; exit code 0; output lists repos.

---

### TC-6: Full run — all triggers fired successfully

**Precondition:** `fleet-deploy-pipeline`, `fleet-helm-chart`, `fleet-secrets-infra` must be delivered and verified.

**What to verify:** Running bootstrap without `DRY_RUN` triggers a `deploy-project` pipeline for every qualifying orchard9 repo.

**How:**
1. Trigger bootstrap pipeline (no `DRY_RUN` override) in Woodpecker UI
2. Monitor Woodpecker pipeline list for new `deploy-project` runs
3. Check final summary in bootstrap logs: "N triggered, 0 failed"

**Pass criteria:**
- Pipeline exits 0
- Summary reports 0 failures
- Woodpecker shows N new `deploy-project` pipeline runs (one per non-skipped repo)

---

### TC-7: Idempotency — re-running bootstrap does not cause errors

**What to verify:** Running bootstrap a second time (after all repos are already deployed) completes without error.

**Precondition:** TC-6 must have completed successfully.

**How:** Trigger the bootstrap pipeline a second time with default settings.

**Pass criteria:**
- Pipeline exits 0
- All `deploy-project` pipelines complete successfully (each is a `helm upgrade --install` no-op or fast reapply)
- No new namespaces created (all already exist)

---

## Environment Requirements

| Requirement | Notes |
|---|---|
| `orchard9/sdlc-cluster` accessible on threesix/gitea | Required for TC-1, TC-2 |
| Python 3.x with stdlib | Required for TC-3, TC-4, TC-5 (local) |
| Woodpecker CI accessible | Required for TC-5, TC-6, TC-7 |
| `fleet-deploy-pipeline` delivered | Required for TC-6, TC-7 |
| k3s cluster accessible (`KUBECONFIG`) | Required to verify pods in TC-6, TC-7 |

---

## QA Completion Criteria

All required test cases (TC-1 through TC-5) must pass. TC-6 and TC-7 are contingent on upstream fleet features being delivered and are verified as part of the v18-fleet-automation milestone acceptance test rather than this feature's individual QA. Record results and any observed issues in `qa-results.md`.
