# Security Audit: fleet-deploy-pipeline

## Scope

`pipelines/deploy-project.yaml` in `orchard9/sdlc-cluster` — a Woodpecker CI pipeline that runs `helm upgrade --install` against the k3s cluster.

## Findings

### F-1: Pipeline has cluster-wide write access via kubeconfig (ACCEPTED)

**Severity:** Medium (design constraint)

The Woodpecker agent runner uses a pre-configured `KUBECONFIG` that grants cluster access. The pipeline inherits this access and can create namespaces, deploy workloads, and install Helm releases cluster-wide.

**Risk:** A compromised Woodpecker agent or malicious pipeline trigger could deploy arbitrary workloads.

**Mitigation already in place:**
- Pipeline is `event: custom` only — no push-triggered execution
- `orchard9/sdlc-cluster` repo access controls who can trigger it
- Woodpecker API access requires authentication

**Action:** ACCEPTED — this is the intended operational model. The runner's kubeconfig should be scoped to the `sdlc-*` namespace prefix using RBAC (operational concern, tracked below).

**Task:** Add RBAC scoping for the Woodpecker runner kubeconfig as an operational task in `fleet-secrets-infra` or a separate hardening feature.

---

### F-2: Input variables passed directly to `helm --set` (ACCEPTED)

**Severity:** Low

`SDLC_PROJECT_SLUG`, `SDLC_REPO`, and `SDLC_BRANCH` are passed directly as `--set` values to helm without sanitization beyond emptiness checks.

**Risk:** A malformed slug (e.g. containing `../` or special chars) could produce unexpected namespace/release names. However, helm validates release names and Kubernetes validates namespace names — both will reject invalid characters at the API level, failing the pipeline before any partial state is created. `--atomic` ensures rollback on any failure.

**Action:** ACCEPTED — Kubernetes and Helm enforce naming constraints. The `--atomic` flag prevents partial deployments. No additional sanitization required at the pipeline level.

---

### F-3: No image digest pinning (TRACKED)

**Severity:** Low

The pipeline uses `alpine/helm:3.14` by tag rather than by digest (e.g. `alpine/helm:3.14@sha256:...`). Tag mutability means a compromised or updated image could change behavior silently.

**Action:** TRACKED — `sdlc task add fleet-deploy-pipeline "Pin alpine/helm image to digest for supply chain integrity"`. Acceptable for initial delivery; should be addressed before production hardening.

---

### F-4: Parameter values echoed to pipeline log (ACCEPTED)

**Severity:** Informational

`SDLC_PROJECT_SLUG`, `SDLC_REPO`, and `SDLC_BRANCH` are echoed to stdout via `echo` statements. These are not secrets — they are project identifiers. No credentials or tokens are logged.

**Action:** ACCEPTED — no sensitive data is logged.

---

### F-5: No validation that SDLC_REPO belongs to orchard9 org (TRACKED)

**Severity:** Low

The pipeline accepts any `SDLC_REPO` value and passes it to the Helm chart's `project.repo` field (used by git-sync). A trigger with `SDLC_REPO=attacker/repo` would configure the deployed sdlc-server to sync from an external repo.

**Risk:** Only users with Woodpecker API access or UI access can trigger the pipeline. Access control is the primary defense. An additional guard could validate `orchard9/` prefix.

**Action:** TRACKED — `sdlc task add fleet-deploy-pipeline "Add org prefix validation: assert SDLC_REPO starts with orchard9/"`. Low urgency given access controls.

---

## Summary

| Finding | Severity | Disposition |
|---|---|---|
| F-1: Cluster-wide kubeconfig access | Medium | Accepted (design); RBAC scoping tracked in infra |
| F-2: `--set` with unvalidated slug | Low | Accepted (Kubernetes/Helm enforce naming) |
| F-3: Image tag not digest-pinned | Low | Tracked as task |
| F-4: Params echoed to log | Info | Accepted (non-sensitive) |
| F-5: No org prefix validation on SDLC_REPO | Low | Tracked as task |

No blockers. Two low-severity items tracked as follow-on tasks. The pipeline is safe to ship for the initial fleet deployment use case.
