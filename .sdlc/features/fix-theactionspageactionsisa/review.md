# Code Review: Fix /actions Page Black Screen

## Summary

Two bugs caused the `/actions` page to appear as a blank/black screen:

1. **Stale frontend build** — The running `sdlc-server` binary was compiled with an older `frontend/dist` that predated the addition of the `/actions` route to React Router and the Actions nav item to the sidebar (added in commit `0bf6f43`). React Router matched no route and rendered an empty `<main>`.

2. **Missing PATH_LABELS entries** — `AppShell.tsx` maintained a `PATH_LABELS` map for mobile header titles, but `/actions`, `/knowledge`, and `/guidelines` were never added. This caused the mobile header to show "SDLC" on those pages.

## Changes Made

### `frontend/src/components/layout/AppShell.tsx`

Added three missing entries to `PATH_LABELS`:

```diff
+  '/actions': 'Actions',
+  '/knowledge': 'Knowledge',
+  '/guidelines': 'Guidelines',
```

This is a one-line fix per missing entry. No behavioral change — purely cosmetic (mobile header title).

### Frontend rebuild + binary install

- `npm run build` regenerated `frontend/dist/` with the current source (new bundle: `index-e_tG_uce.js`)
- `cargo install --path crates/sdlc-cli` compiled and installed the new binary embedding the updated dist

## Verification (Playwright)

Navigation to `http://localhost:61080/actions` on the updated server confirmed:
- Sidebar "setup" section shows "Actions" link with `CalendarClock` icon ✓
- `<main>` renders a loading spinner (ActionsPage component IS rendering) ✓
- No `No routes matched location "/actions"` console warning in new build ✓

## Remaining User Action

The currently running `sdlc ui` process (PID 69940) must be restarted to load the new binary:

```bash
sdlc ui kill sdlc
sdlc ui
```

## Findings Enumerated

| Finding | Action |
|---|---|
| Missing `/actions` in PATH_LABELS | Fixed in AppShell.tsx |
| Missing `/knowledge` in PATH_LABELS | Fixed in AppShell.tsx |
| Missing `/guidelines` in PATH_LABELS | Fixed in AppShell.tsx |
| Stale frontend build in running binary | Fixed by rebuild + `cargo install` |
| Restart required to activate fix | Tracked — user must run `sdlc ui kill sdlc && sdlc ui` |

## Risk

Low. Changes are purely additive to `PATH_LABELS`. No logic changed. Frontend rebuild is deterministic.
