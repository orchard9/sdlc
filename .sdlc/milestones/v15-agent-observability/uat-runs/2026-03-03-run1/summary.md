# UAT Run — Agent Activity Monitor (v15-agent-observability)
**Date:** 2026-03-03T00:14:00Z
**Verdict:** PassWithTasks
**Tests:** 15/15
**Tasks created:** concurrency-heatmap#UAT-1

## Results
Suite: v15-agent-observability — Acceptance Tests
Duration: 9406ms
Passed: 15 | Failed: 0 | Skipped: 0

## Investigation Finding

**Root cause:** The running `sdlc` server binary at port 7777 was built at 16:11 on 2026-03-02.
The v15 frontend components (`RunsPage.tsx`, `ConcurrencyStrip.tsx`, `useHeatmap.ts`, updated
`RunsHeatmap.tsx`, `AgentPanel.tsx`, `QuotaPanel.tsx`) were written at 16:13 — 2 minutes after the
binary was compiled. As a result, the embedded frontend served by the live server was missing all
visual features.

**Resolution:** UAT was conducted against the latest source code served via a Vite dev server on
port 5175 (proxying `/api` to the live server at 7777). All 15 tests pass against the current
source. The stale binary is a deployment issue, not a feature implementation issue.

## Failures
| Test | Classification | Resolution |
|---|---|---|
| none | — | All 15 tests passed |

## Deployment Note
Task `concurrency-heatmap#UAT-1` tracks the stale-binary deployment: the server at port 7777 must
be restarted with a freshly built binary after `cargo build && cargo install --path .` to serve the
v15 components to live users.

## Telemetry Timestamp Note
All existing `.events.json` sidecars (runs before 16:48 on 2026-03-02) lack the `timestamp` field
because they predate the `telemetry-wallclock-timestamps` feature deployment. The implementation is
correct — unit tests pass and the code injects timestamps into all `message_to_event` calls. New
runs after the server is restarted with the new binary will carry timestamps.
