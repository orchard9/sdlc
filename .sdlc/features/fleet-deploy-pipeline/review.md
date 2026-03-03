# Review: fleet-deploy-pipeline

## Summary

`pipelines/deploy-project.yaml` has been created and committed to `orchard9/sdlc-cluster`. The file is live at:

`https://git.threesix.ai/orchard9/sdlc-cluster/src/branch/main/pipelines/deploy-project.yaml`

## Implementation Checklist

- [x] T1: `pipelines/` directory created in `orchard9/sdlc-cluster`
- [x] T2: `pipelines/deploy-project.yaml` authored per design
- [x] T3: YAML structure validated (12 structural checks, all pass)
- [x] T4: File committed and pushed to `main` branch (commit `86399af4`)
- [ ] T5: Woodpecker UI visibility — pending Woodpecker activation of the repo
- [ ] T6: Manual trigger smoke test — deferred (depends on `fleet-helm-chart` chart being in repo)
- [ ] T7: Idempotency re-run test — deferred (same dependency)
- [ ] T8: Test namespace cleanup — N/A until T6 runs

## Structural Validation Results (TC-1 through TC-4)

| Check | Result |
|---|---|
| `when.event: custom` trigger | PASS |
| `deploy` step exists | PASS |
| Image: `alpine/helm:3.14` | PASS |
| `SDLC_PROJECT_SLUG` parameter present | PASS |
| `SDLC_REPO` parameter present | PASS |
| `helm upgrade --install` command | PASS |
| `--create-namespace` flag | PASS |
| `--atomic` flag | PASS |
| `--wait` flag | PASS |
| Ingress host: `${SLUG}.sdlc.threesix.ai` | PASS |
| Namespace: `sdlc-${SLUG}` | PASS |
| Release name: `sdlc-${SLUG}` | PASS |

## Code Quality Findings

**No blockers or majors found.**

**Observation — guard clauses for missing parameters:** The implementation includes explicit `exit 1` guards for empty `SDLC_PROJECT_SLUG` and `SDLC_REPO` (TC-8 requirement). This is an improvement over the minimal design spec and adds safety.

**Observation — `SDLC_BRANCH` default:** The `SDLC_BRANCH` environment variable in the pipeline declaration defaults to `main` at the pipeline level, and the shell uses `${SDLC_BRANCH:-main}` as an additional safety net. Double defaulting is safe and correct.

## Deferred Integration Tests

TC-5, TC-6, TC-7 require:
1. `fleet-helm-chart` — `./helm/sdlc-server/` chart in the repo
2. `fleet-secrets-infra` — ESO ClusterSecretStore operational
3. Woodpecker agent configured with cluster kubeconfig

These tests are tracked in the QA plan and will be executed as part of the `fleet-bootstrap` integration milestone once all dependent features are implemented. This is by design per the spec's dependency section.

## Conclusion

The pipeline file is correctly implemented, syntactically valid, and committed. Structural tests pass. Integration tests are intentionally deferred pending dependent feature completion. The implementation is ready to advance to audit.
