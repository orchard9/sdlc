# Audit: fleet-reconcile-pipeline

## Security Surface

This feature introduces:
- `scripts/reconcile.sh` — a shell script that calls external APIs (Gitea, Woodpecker) and runs `helm list`
- `pipelines/reconcile-projects.yaml` — a Woodpecker CI pipeline that executes the script on a schedule

Both run in the Woodpecker agent environment with access to cluster secrets.

---

## Findings

### A1: Secret exposure via environment variable injection (LOW — mitigated by design)

**Observation:** Secrets (`GITEA_TOKEN`, `WOODPECKER_TOKEN`) are passed as environment variables to the pipeline step. Environment variables are visible to child processes and may appear in verbose shell output.

**Assessment:** This is the standard Woodpecker secret injection pattern — the same pattern used by `deploy-project.yaml`. Secrets are stored in Woodpecker's encrypted secret store and injected at runtime. They are not embedded in the pipeline YAML or script source. Woodpecker masks secret values in log output by default.

**Risk:** Low. The token is only visible to processes running in the same container step. Woodpecker log masking prevents accidental console exposure.

**Action:** Accept as-is. Document that Woodpecker log masking must be enabled (it is by default). No code change needed.

---

### A2: Gitea API token has admin scope (LOW — necessary)

**Observation:** The `gitea_token` secret is the `claude-agent` admin token, which has full Gitea admin access. The reconcile script only uses it to list org repos (`GET /api/v1/orgs/orchard9/repos`).

**Assessment:** The token is over-scoped for the read-only purpose of listing repos. A read-only token with `read:org` scope would be sufficient.

**Risk:** Low in the current single-tenant context. If the token were compromised, an attacker could modify any Gitea repo.

**Action:** Track as future hardening. Create a dedicated read-only Gitea token with `read:org` scope for reconcile use. This is a separate operational hardening task, not a blocker.

---

### A3: Script injects repo name into shell (LOW — no user input)

**Observation:** `slug` and `default_branch` values from the Gitea API response are used in shell commands and as curl JSON payload values.

**Assessment:** Repo names and branch names come from the Gitea API (not from user input to this pipeline). However, a maliciously named repo in the org (e.g., a name containing shell metacharacters or JSON injection characters) could potentially cause issues.

**Shell injection:** The while loop reads `slug` and `default_branch` with `IFS=' '`. Repo names in Gitea are restricted to alphanumeric, hyphen, underscore, and dot characters by Gitea's own validation. A name like `foo; rm -rf /` is not a valid Gitea repo name and cannot be created.

**JSON injection:** The curl `-d` body uses string interpolation for `slug` and `default_branch`. If a repo name contained a `"` character, it could break the JSON body. Gitea disallows `"` in repo names.

**Risk:** Low. Gitea's own input validation prevents the malicious names that would be needed to exploit this.

**Action:** Accept as-is. Document that the script assumes Gitea input validation is in place. For defense-in-depth, a future hardening task could use `jq` to construct the JSON body instead of string interpolation.

---

### A4: No rate limiting / back-pressure on bulk trigger (INFO)

**Observation:** If many repos are undeployed (e.g., first run on a large org), the script fires one Woodpecker API request per undeployed repo in a tight loop. This could create a burst of pipeline queue entries.

**Assessment:** Woodpecker's pipeline queue handles concurrent runs, and each `deploy-project.yaml` run is short. For 80+ repos, this is manageable. Not a security issue — an availability / load concern.

**Risk:** Informational. For very large orgs, this could saturate the Woodpecker agent pool briefly.

**Action:** Accept as-is. The initial bulk provisioning is the `fleet-bootstrap` feature (separate). By design, reconcile handles incremental additions (small deltas). If org scale grows significantly, a sleep or batch size could be added then.

---

### A5: Temp file race condition (LOW — low-probability)

**Observation:** Temp files use `$$` (shell PID) suffix. On a multi-user system, PIDs can be reused after a process exits.

**Assessment:** The Woodpecker container runs as a single process in an isolated container. PID reuse within a container during a single script execution is not possible. This pattern is safe in the CI context.

**Risk:** Negligible in CI container context.

**Action:** Accept as-is.

---

## Overall Security Assessment

The pipeline has no high or critical findings. Three low findings are accepted with documented rationale; two informational items are noted for future hardening. The implementation follows the same security posture as the existing `deploy-project.yaml` pipeline in the repo.

**Verdict: PASS**

The feature is ready to advance to QA.
