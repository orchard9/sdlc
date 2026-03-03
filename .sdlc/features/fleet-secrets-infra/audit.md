# Security Audit: fleet-secrets-infra

## Scope

This audit covers the GCP Secret Manager entry `sdlc-fleet-gitea` and the `ExternalSecret` manifest that makes Gitea credentials available to the `projects` namespace.

## Findings

### F1: Secret access scope is cluster-wide via ClusterSecretStore (ACCEPTED)

**Observation:** The `gcp-secret-manager` ClusterSecretStore allows any `ExternalSecret` in any namespace to pull any GCP secret in the `orchard9` project. The `sdlc-fleet-gitea-credentials` Secret is created in the `projects` namespace, which narrows the blast radius to that namespace.

**Risk:** Low. The `projects` namespace is dedicated to sdlc-server fleet pods. No untrusted workloads run there. The ClusterSecretStore scope is pre-existing (not introduced by this feature) and applies to all secrets in the cluster.

**Action:** Accepted — pre-existing pattern, no change needed.

### F2: Token is a long-lived personal access token (TRACKED)

**Observation:** The `claude-agent` Gitea token is a long-lived PAT stored in GCP Secret Manager. If compromised, it has full Gitea `claude-agent` privileges including push access to all `orchard9` repos.

**Risk:** Medium. Mitigated by:
- Token lives in GCP Secret Manager (not in git, not in Helm values)
- Access requires valid GCP service account credentials (`eso-secret-reader`)
- The ESO SA key is stored in the `external-secrets` namespace, accessible only to cluster admins

**Action:** Track for future improvement — consider rotating to a narrower-scope token or short-lived secret once the fleet is operational. No immediate remediation required for this feature.

### F3: Secret uses automatic replication (INFO)

**Observation:** The GCP secret uses automatic multi-region replication. This is the standard for the orchard9 project (matches all other secrets).

**Risk:** None. Replication provides availability without material security impact.

**Action:** Accepted.

### F4: `projects` namespace has no NetworkPolicy (INFO)

**Observation:** The `projects` namespace does not have NetworkPolicy restrictions. Any pod in the namespace can access the `sdlc-fleet-gitea-credentials` Secret if it has the right RBAC.

**Risk:** Low. ESO sets `ownerReferences` on the created Secret, and the Secret's data is only accessible to pods that mount it or have explicit `get` RBAC on the Secret resource. This is the standard ESO security model.

**Action:** Accepted — consistent with cluster-wide policy; NetworkPolicy is a future hardening concern across all namespaces.

## Security Posture

This feature follows the established ESO pattern used throughout the cluster. The Gitea token is stored as intended (GCP Secret Manager → ESO → Kubernetes Secret) and does not appear in git, Helm values, or environment variables outside the cluster. No new attack surface is introduced beyond what the existing ClusterSecretStore already provides.

**Audit result: PASS** — no blocking findings. F2 tracked for future token rotation work.
