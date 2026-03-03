# QA Results: fleet-deploy-pipeline

**Date:** 2026-03-03  
**Result: PASS (static tests) / DEFERRED (integration tests)**

## TC-1: YAML Syntax Validation — PASS

File `pipelines/deploy-project.yaml` retrieved from Gitea and validated with Python structural checks. All key fields present and correctly formed. No parse errors.

## TC-2: Required Fields Present — PASS

| Field | Expected | Result |
|---|---|---|
| `when.event` | `custom` | PASS |
| Step name | `deploy` | PASS |
| Image | `alpine/helm:3.14` | PASS |
| `SDLC_PROJECT_SLUG` present | yes | PASS |
| `SDLC_REPO` present | yes | PASS |
| `SDLC_BRANCH` present | yes | PASS |
| `helm upgrade --install` | yes | PASS |
| `--create-namespace` | yes | PASS |
| `--atomic` | yes | PASS |
| `--wait` | yes | PASS |

## TC-3: Namespace Construction — PASS

Grep confirmed: `NAMESPACE="sdlc-${SLUG}"` and `RELEASE="sdlc-${SLUG}"` are present in the deploy step. Given `SDLC_PROJECT_SLUG=foo`, namespace and release will be `sdlc-foo`.

## TC-4: Ingress Host Construction — PASS

Grep confirmed: `--set ingress.host="${SLUG}.sdlc.threesix.ai"` is present. Given `SDLC_PROJECT_SLUG=foo`, ingress host will be `foo.sdlc.threesix.ai`.

## TC-5: First-Run Install — DEFERRED

**Reason:** Depends on `fleet-helm-chart` (`./helm/sdlc-server/` chart) being present in `orchard9/sdlc-cluster` and `fleet-secrets-infra` (ESO ClusterSecretStore) being operational. Neither is deployed yet.

**When to re-run:** After `fleet-helm-chart` and `fleet-secrets-infra` are merged. Run as part of `fleet-bootstrap` integration milestone.

## TC-6: Idempotency (Second Run) — DEFERRED

Same dependency as TC-5.

## TC-7: Woodpecker UI Visibility — DEFERRED

**Reason:** The `orchard9/sdlc-cluster` repo requires a `.woodpecker.yml` root file or Woodpecker repository activation (handled by `fleet-repo-scaffold`). Woodpecker activation of the repo is an operational step.

**Status:** The pipeline file is committed. Activation deferred to `fleet-repo-scaffold` delivery.

## TC-8: Missing Required Parameter Behavior — PASS

Grep confirmed: the deploy step contains explicit `exit 1` guards for empty `SDLC_PROJECT_SLUG` and `SDLC_REPO`. Pipeline will fail fast with a clear error message rather than silently deploying a broken release.

## Summary

| TC | Description | Result |
|---|---|---|
| TC-1 | YAML syntax | PASS |
| TC-2 | Required fields | PASS |
| TC-3 | Namespace construction | PASS |
| TC-4 | Ingress host construction | PASS |
| TC-5 | First-run install (integration) | DEFERRED |
| TC-6 | Idempotency re-run (integration) | DEFERRED |
| TC-7 | Woodpecker UI visibility | DEFERRED |
| TC-8 | Missing param error handling | PASS |

**5/5 runnable tests passed. 3 integration tests deferred pending dependent features.**

All static tests pass. Deferred tests are blocked on `fleet-helm-chart` and `fleet-secrets-infra` — not a defect in this feature. Integration tests will be re-run as part of the `fleet-bootstrap` milestone UAT.
