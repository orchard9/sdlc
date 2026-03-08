# Milestone Breakdown

## Milestone 1: Git Status Indicator (Quick Win)

**Goal:** Glanceable git health from every screen.

- Add `GET /api/git/status` endpoint returning composite GitStatus
- Status chip in sidebar header (next to project name)
- Green state: fun micro-animation or celebration icon
- Yellow state: commit button that triggers `/sdlc-commit`
- Red state: warning badge with action guidance
- Poll local status every 5s, remote every 60s
- SSE event on agent-driven commits for instant refresh

**Complexity:** Small. 1 backend endpoint, 1 UI component.

## Milestone 2: Git Page — File Browser

**Goal:** Visual overview of workspace files with git status.

- New "Git" section in sidebar under `integrate`, above `network`
- Master-detail layout: file list (left) + content area (right)
- File list shows path + status badge (M/A/D/R/??)
- Filters: All files, Modified only (default), Staged, Untracked
- Toggle: flat view (full paths) / tree view (hierarchical)
- Build file list component in-house (not a library)
- Keyboard nav: j/k between files, Enter to select

**Complexity:** Medium. Custom file browser, API for full file listing.

## Milestone 3: Git Page — Diff Viewer

**Goal:** Review changes visually before committing.

- `GET /api/git/diff?path=<file>` endpoint returning unified diff
- Side-by-side diff when viewport > 900px, unified when narrower
- Use `@git-diff-view/react` library for rendering
- Syntax highlighting via the library built-in support
- Color coding: content changes (strong) vs whitespace (muted)
- Binary file handling: show "binary changed" with size delta
- Large diff truncation: cap at ~2000 lines with "show more"
- Keyboard nav: [/] to jump between hunks

**Complexity:** Medium. Library integration, responsive breakpoint logic.

⚑ Decided: Three milestones, progressive delivery
⚑ Decided: Modified-only as default filter (user perspective)
⚑ Decided: Build file browser in-house, use library for diff viewer
? Open: What "fun" thing to show for green state
? Open: Should staging (git add) be supported in the UI or just viewing?