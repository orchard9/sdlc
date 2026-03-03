# Security Audit: fleet-bootstrap

## Scope

Two files in `orchard9/sdlc-cluster`:
- `pipelines/bootstrap.py` — Bootstrap logic script
- `pipelines/bootstrap.yaml` — Woodpecker CI pipeline definition

This audit covers credential handling, injection risks, API surface, and blast-radius analysis.

---

## Findings

### Finding 1: Credentials exposed via environment variables in pipeline YAML (ACCEPTED)

**Severity:** Low
**Description:** `bootstrap.yaml` declares `GITEA_URL` as a plain environment variable. The `gitea_token` and `woodpecker_token` are declared as Woodpecker secrets (injected, not in plaintext). However, `GITEA_URL` is exposed in pipeline logs.

**Assessment:** `GITEA_URL` is non-sensitive — it is the cluster-internal Gitea address and does not contain credentials. Tokens are correctly handled via Woodpecker's secrets mechanism, which masks values in logs. No plaintext credentials in the pipeline file.

**Action:** Accepted. No change required.

---

### Finding 2: Gitea token has full `claude-agent` admin scope (TRACKED)

**Severity:** Medium
**Description:** The `gitea_token` used for Gitea API repo listing is the same `claude-agent` admin token used across the fleet. If this token is leaked from the Woodpecker secrets store, it has full admin access to threesix/gitea.

**Assessment:** The bootstrap pipeline only uses this token for `GET /api/v1/orgs/orchard9/repos` (read-only). A read-only token scoped to org membership listing would be sufficient. However, creating a separate scoped token requires Gitea user/token management that is out of scope for this feature.

**Action:** Tracked. Create a follow-up task: "Create read-only Gitea token for bootstrap pipeline repo listing" to scope down the credential. Until then, existing admin token is acceptable given the internal-only network position of the Gitea instance.

---

### Finding 3: Woodpecker token grants pipeline creation privileges (ACCEPTED)

**Severity:** Medium
**Description:** The `woodpecker_token` used to trigger `deploy-project` pipelines has sufficient privileges to create pipeline runs in `orchard9/sdlc-cluster`. If leaked, it allows triggering arbitrary deploys.

**Assessment:** The token is stored in Woodpecker's encrypted secrets store and injected at runtime — never in YAML or git. The blast radius is limited: an attacker with this token can only trigger pipelines in repos they have access to in Woodpecker. The `deploy-project` pipeline itself uses `helm upgrade --install` which is idempotent — triggering it for an already-deployed repo is a no-op.

**Action:** Accepted. Standard operational risk for CI/CD credential management.

---

### Finding 4: No input validation on SKIP_REPOS or CONCURRENCY (LOW)

**Severity:** Low
**Description:** `SKIP_REPOS` is split by comma and stripped — no validation that values are valid repo names. `CONCURRENCY` is cast directly to `int()` without bounds checking.

**Assessment:** Both variables are pipeline-level parameters set by the operator. Malformed `SKIP_REPOS` would result in repos not being skipped (not a security issue — worst case is an extra `helm upgrade --install` for a repo). Invalid `CONCURRENCY` would raise `ValueError` and fail the pipeline clearly. No injection risk because these values are not interpolated into shell commands.

**Action:** Low severity, no fix required. The pipeline fails safely on bad input.

---

### Finding 5: Script runs in pipeline clone context (pipelines/bootstrap.py path) (INFO)

**Severity:** Informational
**Description:** The pipeline command `python3 pipelines/bootstrap.py` runs the script from the cloned repo. If the `orchard9/sdlc-cluster` repo is compromised, a malicious actor could modify `bootstrap.py` to exfiltrate credentials.

**Assessment:** This is the standard CI/CD trust model. The `orchard9/sdlc-cluster` repo has restricted write access (only `claude-agent` admin and `jordan`). Standard git-based integrity controls apply. This is not a weakness specific to fleet-bootstrap.

**Action:** Informational only. No change required.

---

### Finding 6: No rate limiting on Gitea API calls (LOW)

**Severity:** Low
**Description:** The pagination loop calls `GET /api/v1/orgs/orchard9/repos?page=N` in a tight loop without delays. On orgs with many repos, this could cause elevated load on Gitea.

**Assessment:** With 80–100 repos and a page size of 50, this means 2–3 API calls total. Not a meaningful load concern for the self-hosted Gitea instance.

**Action:** Accepted. No fix required.

---

## Summary

| Finding | Severity | Action |
|---|---|---|
| Credentials in env vars | Low | Accepted — tokens use Woodpecker secrets |
| Admin-scoped Gitea token | Medium | Tracked — follow-up task to scope down |
| Woodpecker token grants CI creation | Medium | Accepted — standard CI/CD model |
| No input validation on SKIP_REPOS/CONCURRENCY | Low | Accepted — fails safely |
| Pipeline runs from cloned repo | Info | Accepted — standard CI/CD trust |
| No rate limiting on Gitea API | Low | Accepted — 2-3 total calls |

**Audit verdict:** Approved. No blocking security issues. One medium finding tracked as a follow-up task to scope down the Gitea token. The implementation follows standard CI/CD security practices appropriate for an internal fleet automation tool.
