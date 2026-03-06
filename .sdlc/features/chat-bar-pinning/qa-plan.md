# QA Plan: chat-bar-pinning

## Verification Strategy

This is a pure CSS class addition — no logic changes, no new components. QA consists of:

1. **Code inspection** — verify `shrink-0` was added to the correct elements
2. **Build verification** — TypeScript compiles cleanly, no regressions

## Checks

### C1 — DialoguePanel: running state has shrink-0

In `frontend/src/components/ponder/DialoguePanel.tsx`, the `InputBar` function's running
branch (`if (running) { return ... }`) must have `shrink-0` on the root `<div>`.

### C2 — DialoguePanel: idle state has shrink-0

In `frontend/src/components/ponder/DialoguePanel.tsx`, the `InputBar` function's idle
branch (returns `<form>`) must have `shrink-0` on the root `<form>`.

### C3 — InvestigationDialoguePanel: running state has shrink-0

In `frontend/src/components/investigation/InvestigationDialoguePanel.tsx`, the `InputBar`
function's running branch must have `shrink-0` on the root `<div>`.

### C4 — InvestigationDialoguePanel: idle state has shrink-0

In `frontend/src/components/investigation/InvestigationDialoguePanel.tsx`, the `InputBar`
function's idle branch must have `shrink-0` on the root `<form>`.

### C5 — No other classes removed or changed

The only diff in each element's `className` is the addition of `shrink-0` — no other
classes are added, removed, or reordered.

### C6 — TypeScript build passes

Run `cd frontend && npm run build` (or `npx tsc --noEmit`) and confirm no TypeScript errors
are introduced by these changes.

## Pass Criteria

All 6 checks pass. Any failing check is a blocker.
