# Spec: chat-bar-pinning

## Problem

In `DialoguePanel` and `InvestigationDialoguePanel`, the `InputBar` component sits at the
bottom of a `flex flex-col` container. The scrollable session stream above it uses `flex-1
overflow-y-auto`, but the `InputBar`'s root element lacks `shrink-0`. This means that in
certain layout conditions — particularly when the flex container is height-constrained — the
chat bar can be vertically compressed or pushed off the bottom of the viewport instead of
staying anchored at the bottom.

## Goal

The `InputBar` component (and its rendered root element) in both dialogue panels must always
remain fully visible and anchored to the bottom of the panel, regardless of how much content
the scroll area contains.

## Scope

Two files require changes:

1. `frontend/src/components/ponder/DialoguePanel.tsx` — `InputBar` component
2. `frontend/src/components/investigation/InvestigationDialoguePanel.tsx` — `InputBar` component

In each file, the `InputBar` internal render paths (both the `running` branch and the idle
`<form>` branch) must add `shrink-0` to the root element so that the flex container never
collapses the bar.

## Acceptance Criteria

- Both the running-state (`<div>`) and idle-state (`<form>`) renders of `InputBar` in both
  dialogue panels have `shrink-0` on their outermost element.
- No other layout or visual changes are introduced — only the `shrink-0` class is added.
- The fix is minimal and surgical: no component restructuring, no new abstractions.

## Out of Scope

- Any other panel or component that may have a similar issue — this feature only addresses
  the two known locations identified above.
- Responsive or mobile layout changes beyond the `shrink-0` addition.
