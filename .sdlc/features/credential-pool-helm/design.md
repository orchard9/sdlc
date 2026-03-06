# Design: credential-pool-helm

## Overview

This is a Helm/Kubernetes configuration feature. No Rust or frontend code is involved.
The design covers the ExternalSecret + SecretStore pairing, Helm values structure, how
pods consume `DATABASE_URL`, and the rollout strategy.

## Component Diagram

```
GCP Secret Manager
  └── secret: k3sf-postgres-sdlc
        └── field: database_url = "postgresql://appuser:..."

ClusterSecretStore (gcp-secret-manager)
  └── references GCP Secret Manager via workload identity

ExternalSecret (postgres-sdlc-credentials)  [per namespace]
  ├── secretStoreRef: gcp-secret-manager (ClusterSecretStore)
  ├── remoteRef.key:      <values.postgres.externalSecret.gsmKey>
  ├── remoteRef.property: database_url
  └── target.name:        postgres-sdlc-credentials

Kubernetes Secret (postgres-sdlc-credentials)  [synced, per namespace]
  └── key: DATABASE_URL = "postgresql://appuser:..."

sdlc-server Pod
  └── env[DATABASE_URL] ← secretKeyRef: postgres-sdlc-credentials/DATABASE_URL
```

## ExternalSecret + SecretStore Pairing

- **ClusterSecretStore**: `gcp-secret-manager` — pre-installed cluster-wide; uses GKE
  Workload Identity to authenticate to GCP. The ESO controller syncs secrets into
  namespaces that create `ExternalSecret` objects referencing it.
- **ExternalSecret**: rendered per Helm release (per project namespace). References the
  `ClusterSecretStore` so no per-namespace `SecretStore` is needed.
- **refreshInterval: 1h** — secret is re-synced hourly. Pod restarts pick up the latest
  version. Secret rotation does not require a pod restart within the hour window; to force
  immediate refresh, delete the ExternalSecret and let it re-create.

## Helm Values Structure

```yaml
# values.yaml (excerpt)
postgres:
  externalSecret:
    gsmKey: ""   # e.g. "k3sf-postgres-sdlc"; empty = disabled
```

### Why a single `gsmKey` flag

All project pods share the same Postgres instance and the same connection string.
A single GCP secret (`k3sf-postgres-sdlc`) holds the URL. Per-project Helm releases
each create their own `ExternalSecret` in their own namespace, all referencing the
same GCP secret key. This avoids duplicating the connection string and keeps rotation
to a single GCP secret update.

### Alternative considered: `credentialPool.enabled` boolean

A boolean plus a separate `secretRef` field was considered but adds indirection.
`gsmKey: ""` is both the feature flag and the GCP secret name — simpler.

## DATABASE_URL Injection into Pods

In `templates/deployment.yaml`, the `sdlc-server` container env block:

```yaml
{{- if .Values.postgres.externalSecret.gsmKey }}
- name: DATABASE_URL
  valueFrom:
    secretKeyRef:
      name: postgres-sdlc-credentials
      key: DATABASE_URL
{{- end }}
```

The ExternalSecret controller must successfully sync the secret before the pod can start.
If the ExternalSecret fails (e.g. GCP secret missing), the Kubernetes Secret will not be
created, and the pod will crash with `CreateContainerConfigError`. This is intentional —
a misconfigured credential pool is a hard failure, not a silent no-op.

## Rollout Strategy

### Per-project opt-in (selected approach)

Each Helm release sets `postgres.externalSecret.gsmKey` independently. Projects that do
not set the key continue to run with `OptionalCredentialPool::Disabled`. This allows
gradual rollout:

1. Deploy ExternalSecrets operator cluster-wide (pre-existing).
2. Create GCP secret `k3sf-postgres-sdlc` with `database_url` field.
3. For each project release, set `postgres.externalSecret.gsmKey: k3sf-postgres-sdlc`
   in the release values.
4. Verify pod starts and `DATABASE_URL` is accessible.

### Global rollout (future)

Set a cluster-level default value in the FluxCD `HelmRelease` or a `valuesFrom` source
so all releases inherit `gsmKey`. This is out of scope for this feature.

## Files Changed

| File | Change |
|------|--------|
| `k3s-fleet/deployments/helm/sdlc-server/values.yaml` | Add `postgres.externalSecret.gsmKey: ""` with documentation comment |
| `k3s-fleet/deployments/helm/sdlc-server/templates/external-secret-postgres.yaml` | New: conditional ExternalSecret template |
| `k3s-fleet/deployments/helm/sdlc-server/templates/deployment.yaml` | Add conditional `DATABASE_URL` env block |

## Validation Approach

- `helm template` with `gsmKey: ""` — ExternalSecret absent, no DATABASE_URL in deployment
- `helm template` with `gsmKey: "k3sf-postgres-sdlc"` — ExternalSecret present, DATABASE_URL injected
- Dry-run in test namespace with mock `ClusterSecretStore` (or real one in staging cluster)
- Check ExternalSecret `Ready` condition and Kubernetes Secret key presence
