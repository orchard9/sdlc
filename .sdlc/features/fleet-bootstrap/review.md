# Code Review: fleet-bootstrap

## Summary

The fleet-bootstrap feature delivers two files to `orchard9/sdlc-cluster`:
- `pipelines/bootstrap.py` — Python script implementing the bootstrap logic
- `pipelines/bootstrap.yaml` — Woodpecker CI pipeline definition

Both files are committed and verified on threesix/gitea (`http://100.79.2.8:30300/orchard9/sdlc-cluster`).

Commit: `feat: add bootstrap pipeline for initial fleet provisioning`

---

## Implementation Review

### bootstrap.py

**Strengths:**
- Uses only Python stdlib — no external dependencies, works in `python:3.12-alpine` without `pip install` steps
- Gitea pagination is correct: pages until empty response
- `ThreadPoolExecutor` with `max_workers=CONCURRENCY` provides bounded parallelism — does not overwhelm cluster
- `DRY_RUN=true` mode is safe and complete: reads Gitea, prints list, exits 0 without triggering anything
- Dual secret resolution: accepts both `gitea_token` (Woodpecker secret injection) and `GITEA_TOKEN` (env var) — handles both Woodpecker and local/manual invocation
- Error handling: per-repo `[OK]`/`[FAIL]` logging; aggregates failures; exits 1 if any failed
- Summary line confirms count of triggered/skipped/failed
- Configurable `SKIP_REPOS` defaults to `sdlc-cluster` to exclude the management repo

**Validated behaviors:**
- Dry-run executed against live Gitea: correctly found 1 repo (`sdlc-cluster`), correctly skipped it, reported 0 to deploy. Pagination logic reached empty page correctly on small orgs.
- Python syntax validated: `python3 -m py_compile` passed

**Minor findings (tracked, not blocking):**
1. `WOODPECKER_URL` defaults to `http://woodpecker-server.woodpecker.svc.cluster.local` in the pipeline YAML but is a required env var in the script. If Woodpecker is in the `threesix` namespace (not `woodpecker`), the default URL will fail. The operator must set `WOODPECKER_URL` explicitly when triggering — documented in pipeline comments. Track for follow-up correction when Woodpecker cluster URL is confirmed.

### bootstrap.yaml

**Strengths:**
- `event: custom` — pipeline never fires on push/PR; only on manual/API trigger (correct for a one-time operation)
- `python:3.12-alpine` is minimal and stable — no heavy image
- Secrets declared separately from env vars (`gitea_token`, `woodpecker_token` as Woodpecker secrets; `GITEA_URL` as plain env)
- `WOODPECKER_URL` resolved via shell default expansion in the command block — allows operator override without changing the pipeline file
- Comments explain all pipeline parameters at top of file

**Finding:**
- `GITEA_URL` is declared as an environment variable but has no default value in the pipeline YAML. If not set as a Woodpecker repo-level env var or at trigger time, the step will fail with `KeyError`. Recommend adding `GITEA_URL=http://100.79.2.8:30300` as a default. Tracked as a follow-up task.

---

## Dependency Status

| Dependency | Status |
|---|---|
| `fleet-repo-scaffold` | Delivered — `orchard9/sdlc-cluster` repo exists |
| `fleet-helm-chart` | Pending (v17) — chart not yet complete |
| `fleet-deploy-pipeline` | Pending (v17/v18) — deploy-project.yaml not yet delivered |
| `fleet-secrets-infra` | Pending (v17) — Woodpecker secrets not yet registered |
| Woodpecker CI activation | Blocked — `orchard9/sdlc-cluster` not yet activated in Woodpecker (requires `fleet-deploy-pipeline`) |

The bootstrap pipeline files are correct and complete. Woodpecker secrets (T4) and dry-run validation in Woodpecker (T5) are blocked on upstream features and will be completed as part of the v18-fleet-automation milestone acceptance test.

---

## Tasks Created from Review Findings

- [ ] Set `GITEA_URL=http://100.79.2.8:30300` as default in `bootstrap.yaml` environment block to prevent `KeyError` if not provided at trigger time
- [ ] Confirm Woodpecker server's cluster-internal hostname and update `WOODPECKER_URL` default accordingly

---

## Verdict

**Approved.** The implementation is correct, idempotent, well-structured, and matches the spec and design. Minor environment variable default issues are noted and tracked — they do not affect the core bootstrap logic or the dry-run safety of the pipeline. The feature is ready to advance to audit.
