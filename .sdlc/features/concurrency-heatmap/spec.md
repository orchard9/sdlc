# Spec: Concurrency Heatmap

## Summary

Add a cross-run concurrency heatmap to the Agent Activity panel and a dedicated Runs history page. The heatmap shows a time-bucketed view of idle vs. active agent runs across the run history, letting users spot parallelism opportunities and identify periods when agents were serialized unnecessarily.

## Problem

The current Agent Activity panel lists runs as cards — one at a time, in reverse-chronological order. There is no way to see whether multiple runs overlapped, whether there were long idle gaps between runs, or how much wall-clock time was "wasted" waiting for agents to finish serially when they could have run in parallel. Wave execution and milestone runs are the primary use cases where this matters most.

## Goal

A developer looking at the heatmap should be able to answer:
- Were any agents running in parallel during this session?
- Were there idle gaps longer than a few minutes where no agents were active?
- Which run types dominate execution time?

## Non-Goals

- Per-turn or per-tool breakdown (covered by the Run Activity Feed / telemetry)
- Live streaming updates within a run (existing RunActivityFeed already handles this)
- Editing or re-running from the heatmap (read-only view)

## User Stories

1. **As a developer reviewing a wave execution**, I want to see a horizontal timeline of all agent runs that occurred so I can verify they ran in parallel and confirm no serialization bottleneck exists.
2. **As a developer debugging a slow milestone**, I want to quickly spot idle gaps between runs and correlate them with specific run types to know where to investigate.
3. **As a developer using the Agent Activity panel**, I want a compact visual summary at the top of the panel that tells me at a glance how many runs are in the history and how concurrent they were.

## Functional Requirements

### Data Source

- Use the existing `GET /api/runs` endpoint which returns `RunRecord[]`.
- Each `RunRecord` has: `id`, `key`, `run_type`, `target`, `label`, `status`, `started_at`, `completed_at`.
- Runs where `completed_at` is null (actively running) use `now` as the end time.
- Runs without `started_at` are excluded from the heatmap.

### Heatmap Layout

- The heatmap is a 2D grid:
  - **X axis** — time, divided into uniform buckets. Bucket size adapts to the visible time range:
    - Range ≤ 10 minutes → 30-second buckets
    - Range ≤ 1 hour → 2-minute buckets
    - Range ≤ 6 hours → 10-minute buckets
    - Range > 6 hours → 30-minute buckets
  - **Y axis** — individual run lanes, one per run, sorted by `started_at` ascending (oldest at top).
- Each run lane shows a colored bar covering the buckets that fall within `[started_at, completed_at]`.
- Bars are colored by `run_type`:
  - `feature` → blue
  - `milestone_uat` → purple
  - `milestone_prepare` / `milestone_run_wave` → amber
  - `ponder` / `investigation` → teal
  - `vision_align` / `architecture_align` → green
  - All others → gray

### Concurrency Indicator

- Above the heatmap grid, show a single-row "concurrency strip" — a bar chart where each bucket's height encodes how many runs were active during that bucket (0 = empty / dark, max overlapping = brightest).
- This strip is the primary visual for spotting parallelism gaps.

### Scope and Placement

- **Agent Activity Panel**: Add a compact heatmap (collapsed by default, expandable with a chevron) above the run list. Compact mode shows only the concurrency strip (single row) plus a count label ("N runs, peak concurrency P").
- **Runs History Page** (new route `/runs`): Full-width heatmap with run lanes, concurrency strip, and time axis labels. Linked from the Agent Activity panel header.

### Interaction

- Hovering a bar segment shows a tooltip: `{label} — {run_type} — {duration}`.
- Clicking a run bar opens the existing run detail view (same as clicking a RunCard).
- The time axis auto-zooms to fit all runs with a small margin on each side.
- No manual zoom or pan in V1.

### Time Range Display

- Show the total wall-clock span in human-readable form above the heatmap (e.g., "Last 43 minutes", "Span: 2h 14m").
- Show the time axis as relative labels from the first run's start (e.g., "+0m", "+10m", "+20m").

## API Requirements

No new backend endpoints are required. The feature is entirely frontend-derived from `GET /api/runs`.

## Acceptance Criteria

1. The Agent Activity panel shows a compact concurrency strip when there are 2 or more runs in history.
2. The compact strip is hidden when there is 0 or 1 run (no meaningful concurrency to show).
3. Expanding the compact strip reveals the full heatmap with run lanes and time axis.
4. A `/runs` route exists in the frontend router and renders the full heatmap page.
5. Hovering a run bar shows a tooltip with label, run_type, and duration.
6. Clicking a run bar navigates to or opens the run detail for that run.
7. Colors are consistent with the run_type mapping defined above.
8. The concurrency strip correctly shows 0 for time buckets with no active runs and the count of overlapping runs for buckets with multiple concurrent runs.
9. All heatmap computation is pure client-side; no new API endpoints are added.
10. The feature is responsive — the heatmap scrolls horizontally on narrow screens rather than truncating bars.

## Out of Scope

- Filtering by run type or date range (future enhancement)
- Exporting the heatmap as an image
- Showing per-tool breakdown within a run bar
- Real-time streaming updates to the heatmap while a run is in progress (runs refresh on SSE `run_finished`)
