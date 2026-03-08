# Code Audit: Current Git Status Implementation

## What exists (milestone v1, released 2026-03-07)

### Backend: `crates/sdlc-server/src/routes/git.rs`
- **GET /api/git/status** — parses `git status --porcelain=v2 --branch`, returns rich JSON:
  `branch, dirty_count, staged_count, untracked_count, ahead, behind, has_conflicts, conflict_count, severity, summary`
- **No /api/git/commit endpoint exists** — route is not registered in `lib.rs`

### Frontend: `GitStatusChip.tsx` + `useGitStatus.ts`
- Polls `/api/git/status` every 10s, pauses when tab hidden
- Shows severity dot (green/yellow/red) + one-line summary
- **Commit button** exists (shows when `staged_count > 0`), POSTs to `/api/git/commit`
- **Bug:** commit button silently fails — the endpoint doesn't exist, error is only console.warn'd
- **No hover interaction** — collapsed mode uses native `title` tooltip, expanded mode shows inline text only

### Sidebar placement
- `GitStatusChip` is in the bottom utility section of `Sidebar.tsx`, always visible
- No click-through to a detail view

## Key gaps
1. Commit button is dead — no backend, no feedback, no agent run
2. No hover/touch detail panel — rich API data is wasted on a one-line summary
3. No actionable guidance — chip shows state but not what to do about it