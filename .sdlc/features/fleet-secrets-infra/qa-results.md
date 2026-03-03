# QA Results: fleet-secrets-infra

**Run date:** 2026-03-03
**Environment:** Live k3s cluster (orchard9-k3sf), GCP project orchard9

## Results

| Test Case | Status | Notes |
|---|---|---|
| TC-1: GCP Secret keys | PASS | `["org", "token", "url", "user"]` — all four present |
| TC-2: GCP Secret non-token values | PASS | url, user, org match expected values |
| TC-3: ExternalSecret Ready | PASS | `status: True`, `reason: SecretSynced` |
| TC-4: Kubernetes Secret keys | PASS | `['GITEA_ORG', 'GITEA_TOKEN', 'GITEA_URL', 'GITEA_USER']` |
| TC-5: Token value correct | PASS | Decoded GITEA_TOKEN matches sdlc secrets value |
| TC-6: Commit in k3s-fleet | PASS | Commit `4a3228e` — `sdlc-fleet-gitea-external-secret.yaml` |

## Summary

All 6 test cases pass. The `sdlc-fleet-gitea-credentials` Kubernetes Secret is live in the `projects` namespace and contains the correct Gitea credentials extracted from GCP Secret Manager via ESO. The implementation is committed to the k3s-fleet repo.

**QA: PASS**
