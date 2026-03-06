# Hub Project Onboarding — Scrapbook

## Problem

The hub empty state shows: "Configure projects to send heartbeats. See ~/.sdlc/hub.yaml"

This is a dead end. It points to v1 heartbeat infrastructure (project calls home to hub). The cluster
now does v2 discovery via k8s namespace enumeration and git-sync. The current import flow
(`POST /api/hub/import`) requires an existing remote URL (GitHub), which excludes local-only projects.

A developer with a local git repo has no self-serve path to get it into the cluster.

## User Flow (decided)

1. Click "Add Project" button (replaces the empty state message as primary action)
2. Enter a project name
3. Hub creates a new Gitea repo under `orchard9/` org
4. Hub creates an HTTP access token scoped to that repo
5. Instructions screen shows:
   - `git remote add gitea http://<user>:<token>@<gitea-host>/orchard9/<name>.git`
   - `git push gitea main`
   - "This is your deployment remote — push here to update your cluster instance."
6. After push, reconcile pipeline provisions the cluster instance automatically
7. Instance appears in Running section within ~60 seconds

## Key Decisions (from fit-impact + beat analysis)

**Credential strategy: HTTP token (not SSH deploy key)**
- One command to add remote, no SSH key management
- Create a per-project Gitea user (`<name>-bot`) or use per-repo access token
- Token displayed once in UI with copy button
- Recommendation: per-repo access token via Gitea API if available, else per-project user

**Sync story: push-to-gitea = deploy**
- The git-sync sidecar in every k8s pod polls Gitea every 30s
- User pushes to gitea remote → cluster instance updates automatically
- No need to keep GitHub and Gitea in sync
- Copy must say "deployment remote" not "second remote to keep in sync"

**Trigger provision immediately**
- `POST /api/hub/create-repo` must call `trigger_provision()` after creation
- Mirrors the pattern in `POST /api/hub/import`
- User sees instance within ~60s, not at next scheduled reconcile run

## Technical Scope (from fit-impact)

- `crates/sdlc-server/src/fleet.rs` — new `create_gitea_repo()` + `create_repo_access_token()`
- `crates/sdlc-server/src/routes/hub.rs` — new `POST /api/hub/create-repo` handler
- `frontend/src/pages/HubPage.tsx` — CreateRepoModal (name form → instruction display), EmptyState update
- `frontend/src/api/client.ts` — new `createRepo()` call
- `frontend/src/lib/types.ts` — `CreateRepoResponse` type

## Context

- `v42-fleet-control-plane` is the parent milestone (active, nearly done)
- `fleet-management-api` and `fleet-management-ui` are the direct analogues (released)
- `fleet-reconcile-pipeline` handles auto-provisioning from Gitea (released)
- Gitea org: `orchard9/`, admin token in `app.gitea_token`

