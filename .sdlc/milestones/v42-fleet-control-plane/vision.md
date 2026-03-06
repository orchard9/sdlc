# Vision: v42-fleet-control-plane

## Why this matters

The sdlc fleet has no front door. There's one manually deployed instance at
`sdlc.sdlc.threesix.ai` with no authentication — anyone with the URL has full access.
There's no way to see what's running, start new instances, or import repos.

The fleet is infrastructure without a control plane.

## What a user can do when this ships

1. Go to `sdlc.threesix.ai` and sign in with their Google account
   (livelyideo.tv, masq.me, or virtualcommunities.ai domains)
2. See every running sdlc project instance in the fleet — name, health status,
   active milestone, whether agents are running
3. Click any instance to navigate directly to its workspace
4. See repos in the Gitea org that don't have instances yet, and start one
   with a single click
5. Import an external git repo (GitHub, GitLab, etc.) by pasting a URL —
   the repo gets mirrored to Gitea and an instance is automatically provisioned
6. All sdlc instances (`*.sdlc.threesix.ai`) are behind the same Google auth
   gate — no unauthenticated access to any project

## What this does NOT include

- Per-user access control (all authenticated users see all projects)
- Instance deletion from the UI (too destructive for v1)
- Billing or usage metering
- Multi-tenancy
