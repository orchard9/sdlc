# Spec: Fix /actions Page Black Screen

## Problem

Navigating to `/actions` in the SDLC web UI produces a blank/black content area. The sidebar and app shell render normally, but the main content is completely empty.

**Observed behavior (Playwright verification):**
- Browser console warning: `No routes matched location "/actions"`
- The `<main>` element renders empty
- Sidebar shows Tools/Secrets/Agents but no "Actions" link

## Root Cause

Two compounding issues:

### Issue 1 — Stale frontend build (primary — causes the black screen)

Commit `0bf6f43` added:
- `<Route path="/actions" element={<ActionsPage />} />` to `App.tsx`
- The `Actions` nav item (with `CalendarClock` icon) to `Sidebar.tsx`

However, `frontend/dist` (gitignored build artifact embedded into the server binary at compile time) was **not rebuilt** after this commit. The running `sdlc-server` binary therefore serves an older JavaScript bundle that has no `/actions` route registered in React Router. When the user navigates there, React Router finds no matching route and renders nothing — the "black screen."

### Issue 2 — Missing PATH_LABELS entry (secondary — cosmetic)

`frontend/src/components/layout/AppShell.tsx` maintains a `PATH_LABELS` map used to set the mobile header title. `/actions` was never added, so the mobile header shows "SDLC" instead of "Actions" even after the build is fixed.

## Scope of Fix

| File | Change |
|---|---|
| `frontend/src/components/layout/AppShell.tsx` | Add `'/actions': 'Actions'` to `PATH_LABELS` |
| `frontend/` | Run `npm run build` to produce updated `dist/` |
| `sdlc-server` binary | Recompile to embed the updated `frontend/dist` |

**No changes needed** to:
- `ActionsPage.tsx` — component is correct and complete
- `App.tsx` — route is correctly registered in source
- `Sidebar.tsx` — nav item is correctly registered in source
- `BottomTabBar.tsx` — `/actions` is correctly included in Setup tab roots

## Acceptance Criteria

1. Navigating to `http://localhost:<port>/actions` renders the Actions page (Scheduled Actions, Webhook Routes, Webhook Events sections visible)
2. No `No routes matched location "/actions"` warning in browser console
3. The "Actions" link is visible in the sidebar under "setup"
4. On mobile, the header shows "Actions" as the page title
5. The `/actions` BottomTabBar tab activates when on the Actions page

## Non-Goals

- No changes to ActionsPage functionality or data fetching
- No API changes
