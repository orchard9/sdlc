# QA Results: fleet-bootstrap

## Run Date

2026-03-03

## Environment

- Gitea: `http://100.79.2.8:30300` (threesix/gitea)
- Woodpecker: `https://ci.threesix.ai` (Woodpecker v3.13.0)
- Python: 3.14 (local test), 3.12-alpine (pipeline image target)
- orchard9 org repos: 4 (sdlc-cluster, qa-archived-test, qa-nosdk-test, reconcile-e2e-test)

---

## Test Results

### TC-1: Repository files exist and are correct

**Result: PASS**

Both files confirmed present in `orchard9/sdlc-cluster` on threesix/gitea:
- `pipelines/bootstrap.py`: 5,733 bytes, HTTP 200
- `pipelines/bootstrap.yaml`: 1,051 bytes, HTTP 200

Commit: `feat: add bootstrap pipeline for initial fleet provisioning`

---

### TC-2: bootstrap.yaml has correct Woodpecker event trigger

**Result: PASS**

`pipelines/bootstrap.yaml` contains `event: custom` — confirmed pipeline will not trigger on push or PR events. Manual/API trigger only.

---

### TC-3: Pagination logic handles multiple pages correctly

**Result: PASS**

Unit test with mocked Gitea API simulating:
- Page 1: 50 repos
- Page 2: 23 repos
- Page 3: empty (signals end)

`list_repos()` returned exactly 73 repos with exactly 3 API calls. Pagination logic is correct.

---

### TC-4: Skip list filters out sdlc-cluster

**Result: PASS**

Dry-run against live Gitea (4 repos in orchard9 org):
- `sdlc-cluster` correctly excluded from trigger list
- Output: "1 skipped: sdlc-cluster"
- 3 repos listed as "would trigger": `qa-archived-test`, `qa-nosdk-test`, `reconcile-e2e-test`
- `sdlc-cluster` did not appear in the "would trigger deploy-project for" list

---

### TC-5: DRY_RUN mode exits 0 without triggering any pipelines

**Result: PASS**

With `DRY_RUN=true` and `WOODPECKER_URL=http://mock` (mock, not called):
- Printed repo listing: 3 repos
- Summary: "Total: 3 repos (dry run, nothing triggered)"
- Exit code: 0
- No Woodpecker API calls made (mock URL would have failed if called)

---

### TC-6: Full run — all triggers fired successfully

**Result: DEFERRED**

Requires `fleet-deploy-pipeline` (v17/v18) to be delivered and `orchard9/sdlc-cluster` to be activated in Woodpecker CI. Both prerequisites are pending as of this QA run. This test case will be executed as part of the v18-fleet-automation milestone acceptance test.

---

### TC-7: Idempotency — re-running bootstrap does not cause errors

**Result: DEFERRED**

Depends on TC-6. Will be executed during milestone acceptance test.

---

## Summary

| TC | Description | Result |
|---|---|---|
| TC-1 | Files exist in repo | PASS |
| TC-2 | event: custom trigger | PASS |
| TC-3 | Pagination correctness | PASS |
| TC-4 | Skip list filters sdlc-cluster | PASS |
| TC-5 | DRY_RUN exits 0, no triggers | PASS |
| TC-6 | Full bootstrap run | DEFERRED |
| TC-7 | Idempotency | DEFERRED |

**Required test cases (TC-1 through TC-5): 5/5 PASS**

All required test cases pass per the QA plan. TC-6 and TC-7 are correctly deferred to milestone-level acceptance testing per the QA plan's stated completion criteria.

## Issues Observed

None blocking. Minor issues tracked in review:
1. `GITEA_URL` has no default in `bootstrap.yaml` — operator must set at trigger time (documented in pipeline comments)
2. `WOODPECKER_URL` default assumes `woodpecker` namespace — may need correction when confirmed

**QA Verdict: PASS** — feature is ready to merge.
