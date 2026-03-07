# Spec: Fix released milestone showing verifying UI

## Problem

The `MilestonePreparePanel` component currently determines what to display based solely on `milestone_progress` data from the prepare endpoint. When all features are released, it shows the "verifying" UI (`VerifyingMini`) — regardless of whether the milestone itself has been marked as released.

This means a milestone with `status: 'released'` still shows "All features released" with a "Run UAT" button, which is incorrect. Released milestones should show a released/victory state, not the verifying state.

## Root Cause

`MilestonePreparePanel` receives only `milestoneSlug` as a prop. It fetches prepare data and derives `isVerifying` from progress numbers alone. It has no knowledge of the milestone's actual status (`active | verifying | released | skipped`).

## Solution

1. **Pass milestone status to `MilestonePreparePanel`** — add a `milestoneStatus` prop of type `MilestoneStatus`.
2. **Route on status** — before the existing `isVerifying` logic, check if `milestoneStatus === 'released'`. If so, render a `ReleasedMini` placeholder (simple released state indicator) instead of `VerifyingMini`.
3. **`MilestoneDetail.tsx`** already has the milestone object with `status` — thread it through as the new prop.

## Scope

- `MilestoneDetail.tsx` — pass `milestone.status` to `MilestonePreparePanel`.
- `MilestonePreparePanel.tsx` — accept `milestoneStatus` prop, add routing logic for `released` status, render a simple released indicator.
- No backend changes required — the status is already available on the milestone detail response.

## Out of Scope

- The full victory panel with stats, re-run UAT, and next milestone link — that is the companion feature `released-milestone-victory-panel`.
- This feature provides the routing and a minimal released indicator that the companion feature will later replace with the full `ReleasedPanel` component.

## Acceptance Criteria

- When a milestone has `status: 'released'`, the detail page shows a released indicator (not the verifying/UAT UI).
- When a milestone has `status: 'verifying'` and all features are released, the existing verifying UI with "Run UAT" is still shown.
- When a milestone is `active` with in-progress waves, the wave plan UI is shown as before.
- No regression to existing milestone detail functionality.
