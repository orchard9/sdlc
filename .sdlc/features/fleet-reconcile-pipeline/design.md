# Design: fleet-reconcile-pipeline

## Overview

`reconcile-projects.yaml` is a Woodpecker CI pipeline in `orchard9/sdlc-cluster` that performs a scheduled diff between the orchard9 Gitea org (source of truth for repos) and the k3s cluster (source of truth for running deployments), then closes the gap by triggering `deploy-project.yaml` for any undeployed project.

---

## File Layout

```
orchard9/sdlc-cluster/
├── pipelines/
│   ├── deploy-project.yaml          # dependency (fleet-deploy-pipeline)
│   └── reconcile-projects.yaml      # THIS FEATURE
└── scripts/
    └── reconcile.sh                 # optional: factored-out shell logic
```

The pipeline can inline the reconcile logic directly in the YAML (preferred for < 60 lines) or source it from `scripts/reconcile.sh` for readability.

---

## Pipeline YAML Structure

```yaml
# pipelines/reconcile-projects.yaml

when:
  - event: cron
    cron: reconcile-daily
  - event: manual

steps:
  - name: reconcile
    image: alpine/helm:3.14
    environment:
      GITEA_URL:
        from_secret: gitea_url
      GITEA_TOKEN:
        from_secret: gitea_token
      WOODPECKER_URL:
        from_secret: woodpecker_url
      WOODPECKER_TOKEN:
        from_secret: woodpecker_token
      DRY_RUN:
        from_secret: reconcile_dry_run   # optional, defaults to "false"
    commands:
      - apk add --no-cache curl jq
      - sh /woodpecker/src/scripts/reconcile.sh
```

**Cron setup** — the `reconcile-daily` cron must be registered in Woodpecker for the `orchard9/sdlc-cluster` repo (UI: Settings → Cron, name `reconcile-daily`, schedule `0 2 * * *`).

---

## Reconcile Script Design

### reconcile.sh

```sh
#!/usr/bin/env sh
set -euo pipefail

GITEA_URL="${GITEA_URL}"
GITEA_TOKEN="${GITEA_TOKEN}"
WOODPECKER_URL="${WOODPECKER_URL}"
WOODPECKER_TOKEN="${WOODPECKER_TOKEN}"
DRY_RUN="${DRY_RUN:-false}"

ORG="orchard9"
SELF_REPO="sdlc-cluster"

# ── 1. Collect all orchard9 repos (paginated) ──────────────────────────────
page=1
repos_json="[]"
while true; do
  batch=$(curl -sf \
    -H "Authorization: token ${GITEA_TOKEN}" \
    "${GITEA_URL}/api/v1/orgs/${ORG}/repos?limit=50&page=${page}")
  count=$(echo "$batch" | jq 'length')
  [ "$count" -eq 0 ] && break
  repos_json=$(echo "$repos_json $batch" | jq -s 'add')
  page=$((page + 1))
done

# ── 2. Filter: skip archived, forks, self, and no-sdlc tagged ─────────────
eligible=$(echo "$repos_json" | jq -r '
  .[] |
  select(.archived == false) |
  select(.fork == false) |
  select(.name != "'"${SELF_REPO}"'") |
  select((.topics // []) | index("no-sdlc") == null) |
  .name
')

# ── 3. Collect deployed Helm releases ──────────────────────────────────────
deployed=$(helm list --all-namespaces --output json --filter '^sdlc-' \
  | jq -r '.[].name | ltrimstr("sdlc-")')

# ── 4. Diff and trigger ────────────────────────────────────────────────────
triggered=0
skipped=0

for slug in $eligible; do
  if echo "$deployed" | grep -qx "$slug"; then
    echo "  already deployed: ${slug}"
    skipped=$((skipped + 1))
  else
    default_branch=$(echo "$repos_json" \
      | jq -r --arg s "$slug" '.[] | select(.name == $s) | .default_branch // "main"')
    echo "  → triggering deploy for: ${slug} (branch: ${default_branch})"

    if [ "${DRY_RUN}" != "true" ]; then
      curl -sf -X POST \
        -H "Authorization: Bearer ${WOODPECKER_TOKEN}" \
        -H "Content-Type: application/json" \
        "${WOODPECKER_URL}/api/repos/${ORG}/sdlc-cluster/pipelines" \
        -d "{
          \"branch\": \"main\",
          \"variables\": {
            \"SDLC_PROJECT_SLUG\": \"${slug}\",
            \"SDLC_REPO\": \"${ORG}/${slug}\",
            \"SDLC_BRANCH\": \"${default_branch}\"
          }
        }"
    fi
    triggered=$((triggered + 1))
  fi
done

echo ""
echo "Reconcile summary: checked=$(echo "$eligible" | wc -w | tr -d ' ')  triggered=${triggered}  skipped=${skipped}  dry_run=${DRY_RUN}"
```

---

## Secrets Required

| Secret name | Value | How set |
|---|---|---|
| `gitea_url` | `http://100.79.2.8:30300` | Woodpecker repo secret |
| `gitea_token` | Gitea admin token (`claude-agent`) | Woodpecker repo secret (from ESO/GCP) |
| `woodpecker_url` | Woodpecker server URL | Woodpecker repo secret |
| `woodpecker_token` | Woodpecker API token | Woodpecker repo secret |
| `reconcile_dry_run` | `"false"` (default) | Woodpecker repo secret (optional) |

All secrets are set at the `orchard9/sdlc-cluster` repo level in the Woodpecker UI (or via Woodpecker API), consistent with how `deploy-project.yaml` secrets are managed.

---

## Flow Diagram

```
┌─────────────────────────────────────────────────────┐
│  Woodpecker cron / manual trigger                   │
└───────────────────┬─────────────────────────────────┘
                    │
                    ▼
          ┌─────────────────┐
          │  Gitea API      │
          │  list org repos │
          └────────┬────────┘
                   │  all repos (paginated)
                   ▼
          ┌─────────────────┐
          │  Filter:        │
          │  -archived      │
          │  -forks         │
          │  -sdlc-cluster  │
          │  -no-sdlc topic │
          └────────┬────────┘
                   │  eligible slugs
                   ▼
          ┌─────────────────┐
          │  helm list      │
          │  --all-ns       │
          │  ^sdlc-         │
          └────────┬────────┘
                   │  deployed slugs
                   ▼
          ┌─────────────────────────────┐
          │  diff: eligible - deployed  │
          └──────────────┬──────────────┘
                         │  undeployed slugs
                         ▼
              ┌──────────────────┐
              │  For each slug:  │
              │  POST Woodpecker │
              │  deploy pipeline │
              └──────────────────┘
                         │
                         ▼
              ┌──────────────────┐
              │  deploy-project  │
              │  .yaml runs →    │
              │  helm install    │
              └──────────────────┘
```

---

## Error Handling

| Scenario | Behavior |
|---|---|
| Gitea API unreachable | `curl -sf` fails, pipeline exits non-zero — Woodpecker marks run failed |
| Helm list fails (kubeconfig issue) | Script exits non-zero — pipeline fails with visible error |
| Single deploy trigger fails | Script continues to remaining repos; logs the error and increments a `failed` counter; exits non-zero at end if any failures |
| Woodpecker API token expired | POST fails for all triggers; pipeline exits non-zero |

The "continue on individual deploy failure" behavior ensures a bad slug doesn't block all subsequent repos. The non-zero exit at the end surfaces the failure in Woodpecker.

---

## Dry Run Mode

Setting the `reconcile_dry_run` secret (or pipeline variable) to `"true"` makes the script log all intended triggers without making any API calls to Woodpecker. This is the safe way to validate the reconcile logic without side effects — useful for:
- Initial testing of the pipeline
- Auditing what would be deployed after a long org expansion
- Debugging exclusion filter logic

---

## Cron Registration

The daily cron must be registered via the Woodpecker UI or API after the pipeline file is pushed:

```
Repo: orchard9/sdlc-cluster
Cron name: reconcile-daily
Schedule: 0 2 * * *   (02:00 UTC daily)
Branch: main
```

This is a one-time operational setup step; it is not done by the pipeline itself.

---

## Relationship to fleet-bootstrap

`fleet-bootstrap` (separate feature) handles the initial one-time provisioning of the existing 80+ orchard9 repos. The reconcile pipeline is designed for ongoing drift detection after the fleet is established, not bulk initial provisioning. The two complement each other:
- `fleet-bootstrap` — runs once, deploys all existing repos
- `fleet-reconcile-pipeline` — runs daily, catches anything new or missed
