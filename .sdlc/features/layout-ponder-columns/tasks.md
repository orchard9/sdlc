# Tasks: Ponder Layout — CONTEXT and ARTIFACTS Resizable Panels, Mobile Chat/Files/Team Tabs

## T1 — Add `hideContextHeader` prop to `DialoguePanel`

File: `frontend/src/components/ponder/DialoguePanel.tsx`

- Add `hideContextHeader?: boolean` to the `Props` interface.
- Wrap the `TeamRow` render with `{!hideContextHeader && entry.team.length > 0 && <TeamRow ... />}`.
- Wrap the `OrientationStrip` render with `{!hideContextHeader && <OrientationStrip ... />}`.
- No other logic changes.

## T2 — Add `ResizeDivider` inline component to `PonderPage.tsx`

File: `frontend/src/pages/PonderPage.tsx`

- Add `ResizeDivider` inline component (as described in design.md).
- Props: `onWidthChange: (w: number) => void`, `minWidth: number`, `maxFraction: number`.
- Uses `onMouseDown` to attach `mousemove`/`mouseup` listeners on `document`.
- CSS: `w-1.5 shrink-0 cursor-col-resize` with hover highlight.
- Cleans up event listeners on `mouseup`.

## T3 — Add `ContextPanel` inline component to `PonderPage.tsx`

File: `frontend/src/pages/PonderPage.tsx`

- Add `ContextPanel` inline component.
- Props: `open`, `onToggle`, `slug`, `status`, `team`, `orientation`.
- Collapsed: ~32 px wide, shows only `ChevronRight` / `ChevronLeft` toggle button.
- Expanded: ~200 px wide, shows toggle button + slug + `StatusBadge` + `TeamRow` + `OrientationStrip`.
- Uses `transition-all duration-200` for smooth open/close animation.
- Imports `ChevronRight`, `ChevronLeft` from lucide-react (already available).

## T4 — Add `TeamContextPanel` inline component to `PonderPage.tsx`

File: `frontend/src/pages/PonderPage.tsx`

- Simple mobile-only panel: renders `TeamRow` and `OrientationStrip` with padding.
- Used as the **Team** tab content on mobile.

## T5 — Add `MobileTabButton` inline component to `PonderPage.tsx`

File: `frontend/src/pages/PonderPage.tsx`

- Props: `tab: MobileTab`, `active: boolean`, `badge?: number`, `onClick: () => void`.
- Shows icon (`MessageSquare`, `Files`, `Users`) and label text.
- Shows numeric badge pill on Files tab when `badge > 0`.
- Active state uses `text-primary` / `bg-accent/50`; inactive uses `text-muted-foreground`.

## T6 — Wire desktop state and layout in `EntryDetailPane`

File: `frontend/src/pages/PonderPage.tsx`

- Add state:
  - `contextOpen` (boolean, initialized from `localStorage.getItem('ponder_context_open') !== 'false'`)
  - `workspaceWidth` (number, initialized from stored value or 256)
- Persist both to `localStorage` on change (via inline handler or `useEffect`).
- Restructure the desktop content area (`hidden md:flex`) to include:
  1. `ContextPanel` (left of chat)
  2. `DialoguePanel` with `hideContextHeader` prop
  3. `ResizeDivider`
  4. Workspace column with inline `style={{ width: workspaceWidth }}`
- Remove the old fixed `w-64` workspace `div`.

## T7 — Wire mobile tab bar in `EntryDetailPane`

File: `frontend/src/pages/PonderPage.tsx`

- Add state: `mobileTab: MobileTab` (default `'chat'`).
- Add mobile content area (`md:hidden`):
  - Conditionally renders `DialoguePanel`, `WorkspacePanel`, or `TeamContextPanel` based on `mobileTab`.
- Add mobile tab bar (`md:hidden border-t border-border flex`):
  - Three `MobileTabButton` components.
  - Files button receives `badge={entry.artifacts.length}`.
- Remove the existing mobile Files toggle button from the entry header.
- Remove the existing mobile bottom-sheet overlay + sheet `div`s.
- Remove `mobileWorkspaceOpen` state (no longer needed).

## T8 — Cleanup and type-check

- Remove `mobileWorkspaceOpen` state and all references from `EntryDetailPane`.
- Remove unused imports (e.g. `Files` icon if no longer used in the header).
- Add any new lucide-react imports (`MessageSquare`, `ChevronRight`, `ChevronLeft`) to the import line.
- Verify TypeScript compiles without errors: `cd frontend && npx tsc --noEmit`.
- Verify no lint errors: `cd frontend && npm run lint` (or equivalent).
