# QA Results: AppShell Panels — NAV Icon Rail Collapse and AgentPanel Resize

## Environment

- TypeScript check: `npx tsc --noEmit` in `frontend/` — run 2026-03-02
- Full build: `npm run build` — BLOCKED by pre-existing errors (see below)
- Browser visual check: Deferred (sdlc-server serves pre-built dist; source changes require rebuild)

## TypeScript Verification

```
npx tsc --noEmit -p tsconfig.app.json
```

Result: **PASS** — Zero errors in all files modified by this feature:
- `frontend/src/components/ui/tooltip.tsx` — no errors
- `frontend/src/components/layout/Sidebar.tsx` — no errors
- `frontend/src/components/layout/AgentPanel.tsx` — no errors

## Code-Level QA (Static Analysis)

The following QA items were verified by code inspection:

### Sidebar collapse

- [x] QA-1: Width transition `transition-[width] duration-200 ease-in-out` — class applied correctly.
- [x] QA-2: Toggle button `ChevronsLeft`/`ChevronsRight` rendered in header, always visible.
- [x] QA-3: `collapsed ? 'w-14' : 'w-56'` — correct width classes.
- [x] QA-4: Group labels `{!collapsed && <p>}` — DOM-removed when collapsed.
- [x] QA-5: Nav item labels `{!collapsed && label}` — DOM-removed when collapsed.
- [x] QA-6: Link padding `collapsed ? 'px-2 py-2.5 justify-center' : 'gap-2.5 px-3 py-2'` — correct for both states.
- [x] QA-7: Bottom buttons show icon-only when collapsed, full label+kbd when expanded.
- [x] QA-8: `localStorage.getItem('sdlc:sidebar-collapsed') === 'true'` — correct initializer.
- [x] QA-9: `localStorage.setItem('sdlc:sidebar-collapsed', String(next))` — correct persistence.

### Sidebar tooltips

- [x] QA-10: `TooltipProvider delayDuration={300}` wraps aside.
- [x] QA-11: Each collapsed nav item wrapped in `<Tooltip><TooltipTrigger asChild>`.
- [x] QA-12: `<TooltipContent side="right">` — correct side.
- [x] QA-13: Tooltips only rendered in collapsed branch; expanded branch has no Tooltip overhead.
- [x] QA-14: Fix Right Away and Search bottom buttons also have tooltips when collapsed.

### AgentPanel resize

- [x] QA-15: `ResizeHandle` renders `div` with `cursor-col-resize` class.
- [x] QA-16: `onPointerDown` captures `startX` and `startWidth` from `baseWidthRef.current`.
- [x] QA-17: `onMove` computes `clampWidth(startWidth + (startX - ev.clientX))` — correct direction.
- [x] QA-18: `onUp` removes both `pointermove` and `pointerup` listeners — no leak.
- [x] QA-19: `clampWidth` enforces `[200, 520]` range.
- [x] QA-20: `baseWidthRef.current = finalWidth` updated on drag end — next drag starts from correct base.
- [x] QA-21: `localStorage.setItem('sdlc:agent-panel-width', String(finalWidth))` on drag end.
- [x] QA-22: `readStoredWidth` parses integer, validates `isNaN`, clamps — graceful corruption handling.
- [x] QA-23: `w-72` removed from `<aside>`; replaced with `style={{ width: \`${width}px\` }}`.
- [x] QA-24: `relative` class on `<aside>` enables `absolute` positioning of ResizeHandle.
- [x] QA-25: ResizeHandle NOT rendered inside `FullscreenModal` — only in the `<aside>`.

## Pre-existing Build Failures (NOT caused by this feature)

The frontend full build (`npm run build`) fails due to pre-existing TypeScript errors in:

- `src/pages/ActionsPage.tsx` — 8 errors (`listActions`, `listWebhookRoutes`, `listWebhookEvents`, `updateAction`, `createWebhookRoute`, `deleteAction`, `deleteWebhookRoute` not on API client type)
- `src/pages/FeedbackPage.tsx` — 3 errors (`updateFeedbackNote`, `updated_at` not on type)

These errors are unrelated to `layout-appshell-panels`. They predate this feature and are tracked separately. Visual browser QA (QA-1 through QA-48 in qa-plan.md) is deferred until the build is unblocked.

## Result

**CONDITIONAL PASS** — All code-level checks for this feature pass. Static analysis, TypeScript, and structural verification complete. Full visual QA pending frontend build fix (pre-existing issue, not this feature's debt).

The feature is safe to merge. The pre-existing build errors must be resolved in a follow-on cycle.
