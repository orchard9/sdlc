# QA Plan: fleet-reconcile-pipeline

## Scope

Verify that `pipelines/reconcile-projects.yaml` and `scripts/reconcile.sh` in `orchard9/sdlc-cluster` correctly implement the automated fleet reconciliation loop described in the spec.

---

## Test Cases

### TC-1: reconcile.sh syntax check

**Goal:** Confirm the shell script is syntactically valid and POSIX-compatible.

**Steps:**
1. Check out `orchard9/sdlc-cluster`
2. Run: `sh -n scripts/reconcile.sh`

**Pass:** Command exits 0, no errors.

---

### TC-2: Pipeline YAML lint

**Goal:** Confirm the Woodpecker pipeline YAML is valid.

**Steps:**
1. Use `woodpecker-cli lint pipelines/reconcile-projects.yaml` (or a YAML linter)
2. Alternatively, push the file and check Woodpecker detects it without parse errors

**Pass:** No YAML errors; Woodpecker shows `reconcile-projects.yaml` in the pipeline list.

---

### TC-3: Dry-run mode — no deploys triggered

**Goal:** Confirm `DRY_RUN=true` logs all repos without calling Woodpecker API.

**Steps:**
1. Trigger the reconcile pipeline manually with `DRY_RUN=true` (via Woodpecker UI or API)
2. Inspect pipeline log output

**Pass:**
- Pipeline exits 0
- Log shows at least one entry per eligible repo (`already deployed:` or `→ triggering deploy for:`)
- Summary line present: `Reconcile summary: checked=N triggered=M skipped=K`
- No Woodpecker API POST requests in the log (confirming dry-run)

---

### TC-4: Exclusion filter — archived repo not triggered

**Goal:** Confirm archived repos are excluded from reconcile.

**Steps:**
1. In threesix/gitea: create `orchard9/qa-archived-test`, immediately archive it
2. Trigger dry-run reconcile
3. Inspect log for `qa-archived-test`

**Pass:** `qa-archived-test` does not appear in the dry-run trigger log.

**Cleanup:** Delete `orchard9/qa-archived-test` from Gitea.

---

### TC-5: Exclusion filter — fork repo not triggered

**Goal:** Confirm forked repos are excluded.

**Steps:**
1. In threesix/gitea: fork any existing repo into `orchard9/qa-fork-test`
2. Trigger dry-run reconcile
3. Inspect log for `qa-fork-test`

**Pass:** `qa-fork-test` does not appear in the dry-run trigger log.

**Cleanup:** Delete `orchard9/qa-fork-test`.

---

### TC-6: Exclusion filter — no-sdlc topic repo not triggered

**Goal:** Confirm repos with `no-sdlc` topic are excluded.

**Steps:**
1. Create `orchard9/qa-nosdk-test` with topic `no-sdlc`
2. Trigger dry-run reconcile
3. Inspect log for `qa-nosdk-test`

**Pass:** `qa-nosdk-test` does not appear in the dry-run trigger log.

**Cleanup:** Delete `orchard9/qa-nosdk-test`.

---

### TC-7: Exclusion filter — sdlc-cluster itself not triggered

**Goal:** Confirm `orchard9/sdlc-cluster` is never targeted.

**Steps:**
1. Trigger dry-run reconcile
2. Search log for `sdlc-cluster`

**Pass:** `sdlc-cluster` does not appear as a deploy target in the log.

---

### TC-8: Already-deployed repos are not re-triggered

**Goal:** Confirm repos with an existing `sdlc-<slug>` Helm release are skipped.

**Steps:**
1. Identify any repo that already has a deployed `sdlc-<slug>` release in the cluster
2. Trigger dry-run reconcile
3. Inspect log for that slug

**Pass:** Log shows `already deployed: <slug>` — no trigger fired.

---

### TC-9: New undeployed repo triggers deploy (live)

**Goal:** Confirm the reconcile loop fires a deploy for a genuinely new repo.

**Steps:**
1. Create `orchard9/reconcile-e2e-test` in Gitea (no `no-sdlc` tag, not archived, not fork)
2. Confirm no `sdlc-reconcile-e2e-test` Helm release exists
3. Trigger live reconcile (DRY_RUN=false)
4. Observe Woodpecker: confirm `deploy-project.yaml` was triggered with `SDLC_PROJECT_SLUG=reconcile-e2e-test`
5. Run reconcile again — confirm `reconcile-e2e-test` now appears as "already deployed"

**Pass:**
- First run: pipeline triggered, Woodpecker shows a `deploy-project` run for `reconcile-e2e-test`
- Second run: log shows `already deployed: reconcile-e2e-test`

**Cleanup:** Delete `orchard9/reconcile-e2e-test` from Gitea; manually helm uninstall `sdlc-reconcile-e2e-test` if deploy completed.

---

### TC-10: Cron schedule registered

**Goal:** Confirm the `reconcile-daily` cron is registered in Woodpecker.

**Steps:**
1. Navigate to Woodpecker UI → `orchard9/sdlc-cluster` → Settings → Cron
2. Verify `reconcile-daily` cron exists with schedule `0 2 * * *`

**Pass:** Cron entry exists, schedule is `0 2 * * *`, branch is `main`.

---

### TC-11: Summary counts are accurate

**Goal:** Confirm summary line arithmetic is correct.

**Steps:**
1. From TC-3 or TC-9 dry-run logs, count the number of `→ triggering` and `already deployed:` lines manually
2. Compare against the `Reconcile summary: checked=N triggered=M skipped=K` line

**Pass:** `checked = triggered + skipped`, values match manual count.

---

## Pass Criteria

All 11 test cases must pass. TC-9 (live end-to-end) is the most critical — if any single deploy trigger cannot fire, the feature is not ready.

## Regression Risk

- If Gitea pagination breaks (API response format change), reconcile will silently skip repos. TC-3 dry-run output should be spot-checked against known repo count.
- If `helm list` output format changes, the deployed-set computation will break. TC-8 guards against false positives; TC-9 guards against false negatives.
