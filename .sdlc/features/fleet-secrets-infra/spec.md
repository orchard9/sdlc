# Spec: fleet-secrets-infra

## Summary

Provision the GCP Secret Manager entry `sdlc-fleet-gitea` and create the ESO `ClusterSecretStore` manifest that lets ExternalSecret resources across any `sdlc-<slug>` namespace pull Gitea credentials from GCP without duplicating secrets.

## Background

The `sdlc-cluster` fleet deploys one `sdlc-server` pod per project. Each pod uses a `git-sync` sidecar that must authenticate to the private Gitea instance at `http://100.79.2.8:30300` using the `claude-agent` token. Credentials must not be stored in Helm values or the repo — they live in GCP Secret Manager and are injected at runtime via ExternalSecrets Operator (ESO).

The credentials already exist in sdlc secrets (`sdlc secrets env export gitea`) and were confirmed during the architecture ponder session. This feature makes those credentials available to the Kubernetes cluster.

## Scope

This feature covers:

1. **GCP Secret Manager entry** — A JSON secret named `sdlc-fleet-gitea` containing the Gitea connection fields: `token`, `url`, `user`, `org`.
2. **ESO ClusterSecretStore manifest** — A `ClusterSecretStore` resource (YAML file at `external-secrets/cluster-secret-store.yaml` in the `orchard9/sdlc-cluster` repo) referencing the GCP project and the Workload Identity / service account used by the existing ESO installation.
3. **Validation** — The `ClusterSecretStore` must report `Ready = True` and a test `ExternalSecret` in a scratch namespace must resolve the secret successfully.

This feature does NOT cover:
- The per-project `ExternalSecret` template (that lives in `helm/sdlc-server/templates/external-secret.yaml`, covered by `fleet-helm-chart`)
- The Woodpecker pipelines (covered by `fleet-deploy-pipeline`)
- TLS/ingress setup (covered by `fleet-ingress-tls`)

## Secret Schema

The GCP Secret Manager secret named `sdlc-fleet-gitea` must be JSON with the following fields:

```json
{
  "token": "<claude-agent Gitea token>",
  "url":   "http://100.79.2.8:30300",
  "user":  "claude-agent",
  "org":   "orchard9"
}
```

All four fields must be present. The `token` value comes from the sdlc secrets store (`sdlc secrets env export gitea`).

## ClusterSecretStore

The `ClusterSecretStore` must:
- Be named `gcp-secret-store` (matches the reference already defined in the architecture doc)
- Use `gcpsm` provider (GCP Secret Manager)
- Reference the same GCP project ID as the existing ESO installation in the cluster
- Use Workload Identity for authentication (no static GCP service account key in the cluster)

The file lives at:
```
orchard9/sdlc-cluster/external-secrets/cluster-secret-store.yaml
```

## Acceptance Criteria

1. `gcloud secrets list --filter="name:sdlc-fleet-gitea"` returns the secret in the correct GCP project.
2. `gcloud secrets versions access latest --secret=sdlc-fleet-gitea | jq .` returns valid JSON with all four fields.
3. `kubectl get clustersecretstore gcp-secret-store -o jsonpath='{.status.conditions[0].type}'` returns `Ready`.
4. A test `ExternalSecret` in namespace `sdlc-test` that references `sdlc-fleet-gitea` property `token` results in a k8s Secret with the correct token value.
5. The `cluster-secret-store.yaml` file is committed to `orchard9/sdlc-cluster` on threesix/gitea.

## Dependencies

- ExternalSecrets Operator (ESO) must be installed in the cluster (pre-existing — already used by other workloads in the k3s-fleet).
- The GCP project must have the Secret Manager API enabled.
- The ESO service account must have the `roles/secretmanager.secretAccessor` IAM binding on the GCP project or specifically on `sdlc-fleet-gitea`.
- Gitea token value: retrieved from `sdlc secrets env export gitea` at implementation time.

## Implementation Notes

- **GCP project ID**: Determine at implementation time by inspecting the existing ESO `ClusterSecretStore` or `SecretStore` in the cluster (`kubectl get clustersecretstore -A -o yaml`).
- **Workload Identity annotation**: The ESO service account Kubernetes object in the `external-secrets` namespace will have the Workload Identity annotation (`iam.gke.io/gcp-service-account`). Use the same SA for `sdlc-fleet-gitea`.
- **Secret format**: ESO's `gcpsm` provider supports `property` selectors on JSON secrets (the `property` field in the `remoteRef` block extracts a single key from JSON). This is the access pattern used by the per-project `ExternalSecret` in the Helm chart.
- **Idempotency**: The `gcloud secrets create` command is not idempotent — use `gcloud secrets create || gcloud secrets versions add` pattern or check existence first.
