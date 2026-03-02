# Design: Fix /actions Page Black Screen

This is a two-part fix: a one-line code change plus a rebuild step.

## Change 1: AppShell.tsx — Add PATH_LABELS entry

**File:** `frontend/src/components/layout/AppShell.tsx`

Add `'/actions': 'Actions'` to the `PATH_LABELS` map so the mobile header shows the correct title:

```diff
 const PATH_LABELS: Record<string, string> = {
   '/': 'Dashboard',
   '/milestones': 'Milestones',
   '/features': 'Features',
   '/milestones/archive': 'Archive',
   '/feedback': 'Feedback',
   '/ponder': 'Ponder',
   '/investigations': 'Root Cause',
   '/evolve': 'Evolve',
   '/tools': 'Tools',
   '/secrets': 'Secrets',
   '/agents': 'Agents',
   '/network': 'Network',
   '/vision': 'Vision',
   '/architecture': 'Architecture',
+  '/actions': 'Actions',
+  '/knowledge': 'Knowledge',
+  '/guidelines': 'Guidelines',
 }
```

Note: `/knowledge` and `/guidelines` are also missing and included in this fix to prevent the same issue recurring for those routes.

## Change 2: Rebuild frontend

```bash
cd frontend && npm run build
```

This regenerates `frontend/dist/` with the current source, including the `/actions` route in React Router and the Actions sidebar link from commit `0bf6f43`.

## Change 3: Restart server

The `sdlc-server` binary embeds `frontend/dist` at compile time via `build.rs`. After rebuilding the frontend dist, restarting the running server is sufficient if it reads from the dist directory at runtime, OR the server must be recompiled:

```bash
cargo build -p sdlc-server
```

## No UI Design Required

The Actions page itself is already fully implemented with correct layout. This fix only ensures the page is reachable. No wireframes needed.
