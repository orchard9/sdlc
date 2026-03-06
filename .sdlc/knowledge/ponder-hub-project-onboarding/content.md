# Plan: Hub Self-Serve Project Onboarding

## Vision

A developer with a local git repo can add it to the cluster in under 2 minutes from the hub UI
— no external GitHub required. Click Add Project → enter name → copy two git commands → push
→ cluster instance appears. The deployment remote is the single source of truth: push to Gitea,
cluster updates automatically.

## Milestone

**slug:** v43-hub-project-onboarding
**title:** Hub Self-Serve Project Onboarding — create from local git repo
**status:** active

Acceptance test: Starting from the hub empty state, a user creates a new project by name,
receives a push URL and token, runs the two shown git commands, and sees a running fleet
instance appear in the hub UI within 60 seconds.

## Features

### 1. hub-create-repo-api
**title:** `POST /api/hub/create-repo` — Gitea repo creation with HTTP push credentials
**description:**
New endpoint that creates a Gitea repo and returns push credentials.

Tasks:
- Add `create_gitea_repo(name)` to `fleet.rs` — `POST /api/v1/orgs/{org}/repos`
- Add `create_repo_access_token(owner, repo)` to `fleet.rs` — generate HTTP push token
- Add `POST /api/hub/create-repo` handler in `routes/hub.rs` — takes `{ name }`, returns `{ push_url, token, repo_slug }`
- Call `trigger_provision()` after repo creation (mirror import route pattern)
- Return clear error if repo name already exists in Gitea

### 2. hub-create-repo-ui
**title:** CreateRepoModal — two-step UI: name form → instruction display
**description:**
New modal in HubPage for the create-from-local flow.

Tasks:
- Add `CreateRepoModal` component: step 1 = name input + Create button, step 2 = instruction display
- Instruction display shows two copy-able commands:
  `git remote add gitea <push_url>`
  `git push gitea main`
- Copy text: "This is your deployment remote — push here to update your cluster instance."
- Wire `createRepo()` call to `api/client.ts`
- Add `CreateRepoResponse` type to `lib/types.ts`
- Add "Add Project" button to HubPage (visible when hub is available, next to Import button)

### 3. hub-empty-state-replace
**title:** Replace hub empty state — Add Project as primary action
**description:**
Replace the static "Configure projects to send heartbeats. See ~/.sdlc/hub.yaml" message
in `EmptyState` with the Add Project button as the primary action.

Tasks:
- Update `EmptyState` component in HubPage to show "Add Project" button instead of static message
- Empty state should still show secondary text explaining what the hub does
- Ensure button opens CreateRepoModal

