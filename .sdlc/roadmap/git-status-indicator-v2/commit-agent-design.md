# Design: Commit Button → Agent Run

## Problem
The commit button in GitStatusChip POSTs to `/api/git/commit` which doesn't exist. It silently fails. The user expects it to spawn an agent executing `/sdlc-commit` and show agent activity.

## Root Cause
The v1 milestone shipped the UI with a commit button wired to a non-existent endpoint. The button was aspirational — the backend was never implemented.

## Proposal: Commit via spawn_agent_run

### Backend
Add `POST /api/git/commit` in `git.rs`:
- Uses `spawn_agent_run` (the standard pattern per CLAUDE.md)
- Prompt: execute the `/sdlc-commit` skill — stage, commit, fetch origin, reconcile
- SSE event: `GitCommitCompleted { message: String }`
- Returns `{ run_id: "..." }` immediately (async)

### Frontend
1. Commit button calls POST /api/git/commit
2. Gets back a run_id
3. Navigates to the run detail view (or opens a mini activity panel)
4. Agent activity streams via SSE — user sees real-time progress
5. On completion, git status auto-refreshes

### Why agent, not direct git commit?
- `/sdlc-commit` does more than `git commit` — it stages intelligently, writes a good message, fetches origin, reconciles diverged history
- An agent can handle merge conflicts, decide what to stage, skip secrets
- Consistent with the product's agent-first ethos

## ⚑ Decided: commit button spawns an agent run
This matches the owner's explicit expectation and the product's architecture.

## ? Open: where does agent activity show?
Options:
1. Navigate to /runs/:id (existing run detail page) — works today, but context switch
2. Inline mini-panel in the hover tile — richer UX, more work
3. Toast notification with link to run — lightweight, low-friction

Leaning toward option 1 for v2 (works today) with option 2 as a future enhancement.