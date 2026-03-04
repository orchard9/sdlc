# QA Plan: Fix ActivityTimeSeries Overflow in Agent Sidebar

## Manual Checks

1. Open the app with the Agent Activity sidebar visible.
2. Expand a completed run card (one that shows the `ActivityTimeSeries` chart).
3. Verify: the sidebar does **not** become horizontally scrollable.
4. Verify: the chart bars and x-axis labels are visible and fit within the card width.
5. Verify: the legend row wraps if needed rather than extending beyond the container.
6. Hover over chart bars — verify tooltip still appears correctly.
7. Resize the browser window to a narrow viewport — verify no overflow at any width.

## Regression Check

- Active run cards (showing live `AgentLog`) should be unaffected.
- Other sections of the sidebar (run list, collapsed cards) should look unchanged.
