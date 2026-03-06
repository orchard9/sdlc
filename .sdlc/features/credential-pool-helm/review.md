# Review: credential-pool-helm

## Summary

The Helm chart changes for `credential-pool-helm` consist of three files in
`k3s-fleet/deployments/helm/sdlc-server/`. All changes were reviewed against the spec
and design artifacts.

## Files Reviewed

| File | Status |
|------|--------|
| `values.yaml` | Correct |
| `templates/external-secret-postgres.yaml` | Correct |
| `templates/deployment.yaml` | Correct |

## Findings

### F-1: Conditional guard is consistent
Both `external-secret-postgres.yaml` and the `DATABASE_URL` block in `deployment.yaml`
use identical guards: `{{- if .Values.postgres.externalSecret.gsmKey }}`. This ensures
the ExternalSecret and the env var are always toggled together — no risk of a pod
referencing a missing secret.

**Action:** Accept — no change needed.

### F-2: Secret name consistency
`ExternalSecret.spec.target.name` (`postgres-sdlc-credentials`) matches
`secretKeyRef.name` in the deployment. If they drifted, pods would crash with
`CreateContainerConfigError`.

**Action:** Accept — names are aligned.

### F-3: `Chart.yaml` absent
The `sdlc-server/` directory contains `values.yaml` and `templates/` but no `Chart.yaml`.
`helm template` requires `Chart.yaml` to function. This is either intentional (the chart
is applied via raw manifests or FluxCD's `HelmRelease` with an upstream chart) or an
oversight in the repo scaffold.

**Action:** Track — `sdlc task add credential-pool-helm "Add Chart.yaml to sdlc-server Helm chart directory for local helm template validation"`

This does not block the feature. The templates themselves are syntactically correct Helm.

### F-4: `creationPolicy: Owner`
`ExternalSecret.spec.target.creationPolicy: Owner` means the ESO controller owns the
synced Kubernetes Secret — it will delete the Secret if the ExternalSecret is deleted.
This is the correct policy for managed secrets.

**Action:** Accept.

### F-5: `refreshInterval: 1h`
A 1-hour refresh window means credential rotation takes up to 60 minutes to propagate
without a pod restart. Acceptable for this use case (shared Postgres URL rarely rotates).

**Action:** Accept.

### F-6: `property: database_url` dependency
The GCP secret must contain a JSON-structured value with a `database_url` key. If the
GCP secret is a plain string instead of JSON, the ExternalSecret will fail to sync.
The `values.yaml` comment documents this expectation.

**Action:** Accept — documented.

## QA Validation Note

`helm template` validation (TC-1 through TC-3 in the QA plan) requires `Chart.yaml`.
Without it, the only validation possible is manual inspection of the template files.
Manual inspection confirms:
- Conditional blocks are syntactically valid Helm template syntax
- Secret name references are consistent
- `remoteRef.key` uses `{{ .Values.postgres.externalSecret.gsmKey }}` (parametric, not hardcoded)

## Verdict

The implementation is correct and complete. Finding F-3 (missing `Chart.yaml`) is tracked
as a follow-up task and does not affect production deployments via FluxCD.
