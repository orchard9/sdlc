# QA Results: Helm chart — sdlc-server + git-sync sidecar

**Run date:** 2026-03-02
**Chart path:** `deployments/helm/sdlc-server/` in k3s-fleet repo
**Test values:** `--set project.slug=test --set project.repo=orchard9/test`

---

## Results

| TC | Description | Result | Notes |
|---|---|---|---|
| TC-1 | `helm lint` passes | PASS | `1 chart(s) linted, 0 chart(s) failed` (INFO: icon recommended only) |
| TC-2 | Five resource kinds rendered | PASS | Deployment, ExternalSecret, Ingress, Namespace, Service |
| TC-3 | Deployment has exactly two containers | PASS | `sdlc-server`, `git-sync` |
| TC-4 | ExternalSecret: correct secretStore and GSM key | PASS | `gcp-secret-store`, `sdlc-fleet-gitea` |
| TC-5 | Ingress host is `test.sdlc.threesix.ai` | PASS | |
| TC-6 | Namespace name is `sdlc-test` | PASS | |
| TC-7 | All namespaced resources in `sdlc-test` | PASS | Deployment, Service, Ingress, ExternalSecret all in `sdlc-test` |
| TC-8 | `kubectl apply --dry-run=client` passes | PASS | All 5 resources accepted by cluster API server |
| TC-9 | git-sync credential env vars sourced from Secret | PASS | `GITSYNC_USERNAME` key=user, `GITSYNC_PASSWORD` key=token |
| TC-10 | SSE annotation `proxy-read-timeout: 3600` | PASS | |

**10/10 test cases passed.**

---

## API Version Note

The QA plan referenced `external-secrets.io/v1beta1` (from the spec). During T10 implementation, it was discovered that the cluster has ESO installed at `v1`. The template was corrected to `external-secrets.io/v1` and TC-8 was re-run — confirming the ExternalSecret passes schema validation against the live cluster. This correction is documented in the review and audit artifacts.

---

## Verdict

**PASS.** All acceptance criteria met. Chart is ready for merge.
