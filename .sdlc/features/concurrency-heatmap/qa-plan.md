# QA Plan: Concurrency Heatmap

## Test Strategy

All tests are frontend-only. The feature adds no backend endpoints. Testing covers:
1. Unit tests for the `useHeatmap` computation hook (pure logic)
2. Component rendering tests for `ConcurrencyStrip` and `RunsHeatmap`
3. Integration test for the `/runs` page route
4. Manual smoke tests for the Agent Activity panel compact strip

---

## Unit Tests: `useHeatmap` hook

File: `frontend/src/hooks/useHeatmap.test.ts`

| # | Test Case | Expected Result |
|---|-----------|-----------------|
| U1 | Empty runs array | Returns `HeatmapData` with empty `buckets`, empty `lanes`, `peakConcurrency=0` |
| U2 | Single run with start + end | Returns 1 lane, all bucket values 0 or 1, `peakConcurrency=1` |
| U3 | Two fully overlapping runs | All shared buckets have value 2, `peakConcurrency=2` |
| U4 | Two non-overlapping runs | Buckets between them are 0; each run region has value 1 |
| U5 | Run with `completed_at = null` | Run end time treated as `Date.now()`; lane extends to last bucket |
| U6 | Run missing `started_at` | That run is excluded from `lanes` and `buckets` |
| U7 | Range ≤ 10min | `bucketSizeMs = 30_000` (30 seconds) |
| U8 | Range between 10min and 1h | `bucketSizeMs = 120_000` (2 minutes) |
| U9 | Range between 1h and 6h | `bucketSizeMs = 600_000` (10 minutes) |
| U10 | Range > 6h | `bucketSizeMs = 1_800_000` (30 minutes) |
| U11 | `spanLabel` for 43-minute range | Returns `"43 minutes"` or equivalent human-readable string |
| U12 | `spanLabel` for 134-minute range | Returns `"2h 14m"` or equivalent |

---

## Component Tests: `ConcurrencyStrip`

File: `frontend/src/components/runs/ConcurrencyStrip.test.tsx`

| # | Test Case | Expected Result |
|---|-----------|-----------------|
| C1 | Renders with all-zero buckets | All bars render at minimum height (1px); no error thrown |
| C2 | Renders with mix of zero and non-zero buckets | Non-zero bars taller than zero bars |
| C3 | `height` prop sets SVG/container height | Rendered element has correct height attribute/style |

---

## Component Tests: `RunsHeatmap`

File: `frontend/src/components/runs/RunsHeatmap.test.tsx`

| # | Test Case | Expected Result |
|---|-----------|-----------------|
| H1 | `compact=false` with 2 runs | Renders `ConcurrencyStrip`, run lane rows, and time axis |
| H2 | `compact=true` with 2 runs | Renders `ConcurrencyStrip` and summary label; no lane rows; no time axis |
| H3 | Run label truncated in lane | Label column shows truncated text for long labels; does not overflow |
| H4 | Run type `feature` bar color | Bar element has blue color class applied |
| H5 | Run type `milestone_uat` bar color | Bar element has purple color class applied |
| H6 | `onRunClick` fires on bar click | Callback called with correct `RunRecord` argument |
| H7 | 0–1 runs renders empty state or nothing | No lanes rendered; no concurrency strip shown (or empty state text) |

---

## Integration Test: `/runs` Page

File: `frontend/src/pages/RunsPage.test.tsx`

| # | Test Case | Expected Result |
|---|-----------|-----------------|
| P1 | Renders at `/runs` route | Page title "Run History" visible; no 404 |
| P2 | With 0 runs | Empty state message rendered; no heatmap |
| P3 | With 1 run | Empty state message rendered; no heatmap (< 2 threshold) |
| P4 | With 2+ runs | `RunsHeatmap` rendered in full (non-compact) mode |
| P5 | Clicking a run bar | `focusRun` called on AgentRunContext |

---

## Integration Test: Agent Activity Panel Compact Strip

File: `frontend/src/components/layout/AgentPanel.test.tsx`

| # | Test Case | Expected Result |
|---|-----------|-----------------|
| A1 | Panel with 0–1 runs | No compact strip shown above RunList |
| A2 | Panel with 2+ runs | Compact `RunsHeatmap` rendered above RunList |
| A3 | "View full →" link present | Link with `href="/runs"` rendered in panel header |
| A4 | Clicking bar in compact strip | `focusRun` called with that run's id |

---

## Sidebar Navigation Test

| # | Test Case | Expected Result |
|---|-----------|-----------------|
| S1 | Sidebar "work" group includes "Run History" | Link to `/runs` present in sidebar |
| S2 | Navigating to `/runs` activates the sidebar item | Active state class applied to Run History link |

---

## Manual Smoke Test Checklist

Run after implementation:

- [ ] Start `sdlc ui` and open the app
- [ ] Trigger 3 agent runs from different pages (features, milestone prepare, ponder)
- [ ] Open Agent Activity panel — compact heatmap strip is visible with summary label
- [ ] "View full →" link navigates to `/runs`
- [ ] `/runs` page shows full heatmap with run lanes and time axis
- [ ] Hover over a bar segment — native tooltip shows concurrency count
- [ ] Click a run bar — agent panel opens and scrolls to that run card
- [ ] Verify run type colors match the spec (blue=feature, purple=UAT, amber=prepare, teal=ponder)
- [ ] Resize the browser to narrow width — heatmap page scrolls horizontally, no clipping
- [ ] Trigger a run while the `/runs` page is open — heatmap updates via SSE without page refresh

---

## Pass Criteria

- All unit tests (U1–U12) pass
- All component tests (C1–C7, H1–H7, P1–P5, A1–A4, S1–S2) pass
- Manual smoke checklist fully checked
- No TypeScript errors (`tsc --noEmit`)
- No ESLint errors
