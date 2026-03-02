# Spec: fleet-deploy-pipeline

## Feature

**Woodpecker deploy-project.yaml: on-demand idempotent helm install for one project slug**

## Problem

Provisioning a new sdlc-server instance for an `orchard9/` project requires running a sequence of `kubectl` and `helm` commands manually. There is no standardized, auditable, idempotent way to deploy a single project on demand. Each deployment must be consistent, repeatable, and visible in the Woodpecker CI dashboard.

## Solution

A Woodpecker CI pipeline (`pipelines/deploy-project.yaml`) in the `orchard9/sdlc-cluster` repo that, given two pipeline environment parameters (`SDLC_PROJECT_SLUG` and `SDLC_REPO`), idempotently installs or upgrades a single sdlc-server Helm release for that project.

The pipeline:
1. Uses `helm upgrade --install` — idempotent by design (creates if absent, upgrades if present)
2. Creates the project namespace (`sdlc-<slug>`) if it does not exist
3. Delegates all configuration to the Helm chart values API (slug, repo, branch, Gitea URL, secrets, ingress domain)
4. Requires no human intervention after trigger

## Scope

This feature covers only the Woodpecker pipeline file `pipelines/deploy-project.yaml`. It depends on:
- `fleet-helm-chart` — the `helm/sdlc-server/` chart this pipeline invokes
- `fleet-secrets-infra` — the GCP Secret Manager key and ESO ClusterSecretStore the chart references
- `fleet-repo-scaffold` — the `orchard9/sdlc-cluster` repo must exist with correct structure

This feature does NOT cover:
- The reconcile-projects scheduled pipeline (separate feature)
- Bootstrapping existing projects in bulk (operational concern, not a pipeline file)
- DNS/cert-manager configuration (separate from this pipeline)

## Trigger Mechanism

Woodpecker pipelines in `orchard9/sdlc-cluster` can be triggered:
- Manually from the Woodpecker UI with pipeline environment variable overrides
- Via Woodpecker API (for scripted bootstrap loops)
- By the reconcile pipeline when it detects a new project

The deploy-project pipeline is `event: custom` (manual/API trigger only) — it does not trigger on push.

## Pipeline Parameters

| Variable | Description | Example |
|---|---|---|
| `SDLC_PROJECT_SLUG` | Project slug — becomes namespace suffix and Helm release suffix | `my-project` |
| `SDLC_REPO` | Gitea repo path under orchard9 org | `orchard9/my-project` |
| `SDLC_BRANCH` | Git branch to deploy (default: `main`) | `main` |

## Pipeline Steps

1. **deploy** — `alpine/helm:3.14` image, runs `helm upgrade --install` against `./helm/sdlc-server` with `--set` overrides for slug, repo, branch. Creates namespace with `--create-namespace`.

The pipeline has access to the cluster via kubeconfig. The Woodpecker agent must have `KUBECONFIG` configured pointing to the k3s cluster. This is an operational prerequisite, not a pipeline code concern.

## Idempotency Contract

Running the pipeline N times with the same parameters produces the same deployed state. `helm upgrade --install` guarantees this: if the release does not exist, it installs; if it exists with the same values, it is a no-op (or a fast reapply); if values changed, it upgrades.

## Acceptance Criteria

1. `pipelines/deploy-project.yaml` exists in `orchard9/sdlc-cluster`
2. Triggering the pipeline with `SDLC_PROJECT_SLUG=test-project SDLC_REPO=orchard9/test-project` completes without error
3. After pipeline run: namespace `sdlc-test-project` exists, Deployment is `Running`, Service exists, Ingress exists for `test-project.sdlc.threesix.ai`
4. Running the pipeline a second time with the same parameters succeeds (idempotent)
5. Pipeline is visible and re-triggerable from the Woodpecker UI
