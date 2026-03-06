# Spec: credential-pool-helm

## Summary

Add ExternalSecret and DATABASE_URL injection to the `sdlc-server` Helm chart so that each
project pod deployed by the fleet can connect to the shared PostgreSQL credential pool.
When `postgres.externalSecret.gsmKey` is set, the chart creates an `ExternalSecret` resource
that fetches the connection string from GCP Secret Manager and makes it available to the
`sdlc-server` container as `DATABASE_URL`.

## Problem

The `credential-pool-core` feature adds a PostgreSQL-backed credential pool to `sdlc-server`,
but the server can only connect if `DATABASE_URL` is present in its environment. Kubernetes
pods must not embed secrets in their specs — secrets must be sourced from a secret store.
This feature wires the secret injection so every project pod picks up the credential pool
connection string automatically.

## Scope

- `ExternalSecret` CRD (`external-secrets.io/v1beta1`) that pulls `DATABASE_URL` from GCP
  Secret Manager via a `ClusterSecretStore` named `gcp-secret-manager`
- Helm value `postgres.externalSecret.gsmKey` — when non-empty, enables the ExternalSecret
  and the corresponding `DATABASE_URL` env var in the `sdlc-server` container
- Deployment template: conditionally inject `DATABASE_URL` from the created Kubernetes Secret
- `values.yaml` documentation: describe expected GCP Secret Manager secret format

## ExternalSecret CRD

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: postgres-sdlc-credentials
  namespace: <release-namespace>
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: gcp-secret-manager
    kind: ClusterSecretStore
  target:
    name: postgres-sdlc-credentials
    creationPolicy: Owner
  data:
    - secretKey: DATABASE_URL
      remoteRef:
        key: <postgres.externalSecret.gsmKey>
        property: database_url
```

The ExternalSecret is rendered only when `postgres.externalSecret.gsmKey` is non-empty
(`{{- if .Values.postgres.externalSecret.gsmKey }}`).

## GCP Secret Manager secret format

The GCP secret referenced by `gsmKey` must contain a JSON object with at least:

```json
{
  "database_url": "postgresql://appuser:<password>@postgres.databases.svc.cluster.local:5432/appdb?sslmode=disable"
}
```

The `database_url` field is extracted by the `property: database_url` remoteRef and stored
as the `DATABASE_URL` key in the Kubernetes `Secret` named `postgres-sdlc-credentials`.

## Helm Value Injection

In `values.yaml`:

```yaml
postgres:
  externalSecret:
    # GCP Secret Manager key for the shared Postgres connection string.
    # Secret must have a field named `database_url`.
    # Leave empty to disable credential pool (agents run with ambient auth).
    gsmKey: ""
```

In `templates/deployment.yaml`, inside the `sdlc-server` container env block:

```yaml
{{- if .Values.postgres.externalSecret.gsmKey }}
- name: DATABASE_URL
  valueFrom:
    secretKeyRef:
      name: postgres-sdlc-credentials
      key: DATABASE_URL
{{- end }}
```

## Opt-in Behavior

- `gsmKey: ""` (default) — no ExternalSecret is created, no `DATABASE_URL` is injected.
  The server boots with `OptionalCredentialPool::Disabled` (from `credential-pool-core`).
- `gsmKey: "k3sf-postgres-sdlc"` — ExternalSecret syncs, `DATABASE_URL` env var is set,
  credential pool connects at startup.

This is per-release-instance opt-in: each Helm release (one per project) sets its own
`gsmKey`. All project pods share the same Postgres instance but each has its own release.

## Non-Goals

- Per-project database isolation (all projects share `appdb`)
- Token-level secret isolation
- Any Rust or server-side changes (those are in `credential-pool-core`)
- ExternalSecret operator installation (assumed pre-installed in the cluster)
