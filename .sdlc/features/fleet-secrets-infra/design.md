# Design: fleet-secrets-infra

## Overview

Provision the GCP Secret Manager entry `sdlc-fleet-gitea` and add an `ExternalSecret` manifest in the k3s-fleet repo so that sdlc-server pods across any `sdlc-<slug>` namespace can pull Gitea credentials without duplicating secrets.

## Key Discovery: Existing ClusterSecretStore

The cluster already has a working `ClusterSecretStore` named **`gcp-secret-manager`** (not `gcp-secret-store` as the spec placeholder stated). It is `Ready = True` and uses a GCP service account key stored in `external-secrets/gcp-secret-manager` (key: `secret-access-credentials`). The GCP project is `orchard9`.

This is the store that all other `ExternalSecret` resources in the cluster reference. This feature reuses it — no new `ClusterSecretStore` is needed.

## Architecture

```
GCP Secret Manager (project: orchard9)
  └── sdlc-fleet-gitea  (JSON: token, url, user, org)
           │
           │  ESO ClusterSecretStore: gcp-secret-manager
           │  (already exists, Ready=True)
           ▼
k8s ExternalSecret (namespace: projects — the sdlc fleet namespace)
  └── k8s Secret: sdlc-fleet-gitea-credentials
        ├── GITEA_TOKEN   ← from property "token"
        ├── GITEA_URL     ← from property "url"
        ├── GITEA_USER    ← from property "user"
        └── GITEA_ORG     ← from property "org"
```

## Components

### 1. GCP Secret Manager Entry

**Secret name:** `sdlc-fleet-gitea`
**Project:** `orchard9`
**Format:** JSON

```json
{
  "token": "<claude-agent Gitea personal access token>",
  "url":   "http://100.79.2.8:30300",
  "user":  "claude-agent",
  "org":   "orchard9"
}
```

Created with `gcloud secrets create` (idempotent: check existence first, use `versions add` if already exists).

### 2. ExternalSecret Manifest

**File location:** `deployments/k8s/base/external-secrets/sdlc-fleet-gitea-external-secret.yaml` in the k3s-fleet repo.

**Namespace:** `projects` — the namespace where sdlc-server fleet pods run.

**Name convention:** `sdlc-fleet-gitea-credentials` — matches the Kubernetes Secret name that sdlc-server pods will mount.

The manifest uses the existing `gcp-secret-manager` ClusterSecretStore and extracts all four JSON properties into a single Kubernetes Secret.

### 3. Kustomization Update

Add the new file to `deployments/k8s/base/external-secrets/kustomization.yaml` in the `resources` list (under `# projects namespace` comment group).

## Auth Pattern

The existing `ClusterSecretStore` uses a GCP service account key (`eso-secret-reader@orchard9.iam.gserviceaccount.com`) with `roles/secretmanager.secretAccessor`. Creating the new secret in the same GCP project under the same SA means no IAM changes are needed — the ESO reader can already access all secrets in the `orchard9` project.

## Validation Steps

1. `gcloud secrets list --filter="name:sdlc-fleet-gitea" --project=orchard9` → lists the secret
2. `gcloud secrets versions access latest --secret=sdlc-fleet-gitea --project=orchard9 | jq .` → shows all four fields
3. After applying the kustomize manifest: `kubectl get externalsecret sdlc-fleet-gitea-credentials -n projects` → `READY=True, SYNCED=True`
4. `kubectl get secret sdlc-fleet-gitea-credentials -n projects -o jsonpath='{.data.GITEA_TOKEN}' | base64 -d` → correct token value

## Out of Scope

- The per-project `ExternalSecret` template inside the Helm chart (covered by `fleet-helm-chart`)
- Creating the `projects` namespace (pre-existing in the cluster)
- Woodpecker pipelines (`fleet-deploy-pipeline`)
- TLS/ingress (`fleet-ingress-tls`)

## File Paths

| File | Repo | Action |
|---|---|---|
| `deployments/k8s/base/external-secrets/sdlc-fleet-gitea-external-secret.yaml` | k3s-fleet | Create |
| `deployments/k8s/base/external-secrets/kustomization.yaml` | k3s-fleet | Update (add resource) |
