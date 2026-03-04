# Spec: ThreadsPage Mobile Layout Fix

## Problem

`ThreadsPage` currently renders a two-pane layout (thread list on the left, thread detail on the right) that is designed for desktop viewports. On mobile devices (screen width < `md` breakpoint, i.e. < 768px), both panes are shown side by side within a container that is already narrower than the combined width of the panes. The left pane has a hardcoded `w-[280px]` width and the right pane fills the remaining space — but on a 375px-wide phone screen this creates an unusable layout where:

1. The left list pane takes up most of the screen.
2. The right detail pane has essentially zero width or wraps badly.
3. There is no navigation mechanism to switch between list and detail view.

The `AppShell` mobile header already provides a "back" chevron when `isDetailView` is true (when the URL contains `/threads/:slug`). This means the routing-based navigation pattern is already partially in place — the `ThreadsPage` just needs to respect viewport width to show only one pane at a time.

## Goal

Make `ThreadsPage` work correctly on mobile by implementing a single-column navigation pattern:
- **List state** (`/threads`): show the thread list pane only.
- **Detail state** (`/threads/:slug`): show the thread detail pane only.
- Desktop behavior (≥ `md`) is unchanged: both panes are visible side by side.

## Scope

Changes are confined to `frontend/src/pages/ThreadsPage.tsx`. No new components, no routing changes, no backend changes.

## Behavior

### Mobile (< md breakpoint)

| URL | Visible | Hidden |
|-----|---------|--------|
| `/threads` | ThreadListPane | ThreadDetailPane |
| `/threads/:slug` | ThreadDetailPane | ThreadListPane |

Navigating from the list to a thread (selecting a row) navigates to `/threads/:slug`, which causes the list to hide and the detail to appear. The `AppShell` back button (already rendered for `/threads/:slug`) returns to `/threads`, which re-shows the list.

### Desktop (≥ md)

Both panes shown side by side. Existing layout is preserved exactly.

## Acceptance Criteria

1. On a 375px-wide viewport, visiting `/threads` shows only the list pane; the detail pane is not rendered or is hidden.
2. On a 375px-wide viewport, selecting a thread navigates to `/threads/:slug` and shows only the detail pane; the list pane is not rendered or is hidden.
3. On a 375px-wide viewport, pressing the back chevron (rendered by `AppShell`) returns to `/threads` and re-shows the list.
4. On desktop (≥ 768px), both panes are always visible and the layout is identical to the current implementation.
5. No desktop regression: existing thread creation, comment, delete, synthesize, and promote flows still work.

## Non-goals

- Bottom-tab navigation changes.
- Animation or slide transitions between panes.
- Any changes to `ThreadListPane`, `ThreadDetailPane`, or their sub-components.
- Any backend changes.
