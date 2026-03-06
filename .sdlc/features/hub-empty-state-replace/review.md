# Code Review: hub-empty-state-replace

## Change

`frontend/src/pages/HubPage.tsx` — `EmptyState` component replaced. 9 lines removed, 14 lines added.

## Findings

### PASS — Correct component used

`CreateRepoSection` is defined above `EmptyState` in the same file. No import needed. The component is already available in scope.

### PASS — Layout

`max-w-md mx-auto w-full` constrains the form to a readable width when shown centered on a wide screen, consistent with how form-style empty states are handled elsewhere in the app.

### PASS — No regression on fleet view

`EmptyState` is only shown when `!hasAnyContent`. In the fleet view (when `fleetAvailable`), the "Add New Project" section with `CreateRepoSection` is shown separately. There is no double-render.

### PASS — Build clean

`npm run build` — 0 TypeScript errors, `✓ built in 5.41s`.

### PASS — Heartbeat message removed

The string "Configure projects to send heartbeats. See ~/.sdlc/hub.yaml" no longer appears anywhere in the rendered UI. Users with no projects see an actionable form instead.

## Verdict

APPROVED.
