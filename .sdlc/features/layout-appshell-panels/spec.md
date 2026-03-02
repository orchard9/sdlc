# Spec: AppShell Panels — NAV Icon Rail Collapse and AgentPanel Resize

## Overview

The AppShell currently has a fixed-width sidebar (224 px) and a fixed-width AgentPanel (288 px). Users cannot reclaim horizontal space when working in narrow viewports or on smaller laptop screens. This feature adds two independent layout controls:

1. **NAV icon rail collapse** — the sidebar collapses to a narrow icon-only rail (~52 px) when toggled. Clicking the rail icon expands it back to full width.
2. **AgentPanel drag-to-resize** — the AgentPanel can be resized by dragging its left border. Width persists across sessions via `localStorage`.

Together these give users fine-grained control over the three-column layout (sidebar | content | agent panel) without adding visual noise.

## Goals

- Let users reclaim 172 px of sidebar space with one click, while keeping all nav targets reachable via icons alone.
- Let users dial in an AgentPanel width that suits their screen (200–520 px range).
- Both preferences survive a page refresh.
- No regressions on mobile — BottomTabBar + mobile overlay sidebar are untouched.

## Non-Goals

- Animated sidebar transitions beyond a simple CSS width transition.
- Persisting the collapsed state to the server or to any SDLC state file — `localStorage` is sufficient.
- Resizing the main content column directly (it fills remaining space via flexbox).
- Resizing the sidebar beyond collapse vs. expanded (binary toggle only for the sidebar).

## User Stories

### NAV collapse

- As a user on a 13" laptop, I want to collapse the sidebar to icons so I can focus on content without losing nav access.
- As a user, when the sidebar is collapsed, I can still navigate by clicking the icon for each route — a tooltip with the label appears on hover.
- As a user, I can re-expand the sidebar with one click on the chevron/expand icon in the rail.
- My preference (collapsed vs. expanded) is saved and restored when I reload the page.

### AgentPanel resize

- As a user running a long agent task, I want to widen the AgentPanel so I can read longer run output without scrolling horizontally.
- As a user doing focused coding, I want a narrower AgentPanel to give more room to the main content area.
- I drag the left edge of the AgentPanel to resize it. The resize handle is 4 px wide, styled subtly, with a grab cursor.
- My preferred width is saved and restored on reload.

## Functional Requirements

### Sidebar collapse/expand

- **FR-1:** A toggle button in the sidebar header collapses/expands the sidebar.
  - Collapsed state: sidebar width ≈ 52 px, shows icon only (no label, no group labels).
  - Expanded state: sidebar width = 224 px (current), shows icon + label + group labels.
- **FR-2:** In collapsed state, each nav icon has a `title` attribute (or a Radix `Tooltip`) showing the nav item label on hover.
- **FR-3:** The bottom utility row (Fix Right Away, Search) shows icons only when collapsed; keyboard shortcuts hidden.
- **FR-4:** The toggle button itself is always visible in the sidebar header area regardless of collapsed state.
- **FR-5:** `localStorage` key `sdlc:sidebar-collapsed` stores `"true"` or `"false"`. Read on mount to initialize state.
- **FR-6:** Desktop only — mobile sidebar (slide-in overlay) is unaffected. BottomTabBar is unaffected.

### AgentPanel resize

- **FR-7:** A drag handle is rendered on the left border of the AgentPanel (4 px touch target, cursor: `col-resize`).
- **FR-8:** Dragging the handle changes the panel width within a clamped range: min 200 px, max 520 px.
- **FR-9:** `localStorage` key `sdlc:agent-panel-width` stores the panel width as a number. Default: 288 px (current `w-72`).
- **FR-10:** Width is applied as an inline style `width: Npx` replacing the hardcoded `w-72` class.
- **FR-11:** The resize interaction uses pointer events (`onPointerDown`, `onPointerMove`, `onPointerUp`) and `setPointerCapture` for reliable drag without losing the cursor.
- **FR-12:** When the AgentPanel is in fullscreen modal mode, the resize handle is not rendered.
- **FR-13:** The collapsed toggle button in `AppShell` (the `PanelRightOpen` button) is unaffected by the resize feature.

## Design Notes

- Sidebar collapse toggle: a `ChevronsLeft` / `ChevronsRight` icon button in the header, right-aligned. On collapse it shows `ChevronsRight`; on expand it shows `ChevronsLeft`.
- Group labels (`work`, `plan`, etc.) are hidden when collapsed (opacity-0 or display-none via conditional render).
- The icon rail background color matches the current sidebar (`bg-card`) with the same `border-r border-border`.
- Resize handle: `div` with `absolute left-0 inset-y-0 w-1 cursor-col-resize hover:bg-accent/60` inside a `relative` AgentPanel.
- No animation on AgentPanel resize — direct width update via state.
- Sidebar collapse uses a CSS `transition: width 200ms ease` for a smooth feel.

## Acceptance Criteria

- AC-1: Clicking the sidebar toggle collapses it to icon rail; clicking again re-expands it.
- AC-2: All nav routes remain accessible from the icon rail.
- AC-3: Tooltips appear on hover over each collapsed nav icon.
- AC-4: Sidebar collapse preference survives page reload.
- AC-5: Dragging the AgentPanel left border resizes it smoothly.
- AC-6: AgentPanel width stays within [200, 520] px during drag.
- AC-7: AgentPanel width preference survives page reload.
- AC-8: Mobile layout is visually unchanged (sidebar overlay, BottomTabBar, FAB all work as before).
- AC-9: No layout overflow or scroll artifacts introduced in the main content area.

## Out of Scope

- Making the sidebar resizable (only binary collapse/expand).
- Adding a resize handle to the sidebar.
- Keyboard shortcut to toggle sidebar.
- Server-side persistence of layout preferences.
