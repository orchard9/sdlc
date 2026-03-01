# UAT Run — UAT run history dashboard, CI gate, and full guidance update
**Date:** 2026-03-01T08:15:00Z
**Verdict:** PassWithTasks
**Tests:** 12/16 (4 failed)
**Tasks created:** milestone-uat-history-panel#T6, playwright-github-actions#T5, guidance-playwright-update#T4

## Results
Suite: v06-uat-dashboard-and-ci.spec.ts
Duration: ~900000ms (including server reinstall and rebuild)
Passed: 12 | Failed: 4 | Skipped: 0

## Failures

| Test | Classification | Resolution |
|---|---|---|
| UatHistoryPanel shows verdict badge, date, test count | Selector break | Fixed `getByText('PASS')` → `getByText('PASS', { exact: true })` — rerun passed |
| useSSE hook handles milestone_uat SSE events | Code bug | Task milestone-uat-history-panel#T6 created |
| uat.yml triggers on push to main | Code bug | Task playwright-github-actions#T5 created |
| uat.yml triggers on pull_request with path filters | Code bug | Task playwright-github-actions#T5 created |
| .sdlc/guidance.md §5 references Playwright | Code bug | Task guidance-playwright-update#T4 created |

## Infrastructure Notes
- Frontend needed rebuild (`npm run build`) + binary reinstall (`cargo install`) to embed new UatHistoryPanel
- v05 `run.yaml` was missing — wrote `.sdlc/milestones/v05-playwright-uat-integration/uat-runs/2026-03-01-run01/run.yaml`
- v06 spec runs correctly when server is pre-running; `reuseExistingServer: true` + isolated run causes collection timing issue
