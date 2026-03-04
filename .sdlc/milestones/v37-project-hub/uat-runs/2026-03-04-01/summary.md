# UAT Run — v37-project-hub
**Date:** 2026-03-04T22:05:00Z
**Verdict:** PassWithTasks
**Tests:** 5/6
**Tasks created:** page-title-fix#T4

## Results
Suite: v37-project-hub — Acceptance Tests
Duration: ~120000ms (two run attempts; selector fixes applied between runs)
Passed: 5 | Failed: 1 | Skipped: 0

## Failures

| Test | Classification | Resolution |
|---|---|---|
| Scenario 6: Page title is "sdlc hub" | Code bug | Task page-title-fix#T4 created — embed.rs returns project name in hub mode instead of "sdlc hub" |

## Selector Fixes Applied

Between the first and second run, two selector issues were fixed inline:
1. `waitForLoadState('networkidle')` → `waitForLoadState('domcontentloaded')` — SSE connection keeps network active forever, causing networkidle to never fire
2. `[data-testid="project-card"]` → `getByRole('button', { name: /payments-api/ })` — HubPage has no data-testid attributes; switched to ARIA role + name locators
