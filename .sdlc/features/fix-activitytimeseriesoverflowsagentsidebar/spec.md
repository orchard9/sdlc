# Spec: Fix ActivityTimeSeries Overflow in Agent Sidebar

## Problem

When a run card is expanded in the Agent Activity sidebar, the `ActivityTimeSeries` chart and its legend row overflow horizontally, pushing the sidebar beyond its fixed width and making it scrollable. This breaks the layout and degrades UX.

**Root cause (two independent issues):**

1. `ActivityTimeSeries.tsx` — the `<svg>` element has `className="overflow-visible"`, which allows x-axis labels and bar content to render outside the SVG's reported bounding box. Since the component uses `ResizeObserver` to track the container width and feeds it into `chartWidth`, any momentary width mismatch or floating-point rounding can cause text labels to bleed right.

2. `ActivityTimeSeries.tsx` — the legend row (`<div className="flex items-center gap-3 mb-1">`) has no overflow constraint. With 5 items ("Activity" label + 4 color keys), it can exceed the sidebar panel width on narrow viewports.

3. `RunCard.tsx` — the expanded section (`<div className="px-3 pb-3">`) has no `overflow-hidden` on its wrapper, so neither the chart nor the feed is clipped to the card boundary.

## Acceptance Criteria

- [ ] Expanding a completed run card in the Agent Activity sidebar does **not** cause horizontal overflow or scroll on the sidebar.
- [ ] The `ActivityTimeSeries` chart renders fully within the available width at any sidebar width.
- [ ] The legend row wraps or truncates gracefully rather than extending beyond the container.
- [ ] Tooltip still appears correctly (it may overflow its parent, which is intentional via `absolute` positioning).
- [ ] No visual regression: bars, x-axis labels, and legend colors remain visible and readable.

## Proposed Changes

### 1. `frontend/src/components/runs/ActivityTimeSeries.tsx`

- Remove `className="overflow-visible"` from the `<svg>` element (or replace with `overflow-hidden`).
- Add `overflow-hidden` to the outer container `<div>` (the one with `ref={containerRef}`), so the whole chart assembly is clipped: `className="relative w-full select-none overflow-hidden"`.
- Add `flex-wrap` to the legend row so items wrap on very narrow containers: `className="flex flex-wrap items-center gap-x-3 gap-y-1 mb-1"`.

### 2. `frontend/src/components/layout/RunCard.tsx`

- Add `overflow-hidden` to the expanded panel wrapper: change `className="px-3 pb-3"` to `className="px-3 pb-3 overflow-hidden"`. This is the final containment layer.

## Out of Scope

- Changing the chart's bucket count or visual design.
- Modifying the sidebar's fixed width (acceptable to leave as-is; overflow fix is sufficient).
- Tooltip z-index or positioning changes beyond what the fix necessitates.
