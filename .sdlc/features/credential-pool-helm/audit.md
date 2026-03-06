# Security Audit: credential-pool-helm

## Scope

Helm chart changes: `values.yaml`, `templates/external-secret-postgres.yaml`,
`templates/deployment.yaml`, `Chart.yaml`. No Rust code, no API endpoints, no
frontend changes.

## Findings

### SA-1: Secret value never stored in Helm values or chart
**Severity:** N/A (positive confirmation)

`values.yaml` stores only the GCP Secret Manager *key name* (a string like
`k3sf-postgres-sdlc`), not the actual database password or connection string. The
actual secret value lives in GCP Secret Manager and is fetched by the ESO controller
into a Kubernetes `Secret` at runtime. This is the correct pattern — no secret value
is ever embedded in the Helm chart or its values.

**Action:** Accept.

### SA-2: `creationPolicy: Owner` correctly prevents orphaned secrets
**Severity:** Low (positive confirmation)

Setting `creationPolicy: Owner` means the Kubernetes Secret is owned by the
ExternalSecret. If the Helm release is deleted (ExternalSecret deleted), the synced
Secret is garbage-collected automatically. This prevents orphaned Kubernetes Secrets
containing database credentials from persisting after a release is removed.

**Action:** Accept.

### SA-3: `DATABASE_URL` injected as env var (not mounted file)
**Severity:** Low

Environment variables are readable by any process in the container and may appear in
crash dumps or `kubectl describe pod` output. A volume-mounted file would be slightly
more restrictive. For a shared Postgres connection string (not a per-user token), env
var injection is the conventional Kubernetes pattern and is acceptable.

**Action:** Accept — consistent with existing `ANTHROPIC_API_KEY` injection pattern in
the same deployment template.

### SA-4: `refreshInterval: 1h` — credential rotation window
**Severity:** Low

In the event of credential compromise, the ESO controller will re-sync within 1 hour.
Running pods hold the old value in their environment until restarted. A shorter refresh
interval (e.g. 15m) would reduce the window but increases GCP API calls. Since credential
rotation for a shared Postgres URL is expected to be rare and planned, 1h is acceptable.

**Action:** Accept. If rotation SLA tightens, reduce `refreshInterval` in `values.yaml`.

### SA-5: No per-project credential isolation
**Severity:** Medium (accepted risk, by design)

All project pods in the cluster share the same GCP secret key and therefore the same
Postgres connection string. A compromised pod in one project namespace can connect to
the shared Postgres as the same user as all other projects. This is an accepted design
constraint for the initial credential pool — per-project isolation is explicitly listed
as a non-goal in the spec.

**Action:** Track as future hardening — create a follow-up feature for per-project
database user isolation when the threat model requires it.

### SA-6: `ClusterSecretStore` scope
**Severity:** Low

Using `ClusterSecretStore` (cluster-scoped) rather than a namespace-scoped `SecretStore`
means any namespace can create an `ExternalSecret` referencing `gcp-secret-manager`.
This is pre-existing cluster configuration, not introduced by this feature. Since the
Helm chart controls which namespaces create ExternalSecrets, this is acceptable.

**Action:** Accept — not introduced by this feature.

## Summary

No blocking security findings. SA-5 (shared credential) is a known, accepted design
constraint documented in the spec. All other findings are confirmations of correct
practice or low-severity accepted risks.

| Finding | Severity | Action |
|---------|----------|--------|
| SA-1: No secret in values | N/A | Accept |
| SA-2: Owner policy prevents orphan | Low | Accept |
| SA-3: Env var injection | Low | Accept |
| SA-4: 1h refresh window | Low | Accept |
| SA-5: Shared credentials | Medium | Accept (by design) |
| SA-6: ClusterSecretStore | Low | Accept |
