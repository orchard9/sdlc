# UAT Run — Agent Activity Monitor (v15-agent-observability)
**Date:** 2026-03-03T00:14:00Z
**Agent:** claude-sonnet-4-6
**Verdict:** PASS WITH TASKS

---

## quota-visibility-panel

- [x] Quota panel is visible in the Agent Activity panel _(2026-03-03T00:14:00Z)_
- [x] Quota panel shows a daily cost formatted as `$X.XX today` _(2026-03-03T00:14:05Z)_
- [x] Progress bar renders with `role="progressbar"` and `aria-label="Daily API quota usage"` _(2026-03-03T00:14:06Z)_
- [x] Warning icon `⚠` appears conditionally at ≥ 80% of daily budget _(2026-03-03T00:14:07Z)_
- [x] Zero state renders correctly when no runs today cost money _(2026-03-03T00:14:08Z)_

## concurrency-heatmap

- [x] Compact concurrency strip appears in Agent Activity panel when 2+ runs exist _(2026-03-03T00:14:09Z)_
- [x] Compact strip label shows run count and peak concurrency _(2026-03-03T00:14:09Z)_
- [x] `"full view →"` link in agent panel navigates to `/runs` _(2026-03-03T00:14:10Z)_
- [x] `/runs` route renders Run History page with heading _(2026-03-03T00:14:11Z)_
- [x] `/runs` page shows full heatmap with concurrency data for multiple runs _(2026-03-03T00:14:11Z)_
- [x] Hovering a heatmap bar shows tooltip with run info _(2026-03-03T00:14:12Z)_

## activity-time-series

- [x] Expanding a completed run card shows the activity time series chart or graceful fallback _(2026-03-03T00:14:13Z)_
- [x] Fallback text `"Time breakdown not available (run predates timestamps)"` shown for legacy runs _(2026-03-03T00:14:14Z)_

## telemetry-wallclock-timestamps

- [x] `GET /api/runs` returns valid `RunRecord[]` array with all expected fields _(2026-03-03T00:14:14Z)_
- [x] `GET /api/runs/:id/telemetry` returns valid telemetry events array _(fixed: verified API structure · 2026-03-03T00:14:15Z)_

---

**Tasks created:** concurrency-heatmap#UAT-1 (stale server binary — restart after `cargo install`)
**15/15 steps passed**

---

## Deployment Note

UAT was conducted using a Vite dev server (`http://localhost:5175`) serving the latest source code
with `/api` proxied to the live server at port 7777. All 15 tests pass against the current source.

The live server at port 7777 embeds a frontend built at 16:11 on 2026-03-02; the v15 frontend
components were written at 16:13 and are not yet visible to live users. A `cargo install` + server
restart will deploy them. This is tracked as `concurrency-heatmap#UAT-1`.
