# Scope: git-status-indicator-v2 Features

## Feature breakdown

### 1. git-details-hover-tile (UI)
- Hover/touch popover on GitStatusChip
- Structured display of all git status fields
- Actionable guidance derived from state
- Relocate commit button into the tile
- Mobile: tap-to-toggle

### 2. git-commit-agent (Backend + UI)
- POST /api/git/commit endpoint using spawn_agent_run
- Executes /sdlc-commit skill via agent
- Returns run_id, streams progress via SSE
- Frontend navigates to run detail on click
- Git status auto-refreshes on completion

### 3. git-status-files-api (Backend, optional)
- Extend GET /api/git/status to include file lists (dirty_files, staged_files)
- Cap at first N files + total count
- Enables the hover tile to show specific file names
- Could be a separate endpoint: GET /api/git/files

## Dependency order
1. git-commit-agent (fixes the broken button — highest priority, direct user pain)
2. git-details-hover-tile (new capability, uses existing API data)
3. git-status-files-api (enhancement to the tile, optional for v2)

## What's NOT in scope
- Full git page (that's git-page-shell, a separate milestone)
- Diff viewer, file browser, commit history — all separate milestones already exist
- Push-to-remote button (future)
- Branch switching UI (future)