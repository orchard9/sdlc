# Tasks: AppShell Panels — NAV Icon Rail Collapse and AgentPanel Resize

## Task List

### T1 — Verify/install Tooltip component

Check whether `frontend/src/components/ui/tooltip.tsx` exists (shadcn/ui Tooltip). If absent, install it via `npx shadcn@latest add tooltip` from `frontend/`. Confirm `TooltipProvider`, `TooltipTrigger`, `TooltipContent` are exported.

**Acceptance:** `frontend/src/components/ui/tooltip.tsx` exists with the standard shadcn exports.

---

### T2 — Sidebar collapse: state, toggle button, width transition

In `frontend/src/components/layout/Sidebar.tsx`:

1. Add `collapsed` state initialized from `localStorage.getItem('sdlc:sidebar-collapsed') === 'true'`.
2. Add `toggleCollapsed` that flips state and writes to `localStorage`.
3. Add `ChevronsLeft` / `ChevronsRight` imports from `lucide-react`.
4. Change `<aside className="w-56 ...">` to a dynamic width with `transition-[width] duration-200 ease-in-out` — `w-14` when collapsed, `w-56` when expanded.
5. Add the toggle button to the header, right-aligned: `ChevronsRight` icon when collapsed, `ChevronsLeft` when expanded.
6. Conditionally hide the `<h1>` / `<p>` wordmark when collapsed.

**Acceptance:** Clicking the toggle button animates the sidebar between 224 px and 56 px. Preference persists across reload.

---

### T3 — Sidebar collapse: hide group labels and item labels

In `Sidebar.tsx`:

1. Wrap group label `<p>` in `{!collapsed && (...)}` — hidden when collapsed.
2. On each nav link: remove label text when collapsed. Adjust link padding: `collapsed ? 'px-3 py-2.5 justify-center' : 'px-3 py-2'`.
3. On bottom utility buttons: hide label `<span>` and `<kbd>` when collapsed. Add `justify-center` class when collapsed.

**Acceptance:** Collapsed sidebar shows icons only, centered. No text visible. No layout overflow.

---

### T4 — Sidebar collapse: tooltips for collapsed nav items

In `Sidebar.tsx`:

1. Import `Tooltip`, `TooltipTrigger`, `TooltipContent`, `TooltipProvider` from `@/components/ui/tooltip`.
2. Wrap the nav block in `<TooltipProvider>`.
3. Wrap each nav `<Link>` in `<Tooltip><TooltipTrigger asChild>...</TooltipTrigger>{collapsed && <TooltipContent side="right">{label}</TooltipContent>}</Tooltip>`.
4. Similarly wrap the bottom utility buttons (Fix Right Away, Search) with tooltips showing their labels when collapsed.

**Acceptance:** Hovering any icon in the collapsed rail shows a right-side tooltip with the nav item label.

---

### T5 — AgentPanel: ResizeHandle component and width state

In `frontend/src/components/layout/AgentPanel.tsx`:

1. Add constants: `MIN_WIDTH = 200`, `MAX_WIDTH = 520`, `DEFAULT_WIDTH = 288`.
2. Add `width` state initialized from `localStorage.getItem('sdlc:agent-panel-width')` (parse, clamp, default to 288).
3. Add a `baseWidth` ref that tracks the panel width at the start of each drag.
4. Implement `ResizeHandle` function component (can be file-local):
   - Renders a `<div>` with `absolute left-0 inset-y-0 w-1 cursor-col-resize hover:bg-accent/60 transition-colors z-10`.
   - `onPointerDown`: records `startX`, adds `pointermove` and `pointerup` listeners on `window`.
   - `onPointerMove`: computes delta = `startX - e.clientX`, clamps new width, calls `onWidthChange(newWidth)`.
   - `onPointerUp`: removes listeners, calls `onResizeEnd(finalWidth)` to persist to `localStorage`.
5. Replace `w-72` class on `<aside>` with `style={{ width: \`${width}px\` }}` and `relative` class.
6. Render `<ResizeHandle ... />` as first child of `<aside>` (not inside `FullscreenModal`).

**Acceptance:** Dragging the left edge of the AgentPanel resizes it smoothly within [200, 520] px. Width persists across reload.

---

### T6 — AppShell: ensure no structural regression

Review `frontend/src/components/layout/AppShell.tsx`:

1. Confirm the sidebar div wrapper still works correctly with the new self-contained `collapsed` state in `Sidebar`. No changes expected, but verify `md:translate-x-0` and mobile overlay still function.
2. Confirm the `PanelRightOpen` toggle button (`!panelOpen` branch) is unaffected.
3. Run a quick visual check at `md` breakpoint boundary.

**Acceptance:** No regressions in mobile overlay behavior, `BottomTabBar`, `AgentPanelFab`, or the `PanelRightOpen` button.

---

### T7 — TypeScript / lint clean-up

1. Run `cd frontend && npx tsc --noEmit` — fix any type errors introduced.
2. Run `cd frontend && npm run lint` (if configured) — fix any lint warnings.

**Acceptance:** Zero TypeScript errors. Zero new lint warnings.

---

## Implementation Order

T1 → T2 → T3 → T4 (sidebar track, in order)
T5 (AgentPanel, independent of sidebar track)
T6 (after T2–T5)
T7 (final)

T1 and T5 can start in parallel.
