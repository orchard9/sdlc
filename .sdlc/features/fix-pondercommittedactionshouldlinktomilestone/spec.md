# Spec: Fix Committed Ponder Action Button

## Problem

On the ponder detail page (`/ponder/<slug>`), when a ponder has status `committed` with committed milestones, the action button unconditionally renders a "Prepare" button that triggers milestone preparation (`/api/milestone/<slug>/prepare`). This is incorrect — once a ponder has been committed, its job is done. The user should be directed to the milestone itself, not offered to re-run preparation.

## Root Cause

`PonderPage.tsx` lines 509-533: The committed-state block always renders a `<button>` that calls `startRun` with `milestone_prepare`. It does not check whether the milestone already exists or has been prepared — it blindly offers "Prepare".

## Solution

Replace the "Prepare" button in the committed state with a "View Milestone" navigation link. When `entry.status === 'committed'` and `entry.committed_to.length > 0`, render a link/button that navigates to the milestone page (`/milestone/<slug>`) for the first committed milestone, instead of triggering the prepare agent run.

## Acceptance Criteria

1. When a ponder has status `committed` and `committed_to` contains milestone slugs, the action button reads "View Milestone" (or similar) and navigates to `/milestone/<first-slug>`.
2. The "Prepare" button and its associated `startRun` call are removed from the committed state.
3. No functional regression for other ponder statuses (`exploring`, `parked`, non-committed states).

## Scope

- **File**: `frontend/src/pages/PonderPage.tsx` (lines ~509-533)
- **Change type**: UI behavior fix — replace button action, no backend changes.
