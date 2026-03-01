# Security Audit: uat-run-persistence

## Scope

New types `UatVerdict` and `UatRun`, three path helpers, three free functions in `sdlc-core`. No HTTP routes, no CLI commands, no user-facing inputs in this feature.

## Attack Surface

**File system writes:** `save_uat_run` writes a YAML file under `.sdlc/milestones/<slug>/uat-runs/<id>/run.yaml`. The `slug` and `id` fields are both provided by the caller (the `sdlc-milestone-uat` skill). Neither value is validated by this function.

**Path traversal risk:** Low. `id` is passed directly to `paths::uat_run_dir` and then to `paths::uat_run_manifest`, which concatenates it as a path segment. A malicious `id` containing `..` could write outside the intended directory. Mitigation: The `id` format `YYYYMMDD-HHMMSS-<hex>` is enforced by convention in the skill, not enforced here. For the current use case (agent-generated IDs) this is acceptable — the same pattern is used for `milestone_slug` in `Milestone::load`. Future hardening could add a slug-style validator for `run_id`.

**YAML deserialization:** `serde_yaml::from_str` is used for deserialization. `serde_yaml` does not execute code and does not support YAML merge keys in ways that cause injection. No arbitrary types — structs have fully typed fields. Risk: negligible.

**Sensitive data:** `UatRun` stores test counts, file paths, and task slugs. No secrets, credentials, or PII.

**Denial of service:** `list_uat_runs` reads all entries in the `uat-runs/` directory. A directory with very many entries could be slow, but no production scenario produces more than a handful of UAT runs per milestone.

## Findings

| Severity | Finding | Status |
|---|---|---|
| Low | `run_id` is not slug-validated — a crafted `id` with `..` could path-traverse | Accepted (same posture as other milestone slugs; skill controls ID format) |

## Verdict

No blocking security issues. The feature is safe to ship. The low-severity path-traversal note is tracked as a future hardening task if needed.
