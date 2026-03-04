# Code Review: Fix ActivityTimeSeries Overflow in Agent Sidebar

## Changes Reviewed

### `frontend/src/components/runs/ActivityTimeSeries.tsx`

**Change 1:** Outer container div: `className="relative w-full select-none"` → `"relative w-full select-none overflow-hidden"`
- Correct. This clips any content that overflows the measured container width, including SVG content and the tooltip container. The tooltip uses `absolute` positioning and is rendered inside this div, so it still renders but will be clipped at the container edge — acceptable since the tooltip already clamps its left position with `Math.min(tooltip.x, chartWidth - 120)`.

**Change 2:** SVG `className="overflow-visible"` → `"overflow-hidden"`
- Correct. The `overflow-visible` was the primary offender allowing x-axis labels to bleed beyond the SVG bounding box. With `overflow-hidden`, labels that would extend past the SVG edge are clipped. The SVG width is set to `chartWidth` which is tracked via `ResizeObserver`, so labels should fit within the SVG in steady state anyway.

**Change 3:** Legend row: `"flex items-center gap-3 mb-1"` → `"flex flex-wrap items-center gap-x-3 gap-y-1 mb-1"`
- Correct. Allows wrapping on narrow containers so legend items don't force horizontal overflow. `gap-x-3 gap-y-1` maintains horizontal spacing and adds a small vertical gap on wrap.

### `frontend/src/components/layout/RunCard.tsx`

**Change 4:** Expanded panel wrapper: `"px-3 pb-3"` → `"px-3 pb-3 overflow-hidden"`
- Correct. Provides the outermost containment layer so neither the chart nor the `RunActivityFeed` below it can cause the card or sidebar to scroll horizontally.

## Findings

- No logic changes — purely additive CSS classes.
- No regressions possible on active run cards (`AgentLog` is rendered in the same wrapper but is a scrollable text log with no horizontal overflow concern).
- Tooltip clamping (`Math.min(tooltip.x, chartWidth - 120)`) already handles tooltip positioning correctly within the container.
- No TypeScript changes; no test changes needed.

**Verdict: APPROVED — ready to advance.**
