# UAT Run — Agent Activity Monitor (v15-agent-observability)
**Date:** 2026-03-02T23:00:00Z
**Verdict:** Failed
**Tests:** 5/15
**Tasks created:** quota-visibility-panel#T1, concurrency-heatmap#T2

## Results
Suite: v15-agent-observability — Acceptance Tests
Duration: ~30s
Passed: 5 | Failed: 10 | Skipped: 0

## Root Cause
The server binary was compiled at 16:11 (before the v15 frontend changes were committed).
Source files `AgentPanel.tsx` and `RunsPage.tsx` were modified at 16:13–16:14, after the
binary was compiled. `rust-embed` embeds frontend assets at compile time, so the running
process serves the stale UI.

**Fix applied:** Rebuilt `frontend/dist` (npm run build) and installed new `~/.cargo/bin/sdlc`
binary. A server restart is required to serve the new frontend.

## Failures
| Test | Classification | Resolution |
|---|---|---|
| quota panel is visible in the Agent Activity panel | Code bug (stale binary) | Task quota-visibility-panel#T1 |
| quota panel shows a daily cost in dollars | Code bug (stale binary) | Task quota-visibility-panel#T1 |
| quota panel progress bar has correct ARIA role | Code bug (stale binary) | Task quota-visibility-panel#T1 |
| quota panel shows warning icon at or above 80% usage | Code bug (stale binary) | Task quota-visibility-panel#T1 |
| quota panel renders zero state correctly when no runs today cost money | Code bug (stale binary) | Task quota-visibility-panel#T1 |
| compact concurrency strip appears in agent panel when 2+ runs exist | Code bug (stale binary) | Task concurrency-heatmap#T2 |
| compact heatmap shows run count and peak concurrency label | Code bug (stale binary) | Task concurrency-heatmap#T2 |
| "full view" link in agent panel navigates to /runs | Code bug (stale binary) | Task concurrency-heatmap#T2 |
| /runs route renders Run History page | Code bug (stale binary) | Task concurrency-heatmap#T2 |
| /runs page shows full heatmap with concurrency data for multiple runs | Code bug (stale binary) | Task concurrency-heatmap#T2 |

## Passed Tests
| Test | Notes |
|---|---|
| hovering a heatmap bar on /runs shows a tooltip with run info | Best-effort assertion passes |
| expanding a completed run card shows the activity time series chart or fallback | SVG/fallback present |
| time series fallback text appears for runs without timestamps | Permissive assertion |
| telemetry API response includes timestamp field | Backend verified via HTTP |
| GET /api/runs returns valid RunRecord array with expected fields | Backend verified via HTTP |
