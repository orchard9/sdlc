# Tasks: fleet-secrets-infra

## T1: Create GCP Secret Manager entry `sdlc-fleet-gitea`

Read Gitea credentials from `sdlc secrets env export gitea`. Check if `sdlc-fleet-gitea` already exists in GCP project `orchard9`. If not, create it with `gcloud secrets create --replication-policy=automatic`. Add a version with JSON payload containing `token`, `url`, `user`, `org`. If it already exists, use `gcloud secrets versions add` to push the latest value.

Verify with: `gcloud secrets versions access latest --secret=sdlc-fleet-gitea --project=orchard9 | jq .`

## T2: Create ExternalSecret manifest in k3s-fleet

Create `deployments/k8s/base/external-secrets/sdlc-fleet-gitea-external-secret.yaml` in `/Users/jordanwashburn/Workspace/orchard9/k3s-fleet`.

The manifest creates a Kubernetes Secret `sdlc-fleet-gitea-credentials` in the `projects` namespace, pulling all four JSON properties (`token`, `url`, `user`, `org`) from the GCP secret as environment-style keys (`GITEA_TOKEN`, `GITEA_URL`, `GITEA_USER`, `GITEA_ORG`). References the existing `gcp-secret-manager` ClusterSecretStore.

## T3: Update kustomization.yaml to include the new ExternalSecret

Add `sdlc-fleet-gitea-external-secret.yaml` to the `resources` list in `deployments/k8s/base/external-secrets/kustomization.yaml`. Place it in a `# projects namespace` comment group.

## T4: Apply and validate in the cluster

Apply the kustomize base with `kubectl apply -k deployments/k8s/base/external-secrets/` from the k3s-fleet directory (or via the existing `make` target if one exists). Verify:
- `kubectl get externalsecret sdlc-fleet-gitea-credentials -n projects` → `READY=True, SYNCED=True`
- `kubectl get secret sdlc-fleet-gitea-credentials -n projects -o jsonpath='{.data.GITEA_TOKEN}' | base64 -d` → correct token value

## T5: Commit changes to k3s-fleet repo

Commit the new `sdlc-fleet-gitea-external-secret.yaml` and updated `kustomization.yaml` to the k3s-fleet repo with a descriptive commit message.
