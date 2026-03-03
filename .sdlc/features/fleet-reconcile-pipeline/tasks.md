# Tasks: fleet-reconcile-pipeline

## T1: Write reconcile.sh script

Create `scripts/reconcile.sh` in `orchard9/sdlc-cluster` with the full reconcile logic:
- Paginated Gitea API listing of all `orchard9` repos
- Filter: exclude archived, forks, `sdlc-cluster` itself, repos with `no-sdlc` topic
- `helm list --all-namespaces --output json --filter '^sdlc-'` to get deployed slugs
- Diff: compute undeployed = eligible - deployed
- For each undeployed slug: POST to Woodpecker API to trigger `deploy-project.yaml`
- Dry-run mode: `DRY_RUN=true` logs intended actions without triggering
- Summary line at end: `Reconcile summary: checked=N triggered=M skipped=K`
- Script is POSIX sh (compatible with `alpine` base image), uses `curl` and `jq`

## T2: Write pipelines/reconcile-projects.yaml

Create the Woodpecker pipeline YAML `pipelines/reconcile-projects.yaml` in `orchard9/sdlc-cluster`:
- `when` triggers: `event: [cron, manual]`
- Single step `reconcile` using `alpine/helm:3.14` image
- Installs `curl` and `jq` via `apk add --no-cache`
- Reads secrets: `gitea_url`, `gitea_token`, `woodpecker_url`, `woodpecker_token`, `reconcile_dry_run`
- Executes `scripts/reconcile.sh`

## T3: Register reconcile-daily cron in Woodpecker

Configure the daily cron schedule for the `orchard9/sdlc-cluster` repo in Woodpecker:
- Cron name: `reconcile-daily`
- Schedule: `0 2 * * *` (02:00 UTC daily)
- Branch: `main`

This is done via Woodpecker API or UI after the pipeline file is pushed.

## T4: Push reconcile files to orchard9/sdlc-cluster

Commit and push `scripts/reconcile.sh` and `pipelines/reconcile-projects.yaml` to the `orchard9/sdlc-cluster` repo on threesix/gitea:
- Commit message: `feat: add reconcile-projects pipeline and script`
- Verify Woodpecker detects the new pipeline file

## T5: Validate dry-run mode

Trigger the reconcile pipeline manually with `DRY_RUN=true` and verify:
- Pipeline completes without error
- Output lists all eligible orchard9 repos
- Output shows which slugs are deployed vs. undeployed
- No Woodpecker deploy triggers are fired
- Summary line is printed

## T6: Validate live reconcile against a test repo

Create a test repo `orchard9/reconcile-test-repo` in Gitea (no `no-sdlc` tag, not archived, not fork), then:
- Confirm it appears in dry-run output as "undeployed"
- Trigger live reconcile (DRY_RUN=false)
- Confirm `deploy-project.yaml` is triggered for `reconcile-test-repo`
- Confirm subsequent reconcile run marks it as "already deployed"
- Clean up: delete `orchard9/reconcile-test-repo` from Gitea
