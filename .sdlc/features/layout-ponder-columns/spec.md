# Spec: Ponder Layout — CONTEXT and ARTIFACTS Resizable Panels, Mobile Chat/Files/Team Tabs

## Problem

The current PonderPage detail pane has two layout limitations:

1. **Desktop**: The right-hand `WorkspacePanel` (ARTIFACTS) is a fixed 256 px column (`w-64`). Users
   cannot resize it. When an agent produces many artifacts or long content, the panel feels cramped.
   There is also no dedicated "CONTEXT" panel — team members, orientation, and the orientation strip
   are embedded inside `DialoguePanel`, not independently accessible.

2. **Mobile**: The workspace/artifacts are behind a toggle button that slides up a bottom-sheet.
   There is no tab bar offering direct one-tap access to Chat, Files (artifacts), and Team views.
   The current bottom-sheet approach is hidden by default and requires extra taps to discover.

## Goals

- On desktop (md+): add a drag-to-resize divider between the CHAT column and the ARTIFACTS column so
  the user can grow or shrink the workspace panel. Persist the width in `localStorage`.
- Add a third panel, CONTEXT, on desktop — a collapsible left-edge column containing the orientation
  strip, team row, and entry metadata — so the main chat area can be free of those elements.
- On mobile: replace the Files bottom-sheet toggle with a three-tab bar at the bottom of the detail
  pane: **Chat**, **Files**, **Team**. Each tab switches the visible panel; the currently active tab
  content occupies the full pane height.

## Out of Scope

- Resizable list pane (the entry list sidebar on the left) — remains fixed `w-72`.
- Any changes to `WorkspacePanel` internal content or artifact display.
- Any changes to `DialoguePanel` SSE, session loading, or chat logic.
- Server-side changes of any kind.

## Detailed Behaviour

### Desktop — Resizable ARTIFACTS Panel

- The right-hand workspace column becomes resizable via a vertical drag handle (a thin `<div>`
  between the chat area and the workspace column).
- Default width: 256 px (current `w-64`). Min: 160 px. Max: 50 % of the detail pane width.
- While dragging the handle, a semi-transparent overlay prevents iframe/input interference.
- Width is persisted in `localStorage` under the key `ponder_workspace_width`.
- On first load, if no stored value exists, default to 256 px.
- The drag handle uses CSS `cursor: col-resize` and provides a subtle hover highlight.

### Desktop — CONTEXT Panel

- A new collapsible left column inside `EntryDetailPane` (between the left entry list and the chat
  area) labelled **Context**.
- Contents (top to bottom):
  1. Entry metadata: slug, created-at, status badge.
  2. `TeamRow` component (rendered here instead of inside `DialoguePanel`).
  3. `OrientationStrip` component (rendered here instead of inside `DialoguePanel`).
- Default state: **collapsed** (icon-only strip ~32 px wide) so that the chat area gets maximum
  width on first load. An icon button toggles expansion to ~200 px wide.
- Collapse/expand state persisted in `localStorage` under key `ponder_context_open`.
- `DialoguePanel` must no longer render `TeamRow` or `OrientationStrip` internally when the desktop
  context panel is present (pass a prop `hideContextHeader?: boolean` to suppress them).

### Mobile — Three-Tab Bar

- Replace the existing "Files" toggle button in the entry header with a three-tab bar rendered at
  the bottom of the detail pane area (above the `InputBar` when Chat is active).
- Tabs: **Chat** (Lucide `MessageSquare`), **Files** (Lucide `Files`), **Team** (Lucide `Users`).
- Active tab state stored in component-local React state, defaulting to **Chat**.
- **Chat tab**: renders `DialoguePanel` (full height, same as today). `InputBar` is inside the
  dialogue panel so it remains visible.
- **Files tab**: renders `WorkspacePanel` directly in the content area (full height). No bottom-
  sheet. `InputBar` is hidden.
- **Team tab**: renders a simple panel showing `TeamRow` and `OrientationStrip` stacked. `InputBar`
  is hidden.
- The mobile Files bottom-sheet and its toggle button in the entry header are removed.
- The artifact count badge moves to the **Files** tab label (small pill number).
- On mobile, `DialoguePanel` renders `TeamRow` and `OrientationStrip` as normal (the desktop
  context panel is not present).

## Component Changes

| Component | Change |
|---|---|
| `PonderPage.tsx` / `EntryDetailPane` | Add desktop context panel (collapsible), drag-to-resize divider for workspace, mobile three-tab bar; remove Files bottom-sheet |
| `DialoguePanel.tsx` | Accept `hideContextHeader?: boolean`; skip rendering `TeamRow` + `OrientationStrip` when true |
| No other components change | `WorkspacePanel`, `TeamRow`, `OrientationStrip` are reused as-is |

## Acceptance Criteria

1. On desktop, dragging the divider between chat and workspace resizes the workspace panel; the new
   width survives a page refresh.
2. On desktop, clicking the Context toggle expands/collapses the context panel; state survives a
   page refresh.
3. On desktop, `TeamRow` and `OrientationStrip` appear only in the context panel, not in the chat
   stream.
4. On mobile, tapping the **Files** tab shows the artifact list without a bottom-sheet; tapping
   **Chat** returns to the chat view; tapping **Team** shows the team/orientation panel.
5. On mobile, the Files toggle button and bottom-sheet are gone.
6. No console errors or TypeScript errors introduced.
7. All existing ponder functionality (send message, SSE, commit, status change, advisory) continues
   to work.
