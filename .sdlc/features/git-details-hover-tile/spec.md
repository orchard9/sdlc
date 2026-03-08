# Spec: Git Details Hover Tile

## Summary

Add a popover/hover tile to the `GitStatusChip` in the sidebar that displays structured git status details when the user hovers or clicks the chip. The tile provides at-a-glance context about the working tree state, organized by category, with actionable guidance.

## Problem

The current `GitStatusChip` shows a severity dot and a single-line summary (e.g., "main -- 3 modified"). Users must leave the UI and open a terminal to understand *what* is modified, whether files are staged vs unstaged, and what action to take. The chip conveys urgency (red/yellow/green) but not enough detail to act on.

## Solution

A popover tile anchored to the `GitStatusChip` that appears on hover (desktop) or click (mobile/touch). The tile renders structured sections drawn from the existing `/api/git/status` response:

### Content Sections

1. **Branch & Tracking** -- branch name, ahead/behind counts relative to upstream
2. **Status Counts** -- dirty files, staged files, untracked files, conflict count -- each as a labeled row with count badge
3. **Severity Indicator** -- the computed severity (green/yellow/red) with a human-readable explanation of why that level was chosen
4. **Guidance Line** -- a short contextual suggestion based on severity:
   - Green: "Working tree clean. Ready to push." or "N commits ahead -- consider pushing."
   - Yellow: "Uncommitted changes detected. Stage and commit when ready."
   - Red: "Merge conflicts present -- resolve before continuing." or "Far behind upstream -- pull to catch up."

### Interaction

- **Desktop**: popover appears on hover over the `GitStatusChip`, dismissed on mouse-leave with a small delay (150ms) to prevent flicker
- **Click**: toggles the popover open/closed (works on both desktop and mobile)
- **Dismiss**: clicking outside the popover closes it
- The popover does not block sidebar navigation

### Data Source

No new API endpoints are needed. The tile consumes the same `useGitStatus()` hook data already fetched by `GitStatusChip`. The `GitStatus` interface already includes: `branch`, `dirty_count`, `staged_count`, `untracked_count` (from the API, though not yet in the TS interface), `ahead`, `behind`, `has_conflicts`, `conflict_count` (from the API), and `severity`.

### TypeScript Interface Update

The `GitStatus` TypeScript interface in `useGitStatus.ts` must be extended to include fields the API already returns but the frontend currently ignores:

- `untracked_count: number`
- `conflict_count: number`
- `summary: string`

### UI Component

A new `GitDetailsPopover` component in `frontend/src/components/layout/GitDetailsPopover.tsx` that:

- Accepts a `GitStatus` object as a prop
- Renders the structured sections above
- Uses existing Tailwind utility classes consistent with the sidebar design language (dark card background, muted text, border styling)
- Positions itself above or beside the chip depending on available space (CSS-only or a lightweight positioning approach)

### Integration

The `GitStatusChip` component wraps its existing content in a hover/click trigger that shows/hides the `GitDetailsPopover`. The chip itself remains unchanged visually.

## Out of Scope

- Per-file listing (covered by `git-status-directory-counts` feature)
- Commit actions from within the popover (covered by existing commit button and `git-commit-agent` feature)
- Git log or history display

## Acceptance Criteria

1. Hovering over the `GitStatusChip` in the sidebar shows a popover with branch, status counts, severity explanation, and guidance
2. The popover displays all status categories present in the API response (dirty, staged, untracked, conflicts, ahead, behind)
3. Zero counts are omitted -- only non-zero categories appear
4. The guidance line changes based on severity level
5. The popover dismisses on mouse-leave, outside click, or sidebar navigation
6. The popover works in both collapsed and expanded sidebar states
7. No new API calls -- reuses existing `useGitStatus()` data
8. The `GitStatus` TypeScript interface is updated to include `untracked_count`, `conflict_count`, and `summary`
