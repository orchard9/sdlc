# Design: Scaffold orchard9/sdlc-cluster Repo

## Overview

This feature creates the `orchard9/sdlc-cluster` repository on the threesix/gitea instance and pushes an initial scaffold commit containing the canonical directory structure, Helm chart skeleton, pipeline stubs, and a `.woodpecker.yml` CI definition. There is no UI, no server-side code, and no Rust changes — this is a pure infrastructure scaffolding operation.

## Approach

### Step 1: Create the Repository via Gitea API

Use the Gitea REST API authenticated as `claude-agent` (admin token from `sdlc secrets env export gitea`):

```
POST http://100.79.2.8:30300/api/v1/orgs/orchard9/repos
Authorization: token <GITEA_TOKEN>
Content-Type: application/json

{
  "name": "sdlc-cluster",
  "description": "Manages sdlc-server deployments for the orchard9 fleet",
  "private": false,
  "auto_init": false
}
```

`auto_init: false` — we push the initial commit ourselves with the full scaffold in one shot.

### Step 2: Clone and Build Scaffold Locally

Clone into a temp directory, create the full directory tree and all files, then commit and push.

### Step 3: File Contents

#### `.woodpecker.yml`

```yaml
pipeline:
  lint-helm:
    image: alpine/helm:latest
    commands:
      - helm lint helm/sdlc-server
    when:
      branch: main
```

#### `README.md`

```markdown
# sdlc-cluster

Manages sdlc-server deployments for the orchard9 fleet.
```

#### `helm/sdlc-server/Chart.yaml`

```yaml
apiVersion: v2
name: sdlc-server
version: 0.1.0
description: Helm chart for deploying sdlc-server instances
```

#### `helm/sdlc-server/values.yaml`

```yaml
# Project identity
# project:
#   slug: ""       # unique project slug (e.g. "myproject")
#   repo: ""       # gitea repo URL (e.g. "http://gitea.svc/orchard9/myproject")

# Gitea connection
# gitea:
#   url: ""        # base URL of the gitea instance

# Ingress settings
# ingress:
#   domain: ""     # domain to expose sdlc-server on (e.g. "sdlc.myproject.example.com")
```

#### `helm/sdlc-server/templates/.gitkeep`

Empty file — keeps the `templates/` directory tracked in git. Real templates are added by `fleet-helm-chart`.

#### `pipelines/deploy-project.yaml.stub`

```yaml
# stub: deploy-project pipeline
# Filled in by fleet-deploy-pipeline feature.
```

#### `pipelines/reconcile-projects.yaml.stub`

```yaml
# stub: reconcile-projects pipeline
# Filled in by fleet-deploy-pipeline feature.
```

#### `external-secrets/cluster-secret-store.yaml.stub`

```yaml
# stub: ESO ClusterSecretStore
# Filled in by fleet-secrets-infra feature.
```

### Step 4: Push Initial Commit

```bash
git add -A
git commit -m "chore: scaffold sdlc-cluster repo"
git push origin main
```

## Implementation Strategy

The implementation task will:

1. Export gitea credentials via `sdlc secrets env export gitea` to obtain `GITEA_TOKEN` and `GITEA_URL`.
2. Call the Gitea API to create the `orchard9/sdlc-cluster` repo.
3. Clone the newly created (empty) repo into a temp directory.
4. Create all files as described above.
5. Commit with `chore: scaffold sdlc-cluster repo`.
6. Push to `origin main`.
7. Verify via API that the repo exists and the commit landed.

## Validation

After push, run `helm lint helm/sdlc-server` against the cloned tree to confirm the chart skeleton passes linting. This mirrors what the `.woodpecker.yml` CI step will do.

## Dependencies

- Gitea `claude-agent` token available via `sdlc secrets env export gitea`
- `orchard9` org exists on threesix/gitea (confirmed)
- `helm` CLI available on the developer machine (for local lint validation)
- `git` available

## Files Changed

All changes are outside the sdlc codebase — they live in the newly created `orchard9/sdlc-cluster` Gitea repository. No files in this workspace are modified.
