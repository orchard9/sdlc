# Design: fleet-bootstrap

## Overview

The fleet-bootstrap feature delivers a single Woodpecker CI pipeline file (`pipelines/bootstrap.yaml`) in the `orchard9/sdlc-cluster` repo. When triggered manually or via the Woodpecker API, it pages through all repos in the `orchard9` Gitea org, filters out infrastructure repos, and fires one `deploy-project` pipeline trigger per repo — seeding the entire fleet from zero to fully provisioned in a single run.

There are no Rust changes, no frontend changes, and no new server routes. This is a pure infrastructure pipeline file.

---

## Architecture

### Flow

```
Operator (manual trigger or API call)
       │
       ▼
  bootstrap pipeline (Woodpecker)
       │
       ├── GET /api/v1/orgs/orchard9/repos?page=1&limit=50
       ├── GET /api/v1/orgs/orchard9/repos?page=2&limit=50
       │    ...until empty page
       │
       ├── filter: skip repos in SKIP_REPOS list
       │
       ├── for each qualifying repo (batched N at a time):
       │       POST /api/v1/repos/{owner}/{repo}/raw/ ... (Woodpecker API trigger)
       │       → triggers deploy-project pipeline for that repo
       │
       └── print summary: N triggered, K skipped, F failed
```

### Where This Fits

```
orchard9/sdlc-cluster/
├── pipelines/
│   ├── deploy-project.yaml       ← per-repo idempotent helm install (fleet-deploy-pipeline)
│   ├── reconcile-projects.yaml   ← scheduled new-repo detection (fleet-reconcile-pipeline)
│   └── bootstrap.yaml            ← THIS FEATURE: one-time full-fleet provision
└── helm/sdlc-server/             ← chart each deploy invokes (fleet-helm-chart)
```

---

## Implementation

### Pipeline File: `pipelines/bootstrap.yaml`

```yaml
pipeline:
  bootstrap:
    image: python:3.12-alpine
    environment:
      - GITEA_URL
      - GITEA_TOKEN
      - WOODPECKER_URL
      - WOODPECKER_TOKEN
      - CONCURRENCY=5
      - SKIP_REPOS=sdlc-cluster
      - DRY_RUN=false
    secrets:
      - gitea_token
      - woodpecker_token
    commands:
      - apk add --no-cache curl
      - python3 /pipeline/bootstrap.py
    when:
      event: custom
```

> **Note:** The pipeline uses `event: custom` — it is not triggered by pushes. Credentials are injected via Woodpecker secrets (backed by ESO or set manually in Woodpecker admin UI), not hard-coded.

### Bootstrap Script: `pipelines/bootstrap.py`

The script is embedded inline or co-located in the `pipelines/` directory. It uses only Python standard library + `urllib` (no external dependencies beyond `curl` for health checks, which is optional).

#### Pseudocode

```python
import os, json, urllib.request, urllib.parse, time, sys
from concurrent.futures import ThreadPoolExecutor

GITEA_URL      = os.environ["GITEA_URL"]
GITEA_TOKEN    = os.environ.get("gitea_token") or os.environ["GITEA_TOKEN"]
WP_URL         = os.environ["WOODPECKER_URL"]
WP_TOKEN       = os.environ.get("woodpecker_token") or os.environ["WOODPECKER_TOKEN"]
CONCURRENCY    = int(os.environ.get("CONCURRENCY", "5"))
SKIP_REPOS     = set(os.environ.get("SKIP_REPOS", "sdlc-cluster").split(","))
DRY_RUN        = os.environ.get("DRY_RUN", "false").lower() == "true"
ORG            = "orchard9"

def list_repos(org):
    """Page through all repos in the org."""
    repos, page = [], 1
    while True:
        url = f"{GITEA_URL}/api/v1/orgs/{org}/repos?limit=50&page={page}"
        req = urllib.request.Request(url, headers={"Authorization": f"token {GITEA_TOKEN}"})
        with urllib.request.urlopen(req) as resp:
            batch = json.loads(resp.read())
        if not batch:
            break
        repos.extend(r["name"] for r in batch)
        page += 1
    return repos

def trigger_deploy(repo_name):
    """Trigger deploy-project pipeline for one repo."""
    # Woodpecker API: POST /api/repos/{owner}/{repo}/pipelines
    # with env overrides for SDLC_PROJECT_SLUG, SDLC_REPO, SDLC_BRANCH
    url = f"{WP_URL}/api/repos/{ORG}/sdlc-cluster/pipelines"
    payload = json.dumps({
        "branch": "main",
        "variables": {
            "SDLC_PROJECT_SLUG": repo_name,
            "SDLC_REPO": f"{ORG}/{repo_name}",
            "SDLC_BRANCH": "main",
        }
    }).encode()
    req = urllib.request.Request(
        url, data=payload,
        headers={
            "Authorization": f"Bearer {WP_TOKEN}",
            "Content-Type": "application/json",
        }
    )
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read())

def main():
    repos = list_repos(ORG)
    to_deploy = [r for r in repos if r not in SKIP_REPOS]
    skipped   = [r for r in repos if r in SKIP_REPOS]

    print(f"Found {len(repos)} repos. Deploying {len(to_deploy)}, skipping {len(skipped)}.")
    if DRY_RUN:
        print("DRY_RUN=true — would trigger:")
        for r in to_deploy:
            print(f"  deploy-project: {ORG}/{r}")
        sys.exit(0)

    failed = []
    with ThreadPoolExecutor(max_workers=CONCURRENCY) as pool:
        futures = {pool.submit(trigger_deploy, r): r for r in to_deploy}
        for fut, repo in futures.items():
            try:
                result = fut.result()
                print(f"  [OK] triggered deploy-project for {repo} (pipeline #{result.get('number', '?')})")
            except Exception as e:
                print(f"  [FAIL] {repo}: {e}")
                failed.append(repo)

    print(f"\nSummary: {len(to_deploy) - len(failed)} triggered, {len(skipped)} skipped, {len(failed)} failed.")
    if failed:
        print(f"Failed repos: {', '.join(failed)}")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

---

## Credentials and Secrets

| Secret | Source | Used For |
|---|---|---|
| `gitea_token` | Woodpecker secret (from GSM `sdlc-fleet-gitea`) | Gitea API repo listing |
| `woodpecker_token` | Woodpecker secret (admin token) | Woodpecker API pipeline trigger |
| `GITEA_URL` | Pipeline env var | Gitea instance base URL |
| `WOODPECKER_URL` | Pipeline env var | Woodpecker instance base URL |

Woodpecker secrets are set once in the Woodpecker admin UI for the `orchard9/sdlc-cluster` repo. They are not stored in the repo or in YAML.

---

## Idempotency

The bootstrap pipeline is safe to re-run:
- `list_repos` is read-only (GET)
- Each `trigger_deploy` fires the `deploy-project` pipeline, which uses `helm upgrade --install` — idempotent
- Re-running bootstrap on an already-deployed fleet is a no-op from the fleet perspective (each deploy-project run will fast-apply or skip identical releases)

---

## Concurrency Model

The script uses `ThreadPoolExecutor` with `max_workers=CONCURRENCY` (default 5). Each thread fires a Woodpecker API trigger and returns the pipeline number. The script does NOT wait for each `deploy-project` pipeline to complete — it simply enqueues the work in Woodpecker and moves on. Full fleet provisioning is therefore bounded by the cluster's Helm throughput, not by the bootstrap script's serial speed.

---

## Validation

### Dry Run Verification (no cluster required)

```bash
# Trigger bootstrap with DRY_RUN=true
woodpecker-cli pipeline create orchard9/sdlc-cluster \
  --set DRY_RUN=true
# Verify: pipeline prints list of repos without firing any triggers
```

### Full Bootstrap Verification (cluster required)

After the bootstrap run:
```bash
# Check all deploy-project pipelines were enqueued
woodpecker-cli pipeline list orchard9/sdlc-cluster

# Check namespaces provisioned
kubectl get namespaces | grep sdlc-

# Spot-check one project
kubectl get pods -n sdlc-<repo-name>
```

---

## Files Changed

All changes are in the `orchard9/sdlc-cluster` Gitea repository. No files in the sdlc workspace are modified.

| File | Action |
|---|---|
| `pipelines/bootstrap.yaml` | Create — Woodpecker pipeline definition |
| `pipelines/bootstrap.py` | Create — bootstrap script co-located with pipeline |
| `pipelines/reconcile-projects.yaml.stub` | Replace stub with real content (this feature replaces the stub left by fleet-repo-scaffold) — out of scope but noted |

The stub file `pipelines/reconcile-projects.yaml.stub` from `fleet-repo-scaffold` does not need replacement here; bootstrap is a separate pipeline from reconcile.

---

## Dependencies (Delivery Order)

| Dependency | Required For |
|---|---|
| `fleet-repo-scaffold` | `orchard9/sdlc-cluster` repo exists with `pipelines/` directory |
| `fleet-helm-chart` | Chart that `deploy-project` invokes |
| `fleet-secrets-infra` | `ClusterSecretStore` + GSM secret for credentials |
| `fleet-deploy-pipeline` | `pipelines/deploy-project.yaml` — what bootstrap triggers |
| `fleet-ingress-tls` | Wildcard TLS cert (needed for deployed instances to be reachable via HTTPS) |
| `fleet-reconcile-pipeline` | Optional — bootstrap and reconcile are independent; reconcile handles future repos |
