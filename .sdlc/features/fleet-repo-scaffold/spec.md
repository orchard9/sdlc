# Spec: Scaffold orchard9/sdlc-cluster Repo

## Feature

**Slug:** fleet-repo-scaffold  
**Milestone:** v17-fleet-foundation  
**Title:** Scaffold orchard9/sdlc-cluster repo with base directory structure and .woodpecker.yml

## Problem Statement

The `sdlc-cluster` repository (`orchard9/sdlc-cluster` on threesix/gitea) does not exist yet. This is the infrastructure management project that will deploy and manage sdlc-server instances across all orchard9 projects. Before any Helm charts, pipelines, or secrets infrastructure can be built, the repo must exist with the correct directory skeleton and Woodpecker CI wired up for chart linting.

## Goals

1. Create the `orchard9/sdlc-cluster` repository on threesix/gitea (`http://100.79.2.8:30300`).
2. Establish the canonical directory structure agreed on in ponder session 4 (see `sdlc-cluster-architecture.md`).
3. Add a `.woodpecker.yml` at the repo root that runs `helm lint` on push — the sdlc-cluster repo's own CI.
4. Add placeholder/stub files so each directory is non-empty and the repo structure is legible at a glance.
5. Push an initial commit so the repo is in a known good state for downstream features (fleet-helm-chart, fleet-deploy-pipeline, fleet-secrets-infra) to build on.

## Out of Scope

- Actual Helm chart content (that is fleet-helm-chart)
- Woodpecker deploy-project or reconcile pipelines (that is fleet-deploy-pipeline)
- ESO ClusterSecretStore manifest (that is fleet-secrets-infra)
- Any k3s or cluster-side resources — this feature is repo creation only

## Acceptance Criteria

1. `orchard9/sdlc-cluster` exists on threesix/gitea and is publicly readable by the `claude-agent` admin account.
2. Repo contains the following top-level directories:
   - `helm/sdlc-server/` — Helm chart skeleton (Chart.yaml + values.yaml + templates/ directory)
   - `pipelines/` — home for deploy-project.yaml and reconcile-projects.yaml (stubs only in this feature)
   - `external-secrets/` — home for cluster-secret-store.yaml (stub only in this feature)
3. `.woodpecker.yml` at repo root runs `helm lint helm/sdlc-server` on push to `main`.
4. A `README.md` in the repo root describes the purpose: "Manages sdlc-server deployments for the orchard9 fleet."
5. Initial commit is pushed with message `chore: scaffold sdlc-cluster repo`.
6. `helm lint helm/sdlc-server` passes locally on a developer machine with Helm installed.

## Directory Structure (target)

```
orchard9/sdlc-cluster/
├── .woodpecker.yml                  # Helm lint CI for this repo
├── README.md                        # Purpose statement
├── helm/
│   └── sdlc-server/
│       ├── Chart.yaml               # name: sdlc-server, version: 0.1.0
│       ├── values.yaml              # documented placeholder values
│       └── templates/
│           └── .gitkeep             # empty until fleet-helm-chart fills it
├── pipelines/
│   ├── deploy-project.yaml.stub     # placeholder — filled by fleet-deploy-pipeline
│   └── reconcile-projects.yaml.stub # placeholder — filled by fleet-deploy-pipeline
└── external-secrets/
    └── cluster-secret-store.yaml.stub # placeholder — filled by fleet-secrets-infra
```

## .woodpecker.yml Behavior

- Trigger: push to `main` branch
- Step: `helm lint helm/sdlc-server` using an image that has Helm installed (e.g., `alpine/helm:latest`)
- No deploy steps — this is CI for the sdlc-cluster repo itself, not for deploying projects

## Dependencies

- Gitea `claude-agent` admin token must be available (stored in sdlc secrets under `gitea`)
- `orchard9` org must exist on threesix/gitea (confirmed created during ponder setup)
- Helm must be available in the Woodpecker runner image

## Implementation Notes

- Use the Gitea REST API (`POST /api/v1/orgs/orchard9/repos`) with the `claude-agent` token to create the repo
- Clone locally, create all files, push
- The `Chart.yaml` minimum valid content: `apiVersion: v2`, `name: sdlc-server`, `version: 0.1.0`
- The `values.yaml` should document the expected values shape (project.slug, project.repo, gitea.url, ingress.domain) as comments — actual logic lives in fleet-helm-chart
- Stub files use `.stub` extension so they don't confuse Helm or Woodpecker before real content is added
