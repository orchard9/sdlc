# Tasks: Scaffold orchard9/sdlc-cluster Repo

## T1: Create orchard9/sdlc-cluster repository on threesix/gitea

Export the gitea token via `sdlc secrets env export gitea`, then call:

```
POST http://100.79.2.8:30300/api/v1/orgs/orchard9/repos
```

with `name: sdlc-cluster`, `description: "Manages sdlc-server deployments for the orchard9 fleet"`, `private: false`, `auto_init: false`.

Verify the repo URL is accessible.

## T2: Clone repo and create all scaffold files

Clone the empty repo into a temp directory. Create:

- `.woodpecker.yml` — helm lint CI step (image: `alpine/helm:latest`, trigger: push to main)
- `README.md` — one-line purpose statement
- `helm/sdlc-server/Chart.yaml` — apiVersion: v2, name: sdlc-server, version: 0.1.0
- `helm/sdlc-server/values.yaml` — commented placeholder values (project.slug, project.repo, gitea.url, ingress.domain)
- `helm/sdlc-server/templates/.gitkeep` — empty file to track templates/ dir
- `pipelines/deploy-project.yaml.stub` — stub comment
- `pipelines/reconcile-projects.yaml.stub` — stub comment
- `external-secrets/cluster-secret-store.yaml.stub` — stub comment

## T3: Validate helm lint locally

Run `helm lint helm/sdlc-server` inside the cloned directory. Confirm it exits 0 with no errors. Fix Chart.yaml or values.yaml if lint fails.

## T4: Commit and push initial scaffold

```bash
git add -A
git commit -m "chore: scaffold sdlc-cluster repo"
git push origin main
```

Verify via Gitea API (`GET /api/v1/repos/orchard9/sdlc-cluster/commits`) that the commit appears.
