# Spec: fleet-reconcile-pipeline

## Feature

**Woodpecker reconcile-projects.yaml: scan orchard9 org and ensure every repo has a deployment**

## Problem

New projects are created in the `orchard9` Gitea org manually and sporadically. Each new project requires manually triggering `deploy-project.yaml` to provision its sdlc-server instance. As the org grows to 80+ projects, this manual step becomes a persistent operational gap — new repos may go undeployed for days, or be forgotten entirely. There is no automated mechanism to detect and close this gap.

## Solution

A Woodpecker CI pipeline (`pipelines/reconcile-projects.yaml`) in the `orchard9/sdlc-cluster` repo that runs on a schedule, lists all repositories in the `orchard9` Gitea org, compares them against deployed Helm releases, and triggers `deploy-project.yaml` for any repo that lacks a deployment.

The reconcile pipeline:
1. Lists all repos in `orchard9` via the Gitea API
2. Lists all deployed Helm releases in the cluster (releases matching pattern `sdlc-<slug>`)
3. Computes the delta: repos in Gitea with no matching Helm release
4. For each undeployed repo, triggers `deploy-project.yaml` via the Woodpecker API with the correct `SDLC_PROJECT_SLUG` and `SDLC_REPO` parameters
5. Runs on a cron schedule (daily) and is also manually triggerable

## Scope

This feature covers only `pipelines/reconcile-projects.yaml`. It depends on:
- `fleet-deploy-pipeline` — the `deploy-project.yaml` pipeline this reconcile loop triggers
- `fleet-helm-chart` — must be deployed so Helm release names are stable (`sdlc-<slug>`)
- `fleet-secrets-infra` — GCP/ESO secret access for kubeconfig and Gitea token
- `fleet-repo-scaffold` — `orchard9/sdlc-cluster` repo structure must exist

This feature does NOT cover:
- The `fleet-bootstrap` bulk provisioning of the initial 80+ existing projects (separate feature)
- TLS/ingress wildcard cert automation (separate feature: `fleet-ingress-tls`)
- Creating or modifying any Kubernetes resources directly — reconcile only triggers existing pipeline

## Exclusion Criteria

The following repo categories are excluded from reconcile deployment:
- Repos named `sdlc-cluster` (the management repo itself)
- Repos with topic/tag `no-sdlc` set in Gitea (opt-out mechanism)
- Repos that are forks (Gitea API `fork: true`)
- Archived repos (Gitea API `archived: true`)

## Trigger Mechanism

| Trigger | Description |
|---|---|
| `cron` schedule | Daily at 02:00 UTC — automated reconcile |
| `manual` / Woodpecker UI | Human-triggered reconcile run on demand |

The pipeline uses `event: [cron, manual]` in its Woodpecker config.

## Pipeline Logic (pseudocode)

```
repos = gitea_api.list_org_repos("orchard9", all_pages=True)
repos = filter(repos, not archived, not fork, not topic "no-sdlc", name != "sdlc-cluster")

releases = helm_list("--all-namespaces", "--filter", "^sdlc-")
deployed_slugs = { release.name.removeprefix("sdlc-") for release in releases }

for repo in repos:
    slug = repo.name  # slug == repo name by convention
    if slug not in deployed_slugs:
        woodpecker_api.trigger_pipeline(
            repo="orchard9/sdlc-cluster",
            pipeline="deploy-project.yaml",
            env={
                "SDLC_PROJECT_SLUG": slug,
                "SDLC_REPO": f"orchard9/{slug}",
                "SDLC_BRANCH": repo.default_branch or "main",
            }
        )
        log(f"Triggered deploy for {slug}")
    else:
        log(f"Already deployed: {slug}")

log(f"Reconcile complete: {len(repos)} repos checked, {triggered} triggered")
```

## Pipeline Steps

| Step | Image | Action |
|---|---|---|
| `reconcile` | `alpine/helm:3.14` + Gitea/Woodpecker CLI | Main reconcile script (shell or Python) |

The reconcile step has access to:
- `GITEA_TOKEN` — from ESO secret (Gitea admin API token for listing repos)
- `GITEA_URL` — Gitea base URL (`http://100.79.2.8:30300`)
- `KUBECONFIG` — from Woodpecker agent environment (for `helm list`)
- `WOODPECKER_TOKEN` — Woodpecker API token (for triggering pipelines)
- `WOODPECKER_URL` — Woodpecker server URL

## Idempotency Contract

Running the reconcile pipeline N times produces the same end state. `deploy-project.yaml` is idempotent — calling `helm upgrade --install` on an already-deployed project is a safe no-op. The reconcile pipeline checks before triggering but even if it double-triggers, the deploy pipeline handles it safely.

## Acceptance Criteria

1. `pipelines/reconcile-projects.yaml` exists in `orchard9/sdlc-cluster` and is syntactically valid Woodpecker YAML.
2. Manually triggering the pipeline with no parameters completes without error.
3. For each `orchard9` repo that has no `sdlc-<slug>` Helm release, the pipeline triggers `deploy-project.yaml`.
4. Already-deployed projects are logged as "already deployed" and no action is taken.
5. Excluded repos (`sdlc-cluster`, archived, fork, `no-sdlc` tag) do not trigger a deploy.
6. The pipeline runs on cron schedule without manual intervention.
7. Pipeline run output shows a summary: total repos checked, how many triggered, how many already deployed.

## Dependencies

- `fleet-deploy-pipeline` released — `pipelines/deploy-project.yaml` must be in place
- Woodpecker API token available in the runner environment (`WOODPECKER_TOKEN`)
- Gitea API access with `GITEA_TOKEN` (same ESO secret used by deploy-project)
- Woodpecker agent has `helm` CLI available (`alpine/helm:3.14` image)
- `kubectl`/kubeconfig configured on Woodpecker agent for `helm list --all-namespaces`

## Implementation Notes

- Use Gitea REST API pagination: `GET /api/v1/orgs/orchard9/repos?limit=50&page=N`
- `helm list --all-namespaces --output json --filter '^sdlc-'` gives the current deployed set
- Woodpecker pipeline trigger API: `POST /api/repos/{owner}/{name}/pipelines` with `{branch, variables}`
- The script may be inline shell in the YAML or a separate `scripts/reconcile.sh` — both are acceptable
- Consider adding a `--dry-run` parameter (pipeline env var) that logs what would be triggered without actually triggering — useful for validation
