# Spec: Git Status Chip

## Overview

A compact React component displayed in the sidebar's bottom utility area that provides at-a-glance git repository health. The chip polls `GET /api/git/status` (provided by the `git-status-api` feature) and renders a color-coded severity indicator (green/yellow/red) with summary text. When staged changes exist, a commit button shortcut is displayed.

## Dependencies

- **git-status-api** — provides `GET /api/git/status` returning branch name, dirty-file count, staged-file count, ahead/behind counts, conflict presence, and composite severity (`green | yellow | red`).

## User Stories

1. **As a developer**, I want to see the git health of my workspace at a glance without navigating away from my current page, so I can catch uncommitted work or diverged branches early.
2. **As a developer**, I want a quick-commit shortcut when I have staged changes, so I can commit without switching context.
3. **As a developer**, I want the status to update automatically so I always see current information.

## Functional Requirements

### FR-1: Severity Indicator
- Display a colored dot or badge: green (clean), yellow (dirty but no conflicts), red (conflicts or significant divergence).
- Color is derived from the `severity` field in the API response.

### FR-2: Summary Text
- When expanded (sidebar not collapsed): show branch name and a short summary (e.g., "main - clean", "main - 3 modified", "main - 2 conflicts").
- When collapsed: show only the colored dot icon; full text available via tooltip.

### FR-3: Commit Button
- Visible only when `staged_count > 0` in the API response.
- Clicking the commit button triggers a commit action (POST to a commit endpoint or opens a commit dialog — design will specify).

### FR-4: Polling
- Poll `GET /api/git/status` on a configurable interval (default: 10 seconds).
- Also re-fetch on window focus to catch changes made in external editors/terminals.
- Handle API errors gracefully — show a muted/disabled state, do not crash.

### FR-5: Sidebar Integration
- Rendered in the sidebar's bottom utility section (alongside Search, Fix Right Away, Ask Code).
- Respects the `collapsed` prop — shows icon-only when collapsed, full chip when expanded.
- Consistent styling with existing sidebar utility buttons.

### FR-6: Loading & Error States
- Initial load: show a subtle loading skeleton or muted icon.
- API unavailable: show a grey/disabled dot with "Offline" tooltip. No error toasts.

## Non-Functional Requirements

- **NFR-1**: Component must not cause layout shift in the sidebar during loading or state transitions.
- **NFR-2**: Polling must be paused when the browser tab is not visible (use `document.hidden` or `visibilitychange`).
- **NFR-3**: Must work with both light and dark themes using existing design tokens.

## Out of Scope

- The git status API endpoint itself (handled by `git-status-api` feature).
- Commit message composition UI (future feature — the commit button will use a simple default message or delegate to an existing commit flow).
- Branch switching or git operations beyond commit.

## Acceptance Criteria

1. Chip renders in the sidebar bottom utility area with correct severity color.
2. Summary text shows branch name and status when sidebar is expanded.
3. Chip collapses to icon-only when sidebar is collapsed, with tooltip.
4. Commit button appears only when staged files exist.
5. Status auto-refreshes on the configured interval and on window focus.
6. Graceful degradation when API is unavailable.
