# Spec: Git Commit Agent

## Summary

Add a `POST /api/git/commit` endpoint that uses `spawn_agent_run` to generate a commit message from the current diff and commit all changes via the existing `sdlc commit` CLI command. The frontend `GitStatusChip` already calls this endpoint — this feature provides the backend implementation.

## Problem

The `GitStatusChip` component in the sidebar has a "Commit" button that calls `POST /api/git/commit`, but no such endpoint exists. Users clicking the button get a 404. The commit flow needs an agent to read the diff, generate a meaningful conventional-commit-style message, and execute `sdlc commit --message "<msg>"`.

## Solution

### Backend: `POST /api/git/commit`

Add a new route handler in `crates/sdlc-server/src/routes/git.rs` that:

1. Accepts a POST request (no body required — commits whatever is dirty/staged).
2. Calls `spawn_agent_run` with a prompt that instructs the agent to:
   - Run `git diff HEAD --stat` and `git diff HEAD` to understand the changes.
   - Generate a single-line conventional-commit message (120 chars max).
   - Execute `sdlc commit --message "<generated message>"`.
   - Report the result (commit SHA, merge status, ahead/behind).
3. Returns immediately with a `run_id` (agent runs asynchronously).
4. Emits an SSE event on completion so the frontend can refresh git status.

### Frontend: Update GitStatusChip

The existing `GitStatusChip` already calls `POST /api/git/commit`. After the endpoint exists, the chip should:
- Show a loading/committing state while the agent runs.
- Listen for the SSE completion event and refresh git status.

### Route Registration

Register the new route in `build_router_from_state` in `lib.rs`:
```
.route("/api/git/commit", post(routes::git::start_git_commit))
```

## Non-Goals

- Custom commit messages from the UI (agent always auto-generates).
- Pushing to remote (the `sdlc commit` command never pushes).
- Selecting specific files to commit (commits everything, matching `sdlc commit` behavior).

## Acceptance Criteria

1. `POST /api/git/commit` returns 200 with a `run_id` when there are changes to commit.
2. The agent generates a meaningful commit message from the diff.
3. The agent executes `sdlc commit --message "<msg>"` successfully.
4. An SSE event is emitted when the commit run completes.
5. The `GitStatusChip` refreshes after the commit completes.
6. If no changes exist, the agent run completes gracefully (no error).
7. If an agent is already running for the commit key, returns 409 Conflict.
