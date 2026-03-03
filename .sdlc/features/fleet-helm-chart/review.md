# Code Review: Helm chart — sdlc-server + git-sync sidecar

## Summary

The chart is complete and all 10 tasks are implemented. The chart lives at `deployments/helm/sdlc-server/` in the k3s-fleet repo and renders five Kubernetes resources for a single orchard9 project instance. All acceptance criteria from the spec pass.

---

## Files Produced

```
deployments/helm/sdlc-server/
  Chart.yaml
  values.yaml
  templates/
    _helpers.tpl
    namespace.yaml
    external-secret.yaml
    deployment.yaml
    service.yaml
    ingress.yaml
```

---

## Acceptance Criteria Verification

| AC | Criterion | Result |
|---|---|---|
| AC-1 | `helm lint` passes with no errors | PASS — `1 chart(s) linted, 0 chart(s) failed` |
| AC-2 | `helm template` renders all five template files | PASS — Namespace, ExternalSecret, Deployment, Service, Ingress |
| AC-3 | Deployment has exactly two containers: `sdlc-server` and `git-sync` | PASS |
| AC-4 | ExternalSecret references correct secretStore and GSM key | PASS — `gcp-secret-store`, `sdlc-fleet-gitea` |
| AC-5 | Ingress host matches `test.sdlc.threesix.ai` | PASS |
| AC-6 | Namespace is `sdlc-test` | PASS |
| AC-7 | `kubectl apply --dry-run=client` passes without errors | PASS — all 5 resources accepted |

---

## Notable Decisions

**API version correction:** The spec mentioned `external-secrets.io/v1beta1` but the cluster has ESO installed at `v1`. The template uses `v1` — confirmed by inspecting `kubectl api-resources` against the cluster. This is the correct version.

**Deployment strategy — Recreate:** Single-writer constraint for sdlc-server (YAML file store) makes Recreate the safe strategy. RollingUpdate would risk two pods racing to write the same `.sdlc/` files concurrently during a rollout.

**emptyDir shared volume:** The workspace is ephemeral by design — git-sync re-clones on pod start, sdlc-server reads from the live worktree. No PVC needed. This is intentional per the spec's non-goals.

**Credentials scoped to git-sync only:** `GITSYNC_USERNAME` / `GITSYNC_PASSWORD` come from the `gitea-credentials` Secret sourced from ESO. The sdlc-server container has no access to Gitea credentials — correct separation of concerns.

**SSE keep-alive annotation:** `nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"` is set on the Ingress to prevent Nginx from closing SSE connections before the browser does.

---

## Issues Found

**None.** All templates render correctly, all resources pass dry-run validation, and all acceptance criteria are met.

---

## Verdict

APPROVED. Chart is ready for use by the fleet-deploy-pipeline feature.
