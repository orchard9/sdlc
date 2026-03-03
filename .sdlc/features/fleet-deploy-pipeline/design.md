# Design: fleet-deploy-pipeline

## Overview

A single Woodpecker CI pipeline file (`pipelines/deploy-project.yaml`) that idempotently installs or upgrades one sdlc-server Helm release for a given project slug. The pipeline is triggered on demand (manually or via API) and requires no human intervention after the trigger.

## Pipeline File Location

```
orchard9/sdlc-cluster
└── pipelines/
    └── deploy-project.yaml
```

## Pipeline YAML

```yaml
# pipelines/deploy-project.yaml
# Trigger: manual (Woodpecker UI) or API call
# Required env: SDLC_PROJECT_SLUG, SDLC_REPO
# Optional env: SDLC_BRANCH (default: main)

when:
  event: custom

variables:
  - &helm_image alpine/helm:3.14

steps:
  - name: deploy
    image: *helm_image
    environment:
      SDLC_PROJECT_SLUG:
        from_secret: false
      SDLC_REPO:
        from_secret: false
      SDLC_BRANCH: main
    commands:
      - |
        SLUG="${SDLC_PROJECT_SLUG}"
        REPO="${SDLC_REPO}"
        BRANCH="${SDLC_BRANCH:-main}"
        NAMESPACE="sdlc-${SLUG}"
        RELEASE="sdlc-${SLUG}"

        echo "Deploying project: ${SLUG}"
        echo "Repo: ${REPO}, Branch: ${BRANCH}"
        echo "Namespace: ${NAMESPACE}, Release: ${RELEASE}"

        helm upgrade --install "${RELEASE}" ./helm/sdlc-server \
          --namespace "${NAMESPACE}" \
          --create-namespace \
          --set project.slug="${SLUG}" \
          --set project.repo="${REPO}" \
          --set project.branch="${BRANCH}" \
          --set ingress.host="${SLUG}.sdlc.threesix.ai" \
          --atomic \
          --timeout 5m \
          --wait
```

## Parameter Table

| Variable | Required | Default | Description |
|---|---|---|---|
| `SDLC_PROJECT_SLUG` | Yes | — | Project slug; becomes namespace suffix and Helm release suffix |
| `SDLC_REPO` | Yes | — | Gitea repo path under orchard9 org (e.g. `orchard9/my-project`) |
| `SDLC_BRANCH` | No | `main` | Git branch to deploy |

## Helm Values Passed

| `--set` key | Value | Purpose |
|---|---|---|
| `project.slug` | `$SDLC_PROJECT_SLUG` | Labels, namespace references inside chart |
| `project.repo` | `$SDLC_REPO` | git-sync sidecar clones this repo |
| `project.branch` | `$SDLC_BRANCH` | git-sync branch |
| `ingress.host` | `${SLUG}.sdlc.threesix.ai` | Ingress hostname — composed from slug |

## Idempotency

`helm upgrade --install` is the idempotency primitive:
- First run: installs the release
- Subsequent runs with same values: no-op (Helm detects no change)
- Subsequent runs with different values: upgrades in-place

`--create-namespace` ensures the namespace exists without error if already present.

`--atomic` rolls back automatically on failure — the cluster is never left in a partial state.

## Trigger Modes

1. **Manual (Woodpecker UI):** User opens the `orchard9/sdlc-cluster` repo in Woodpecker, selects this pipeline, sets env vars in the "New Build" dialog, clicks Run.
2. **API trigger:** `POST /api/repos/{owner}/{repo}/pipelines` with `variables` payload — used by scripted bootstrap loops and the reconcile pipeline.
3. **Reconcile pipeline:** The separate `reconcile-projects` pipeline detects new project entries and triggers this pipeline via Woodpecker API for each new slug.

The pipeline does **not** trigger on `push` events — `event: custom` ensures it only runs when explicitly triggered.

## Dependencies

| Dependency | What it provides |
|---|---|
| `fleet-helm-chart` | `./helm/sdlc-server/` chart directory in the repo |
| `fleet-secrets-infra` | ESO ClusterSecretStore + GCP Secret Manager entry the chart's ExternalSecret references |
| `fleet-repo-scaffold` | `orchard9/sdlc-cluster` repo with `pipelines/` directory structure |

## Operational Prerequisites (outside pipeline scope)

- The Woodpecker agent runner has `KUBECONFIG` configured pointing to the k3s cluster
- The runner has network access to the cluster API server
- `alpine/helm:3.14` image is accessible from the runner

## Sequence Diagram

```
User / Reconcile Pipeline
        │
        ▼
  Woodpecker API
  POST /api/repos/orchard9/sdlc-cluster/pipelines
  {variables: {SDLC_PROJECT_SLUG: "foo", SDLC_REPO: "orchard9/foo"}}
        │
        ▼
  Woodpecker Agent (runner)
        │  checks out orchard9/sdlc-cluster
        │  runs: pipelines/deploy-project.yaml
        │
        ▼
  deploy step (alpine/helm:3.14)
        │  helm upgrade --install sdlc-foo ./helm/sdlc-server
        │    --namespace sdlc-foo --create-namespace
        │    --set project.slug=foo ...
        │    --atomic --wait
        │
        ▼
  k3s cluster
        ├── Namespace: sdlc-foo (created if absent)
        ├── Deployment: sdlc-server (Running)
        ├── Service: sdlc-foo
        ├── Ingress: foo.sdlc.threesix.ai
        └── ExternalSecret → GCP Secret Manager
```

## Acceptance Verification

1. File `pipelines/deploy-project.yaml` exists in `orchard9/sdlc-cluster`
2. Woodpecker shows the pipeline for `orchard9/sdlc-cluster`
3. Manual trigger with `SDLC_PROJECT_SLUG=test-project SDLC_REPO=orchard9/test-project` completes green
4. `kubectl get ns sdlc-test-project` → exists
5. `kubectl get deployment -n sdlc-test-project` → Running
6. Second trigger with same parameters → pipeline succeeds (idempotent)
