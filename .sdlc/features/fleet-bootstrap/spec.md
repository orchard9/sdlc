# Spec: fleet-bootstrap

## Feature

**Slug:** fleet-bootstrap
**Milestone:** v18-fleet-automation
**Title:** Provision initial orchard9 projects by triggering reconcile pipeline for all existing repos

## Problem

The `orchard9` Gitea org contains 80+ existing repositories. After the `fleet-reconcile-pipeline` and `fleet-deploy-pipeline` features are delivered, a scheduled Woodpecker reconcile pipeline will handle new repos automatically. However, the existing repos at launch time will not have sdlc-server instances deployed yet — they predate the reconcile system and will never trigger a "new repo" event.

A one-time bootstrap operation is needed to provision all existing orchard9 repos in a single run, transforming the fleet from zero deployed instances to full fleet coverage.

## Solution

A Woodpecker CI pipeline (`pipelines/bootstrap.yaml`) in the `orchard9/sdlc-cluster` repo that:

1. Queries the Gitea API for all repos in the `orchard9` org
2. For each repo, triggers the `deploy-project` pipeline via the Woodpecker API (same mechanism the reconcile pipeline uses)
3. Runs serially or with bounded parallelism to avoid overwhelming the cluster
4. Reports final counts: total repos found, deployments triggered, failures encountered

The pipeline is `event: custom` (manual/API trigger only) — it runs once at fleet launch, not on a schedule.

## Scope

This feature covers only the `pipelines/bootstrap.yaml` pipeline file in `orchard9/sdlc-cluster`. It depends on:

- `fleet-reconcile-pipeline` — provides the Woodpecker API trigger pattern this pipeline replicates
- `fleet-deploy-pipeline` — the pipeline each repo bootstrap call will invoke
- `fleet-helm-chart` — the chart each deploy invocation installs
- `fleet-secrets-infra` — the ESO ClusterSecretStore each deployment references
- `fleet-repo-scaffold` — the `orchard9/sdlc-cluster` repo with the canonical `pipelines/` directory

This feature does NOT cover:
- Ongoing reconciliation (that is `fleet-reconcile-pipeline`)
- DNS or TLS provisioning (that is `fleet-ingress-tls`)
- Re-runs or idempotent retry logic beyond what `deploy-project` already provides
- Monitoring the deployed fleet after bootstrap

## Bootstrap Strategy

### Gitea Pagination

The Gitea API returns paginated results. The bootstrap script must page through all results (`?page=1&limit=50`, then `?page=2`, etc.) until an empty page is returned.

### Trigger Mechanism

For each repo `orchard9/<repo-name>`, the bootstrap pipeline calls the Woodpecker API to trigger the `deploy-project` pipeline with:

```
SDLC_PROJECT_SLUG=<repo-name>
SDLC_REPO=orchard9/<repo-name>
SDLC_BRANCH=main
```

This is identical to what the reconcile pipeline does for new repos — bootstrap is the one-time equivalent.

### Rate Limiting / Parallelism

To avoid hammering the cluster with 80+ simultaneous helm installs, the bootstrap script triggers deployments serially or with a small concurrency limit (e.g., 5 at a time). Each trigger fires the `deploy-project` pipeline asynchronously in Woodpecker — the bootstrap pipeline does not wait for each helm install to complete.

### Repos to Skip

Some repos in the `orchard9` org are infrastructure repos that should not get an sdlc-server deployment:
- `sdlc-cluster` — the infrastructure management repo itself

The bootstrap pipeline skips any repo in a configurable skip list.

## Pipeline Parameters

| Variable | Description | Default |
|---|---|---|
| `CONCURRENCY` | Max simultaneous deploy-project triggers | `5` |
| `SKIP_REPOS` | Comma-separated repo names to skip | `sdlc-cluster` |
| `DRY_RUN` | If `true`, print what would be triggered without firing | `false` |

## Pipeline Steps

1. **bootstrap** — Uses a `python:3-alpine` (or similar) image with `curl` available. Runs a script that:
   - Loads `GITEA_TOKEN`, `GITEA_URL`, `WOODPECKER_TOKEN`, `WOODPECKER_URL` from environment (injected via ESO-backed secrets)
   - Pages through `GET /api/v1/orgs/orchard9/repos`
   - Filters out repos in `SKIP_REPOS`
   - For each remaining repo, calls the Woodpecker API to trigger `deploy-project`
   - Prints a summary: `Triggered N / M repos (K skipped, F failed)`

## Credentials

The pipeline requires two sets of credentials:

| Credential | Used For | Source |
|---|---|---|
| `GITEA_TOKEN` | Gitea API repo listing | Same `sdlc-fleet-gitea` GSM key as `fleet-secrets-infra` |
| `WOODPECKER_URL` | Woodpecker API base URL | Pipeline env var |
| `WOODPECKER_TOKEN` | Trigger pipeline via Woodpecker API | Stored in GSM, injected via ESO |

## Idempotency

Because the underlying `deploy-project` pipeline is idempotent (uses `helm upgrade --install`), running the bootstrap pipeline multiple times is safe. A second run redeploys or no-ops each project — no duplicates, no conflicts.

## Acceptance Criteria

1. `pipelines/bootstrap.yaml` exists in `orchard9/sdlc-cluster`
2. Running the pipeline with `DRY_RUN=true` prints a list of all orchard9 repos that would be triggered (excluding skip list), and exits 0
3. Running the pipeline without `DRY_RUN` triggers a `deploy-project` pipeline run for each non-skipped repo (visible in Woodpecker UI)
4. The pipeline does not trigger a `deploy-project` run for `sdlc-cluster` itself
5. After a full bootstrap run, all orchard9 repos have a corresponding namespace `sdlc-<slug>` in the cluster and a running sdlc-server Pod
6. Running the bootstrap pipeline a second time completes without error (idempotent via `deploy-project`)
7. Pipeline reports final counts: repos found, triggered, skipped, failed
