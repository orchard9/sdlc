# Code Review: AppShell Panels — NAV Icon Rail Collapse and AgentPanel Resize

## Summary of Changes

Three files changed, one new file created:

| File | Change |
|---|---|
| `frontend/src/components/ui/tooltip.tsx` | New — shadcn-style Tooltip wrapper around `@radix-ui/react-tooltip` |
| `frontend/src/components/layout/Sidebar.tsx` | Added `collapsed` state, toggle button, conditional label/group rendering, Tooltip wrapping for collapsed icons, `localStorage` persistence |
| `frontend/src/components/layout/AgentPanel.tsx` | Added `width` state, `baseWidthRef`, `ResizeHandle` component, inline style width, `localStorage` persistence |
| `frontend/package.json` | Added `@radix-ui/react-tooltip` dependency |

No Rust, server, or test files were modified. This is a pure frontend change.

## Correctness Review

### Sidebar collapse

- State initializer reads `localStorage` once on mount via lazy initializer function — correct, avoids re-reading on every render.
- `toggleCollapsed` writes `String(next)` — produces `"true"` / `"false"`, matching the initializer comparison `=== 'true'`. Consistent.
- Width transition: `transition-[width] duration-200 ease-in-out` — scoped to width only, no unintended property transitions.
- `shrink-0` on `<aside>` prevents flexbox from collapsing the sidebar when content area is wide. Correct.
- `overflow-hidden` on `<aside>` prevents content from spilling out during the CSS width transition. Correct.
- Group labels conditionally rendered with `{!collapsed && (...)}` — not just hidden, actually removed from DOM. No stale text visible.
- Nav item labels `{!collapsed && label}` — same pattern, clean.
- Link `key` prop: expanded state uses `<div key={path}>`, collapsed state uses `<Tooltip key={path}>` — both carry the correct key. No duplicate key issues.

### Tooltip behavior

- `TooltipProvider` wraps the entire `<aside>` — correct placement, avoids nesting providers.
- `delayDuration={300}` — reasonable hover delay, not instant (avoids tooltip flicker on mouse pass).
- `<TooltipContent side="right">` — correct side for a left sidebar.
- Tooltips only rendered in collapsed branch; the expanded branch has no Tooltip wrappers — no unnecessary tooltip overhead when expanded.
- `<TooltipTrigger asChild>` used correctly — passes the render prop through to the child element without adding an extra DOM node.

### AgentPanel resize

- `ResizeHandle` component is file-local (not exported) — clean encapsulation.
- Pointer events used correctly: `onPointerDown` on the div, then `window.addEventListener` for `pointermove`/`pointerup`. This pattern is correct for drag — pointer capture via window ensures movement outside the element is tracked.
- `startWidth` captured in closure at drag start from `baseWidthRef.current` — correctly frozen for the duration of the drag. Delta arithmetic `startX - ev.clientX` is correct: moving left increases panel width.
- `onUp` removes both listeners — no listener leak.
- `clampWidth` applied in both `onMove` (for live width) and `onUp` (for final persistence value) — consistent.
- `baseWidthRef.current = finalWidth` updated in `handleResizeEnd` — next drag correctly starts from the last settled width.
- `w-72` Tailwind class removed; replaced with `style={{ width: \`${width}px\` }}` — no class/style conflict.
- `relative` class added to `<aside>` — required for `absolute` positioning of `ResizeHandle`. Correct.
- `ResizeHandle` not rendered inside `FullscreenModal` — correct per spec.

### localStorage

- `sdlc:sidebar-collapsed`: written as `"true"` / `"false"`, read and compared correctly.
- `sdlc:agent-panel-width`: written as integer string, parsed with `parseInt`, validated with `isNaN`, clamped. Handles corrupt values gracefully.
- Both keys use the `sdlc:` namespace prefix — consistent with project conventions.

## Issues Found and Addressed

None — the implementation matches the spec and design documents exactly. TypeScript reports zero errors (`npx tsc --noEmit`).

## Observations (Non-blocking)

1. **`baseWidthRef` initialized from `width` state:** `useRef<number>(width)` captures the initial state value at render time. This is correct — the ref is only used as a "start of drag" snapshot, and is updated on every drag end. No stale ref concern.

2. **Sidebar collapse does not affect mobile overlay:** The `collapsed` state is scoped to the `Sidebar` component. The mobile overlay is driven by `sidebarOpen` in `AppShell`. The two are fully independent — confirmed by code inspection of `AppShell.tsx`.

3. **AppShell structural regression check:** `AppShell.tsx` was not modified. The sidebar wrapper div uses `md:translate-x-0` and `md:static` — these are unaffected by the sidebar's internal `collapsed` state change.

4. **`@radix-ui/react-tooltip` security posture:** Installed from the official `@radix-ui` scope, consistent with `@radix-ui/react-slot` already in use. No new attack surface introduced.

## Verdict

APPROVED. All acceptance criteria from the spec are satisfied. No blocking issues.
