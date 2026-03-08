# QA Plan

## Verification
1. Run `tsc -b` in `frontend/` — must exit 0 with no TS6133 errors
2. Run `vite build` in `frontend/` — must produce a successful build
3. Confirm `isRunning` and `focusRun` still work (no runtime regression) by checking the destructuring is intact
