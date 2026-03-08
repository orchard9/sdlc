# Tasks

## T1: Remove unused `startRun` from destructuring
- File: `frontend/src/pages/PonderPage.tsx` line 395
- Change `const { isRunning, focusRun, startRun } = useAgentRuns()` to `const { isRunning, focusRun } = useAgentRuns()`

## T2: Verify frontend build passes
- Run `npm --prefix frontend run build` and confirm `tsc -b && vite build` succeeds
