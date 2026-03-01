# UAT Run — sdlc-milestone-uat uses Playwright as its execution engine
**Date:** 2026-03-01T07:26:00Z
**Verdict:** Pass
**Tests:** 10/11 (1 skipped — v04 has no persisted UatRun yet, expected)
**Tasks created:** none

## Results
Suite: v05-playwright-uat-integration.spec.ts
Duration: 1700ms
Passed: 10 | Failed: 0 | Skipped: 1

## Test Breakdown

| Test | Status | Notes |
|---|---|---|
| @microsoft/playwright-mcp is registered in .mcp.json | ✅ Pass | |
| start_milestone_uat includes Playwright MCP tools in allowed_tools | ✅ Pass | |
| sdlc-milestone-uat skill has Mode A language | ✅ Pass | |
| sdlc-milestone-uat skill has Mode B language | ✅ Pass | |
| UatRun struct in sdlc-core has all required fields | ✅ Pass | |
| save_uat_run, list_uat_runs, latest_uat_run exist in sdlc-core | ✅ Pass | |
| GET /api/milestones/{slug}/uat-runs returns a JSON array | ✅ Pass | |
| GET /api/milestones/{slug}/uat-runs returns array for milestone with no runs | ✅ Pass | |
| GET /api/milestones/{slug}/uat-runs/latest returns 200 or 404 | ✅ Pass | |
| GET /api/milestones/{slug}/uat-runs/latest with runs returns UatRun shape | ⏭ Skipped | No persisted run yet — correct behaviour |
| uat-runs directory created for milestones with completed UAT | ✅ Pass | |

## Fixes Applied
- **ESM `__dirname` fix**: Spec initially used `__dirname` (CommonJS); fixed to `fileURLToPath(import.meta.url)` for ESM compatibility
- **Server rebuild**: New UAT-run routes needed a binary rebuild before API tests could pass
