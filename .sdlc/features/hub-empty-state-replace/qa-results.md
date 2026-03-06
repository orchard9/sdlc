# QA Results: hub-empty-state-replace

## Build

`cd frontend && npm run build` — clean, 0 TypeScript errors, built in 5.41s.

## Verification

- Static string "Configure projects to send heartbeats. See ~/.sdlc/hub.yaml" removed from HubPage.tsx — confirmed absent.
- `EmptyState` now renders `CreateRepoSection` — the same component used in the fleet view's "Add New Project" section.
- No regressions in fleet view — `EmptyState` and the fleet "Add New Project" section are mutually exclusive (EmptyState shown only when `!hasAnyContent`).

## Status: PASSED
