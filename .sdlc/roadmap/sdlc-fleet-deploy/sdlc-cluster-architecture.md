# sdlc-cluster Architecture

## What it is
A standalone infrastructure management project (`orchard9/sdlc-cluster`) that deploys and manages sdlc-server instances across many projects. sdlc itself is unchanged.

## Core Pattern

```
orchard9/<slug> (Gitea repo with .sdlc/)
          │
          ├─ git-sync sidecar watches this repo
          │
k3s pod:  ├─ sdlc-server (unchanged binary) ← reads/writes /workspace/<slug>/.sdlc/
          ├─ git-sync (sidecar) ← clones + pulls from Gitea → /workspace/<slug>/
          │
ingress:  └─ <slug>.sdlc.threesix.ai → pod:port 8080
```

## Pod Model
- **Main container:** `sdlc-server` binary (unchanged). Reads from shared emptyDir volume.
- **Sidecar container:** `registry.k8s.io/git-sync/git-sync:v4`. Syncs Gitea → shared emptyDir.
- **Volume:** `emptyDir` — no Longhorn PVC. stateless pod, git is the persistence layer.
- **Namespace per project:** `sdlc-<slug>`

## Secrets
- GCP Secret Manager key: `sdlc-fleet-gitea` (url, token, user, org)
- ESO ClusterSecretStore → pulls from GSM → k8s Secret `gitea-credentials` per namespace
- git-sync uses GITEA_TOKEN from Secret to authenticate Gitea clone

## Helm Chart (helm/sdlc-server/)
Values API:
```yaml
project:
  slug: my-project
  repo: orchard9/my-project
  branch: main
gitea:
  url: http://100.79.2.8:30300
  externalSecret:
    secretStore: gcp-secret-store
    gsmKey: sdlc-fleet-gitea
ingress:
  domain: sdlc.threesix.ai
  tlsSecretName: sdlc-wildcard-tls
```

## Woodpecker Pipelines
1. **deploy-project.yaml** — installs/upgrades one project by slug. Idempotent. Triggered on-demand.
2. **reconcile-projects.yaml** — scheduled pipeline. Scans orchard9 Gitea org → diffs vs deployed pods → runs deploy-project for new repos.

## Repo Structure
```
orchard9/sdlc-cluster/
├── helm/sdlc-server/           # Helm chart for one sdlc-server pod
│   ├── Chart.yaml
│   ├── values.yaml
│   └── templates/
│       ├── namespace.yaml
│       ├── deployment.yaml     # sdlc-server + git-sync sidecar
│       ├── service.yaml
│       ├── ingress.yaml        # <slug>.sdlc.threesix.ai
│       └── external-secret.yaml
├── pipelines/
│   ├── deploy-project.yaml     # On-demand: deploy one project
│   └── reconcile-projects.yaml # Scheduled: sync all orchard9 repos
├── external-secrets/
│   └── cluster-secret-store.yaml
└── .woodpecker.yml             # sdlc-cluster's own CI (chart linting)
```

## Open Questions
1. GCP project ID + GSM key path for sdlc-fleet-gitea secret
2. Is sdlc.threesix.ai in GCP Cloud DNS? (for cert-manager DNS01 wildcard cert)
3. Is there an existing sdlc-server Docker image in Zot? Or does this need a build pipeline?
4. git-sync auth: token-in-URL (`http://user:token@host/repo`) or HTTP header injection?
5. Bootstrap: how to provision the first 80 projects (loop script triggering Woodpecker per project)

## Key Decisions
- sdlc is FROZEN — zero changes to Rust codebase
- sdlc-cluster is a NEW project (orchard9/sdlc-cluster)
- No Longhorn PVCs — git-sync sidecar + emptyDir
- Woodpecker = the CD system (not ArgoCD, not raw CronJob)
- GCP Secret Manager = credential store (ESO bridge)
- orchard9/ org = fresh, new projects only (jordan's 145 stay under jordan/)
