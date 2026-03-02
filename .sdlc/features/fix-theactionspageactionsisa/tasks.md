# Tasks: Fix /actions Page Black Screen

## T1 — Add `/actions` (and missing siblings) to PATH_LABELS in AppShell.tsx

**File:** `frontend/src/components/layout/AppShell.tsx`

Add the three missing entries to the `PATH_LABELS` map:
- `'/actions': 'Actions'`
- `'/knowledge': 'Knowledge'`
- `'/guidelines': 'Guidelines'`

**Done when:** The map contains all three entries; no other changes required.

---

## T2 — Rebuild frontend dist

Run `npm run build` in the `frontend/` directory to regenerate `frontend/dist/` with the current source code (including the `/actions` route in React Router and the Actions sidebar item from commit `0bf6f43`).

**Done when:** `frontend/dist/assets/index-*.js` timestamp is newer than the last source edit and the file contains the string `"/actions"`.

---

## T3 — Verify fix in running server

Navigate to `/actions` in the SDLC web UI (after restarting/rebuilding the server) and confirm:
- The Actions page renders with Scheduled Actions, Webhook Routes, and Webhook Events sections
- No `No routes matched location "/actions"` console warning
- The "Actions" sidebar link is visible and highlighted when on the page

**Done when:** Playwright snapshot of `/actions` shows a non-empty `<main>` with the Actions page content.
