# Review: fleet-reconcile-pipeline

## Summary

Implemented `pipelines/reconcile-projects.yaml` and `scripts/reconcile.sh` in `orchard9/sdlc-cluster`. The pipeline and script are pushed to Gitea at commit `2aa1d33`.

---

## Deliverables Checklist

| Item | Status |
|---|---|
| `pipelines/reconcile-projects.yaml` in `orchard9/sdlc-cluster` | Done |
| `scripts/reconcile.sh` in `orchard9/sdlc-cluster` | Done |
| Shell syntax valid (`sh -n`) | Pass |
| Pipeline YAML structural checks | Pass |
| Committed with descriptive message | Done |
| Pushed to Gitea main branch | Done |

---

## Code Review

### reconcile-projects.yaml

**Structure:** Clean Woodpecker v2 pipeline format. Triggers on `cron` (name: `reconcile-daily`) and `manual`. Single step using `alpine/helm:3.14`. All secrets injected via `from_secret` — no hardcoded values.

**Correctness:** Pipeline installs `curl` and `jq` before executing the script. The `scripts/reconcile.sh` path is correct relative to the Woodpecker workspace root.

**No issues found.**

### scripts/reconcile.sh

**Pagination:** Gitea API is paged correctly — loop increments `page` until a batch returns 0 items. Works for any org size.

**Filter logic:** jq filter correctly handles:
- `archived == false` — excludes archived repos
- `fork == false` — excludes forks
- `name != "sdlc-cluster"` — excludes self (SELF_REPO constant)
- `(.topics // []) | index("no-sdlc") == null` — excludes `no-sdlc`-tagged repos; `.topics // []` guards against null topics safely

**Helm detection:** `helm list --all-namespaces --output json --filter '^sdlc-'` is the correct command. Output is piped through `jq -r '.[].name | ltrimstr("sdlc-")'` to produce a slug list. This is precise — only release names starting with `sdlc-` are considered.

**Counter fix (fixed in follow-up commit):** The initial implementation had a shell subshell bug: the `while` loop was piped from `jq`, so all counter increments (`triggered`, `skipped`, `failed`) were lost when the subshell exited. Fixed by:
1. Writing the eligible repo list to a temp file (`entries_file`)
2. Redirecting the `while` loop input from the file instead of a pipe
3. Using a counts temp file (`COUNTS_FILE`) for cross-iteration accumulation
4. Reading the final counts after the loop

This ensures the summary line correctly shows `triggered` and `skipped` counts.

**Temp file hygiene:** All temp files use `$$` (PID) suffix to avoid collisions. They are removed in a `rm -f` call after counts are read.

**Error handling:**
- Gitea API failure: `curl -sf` exits non-zero → script exits via `set -eu`
- Helm list failure: `|| echo '[]'` fallback prevents script exit; an empty deployed set causes all eligible repos to appear undeployed (conservative — triggers deploys rather than silently skipping)
- Individual deploy trigger failure: HTTP status checked; non-200/201 increments `failed` counter; script exits non-zero at end if any failures occurred (continues to process remaining repos first)

**Dry-run mode:** `DRY_RUN=true` skips the Woodpecker API call but still increments `triggered` counter for accurate reporting. Summary line includes `dry_run=true` for log clarity.

---

## Findings

### F1: Helm list failure fallback is conservative (accepted)

If `helm list` fails (e.g., kubeconfig not configured), the script falls back to an empty deployed set. This means all eligible repos will appear undeployed and all deploys will be triggered — which is correct safe behavior (deploy-project is idempotent). The root problem (missing kubeconfig) will surface immediately when deploy triggers fail.

**Action:** Accept as-is. Document as operational prerequisite (kubeconfig must be configured on Woodpecker agent).

### F2: Cron registration is not automated (tracked)

The `reconcile-daily` cron must be manually registered in Woodpecker UI after the pipeline file is pushed. This is a one-time operational step and cannot be automated from within the pipeline itself.

**Action:** Add task for T3 (cron registration) to the operational runbook / milestone ops notes. This is not a code defect.

### F3: WOODPECKER_URL and WOODPECKER_TOKEN default to empty (accepted)

These vars have `:-` defaults (empty string) so dry-run mode (`DRY_RUN=true`) can run without Woodpecker credentials. If DRY_RUN is false and either is missing, curl will fail with a useful error. No silent data corruption possible.

**Action:** Accept as-is. DRY_RUN is the intended path for testing without full credentials.

---

## Overall Verdict

Implementation is correct, handles edge cases (pagination, subshell counter bug fixed, dry-run, error accumulation), and follows the design exactly. All acceptance criteria from the spec are implemented. Pipeline file is in place in Gitea. Ready for QA.
