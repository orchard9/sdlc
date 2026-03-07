# Spec: Parked Ponder Resume Button

## Problem

When a ponder is parked, the UI hides all action buttons (Commit, Start from title & brief, seed input). The only way to resume exploring is through the generic status-change modal (gear icon → select "exploring" → Apply) — a 3-click flow buried behind an icon that doesn't suggest "resume".

## Solution

Add a dedicated **Resume** button in the ponder detail header when `entry.status === 'parked'`. Clicking it sets the status back to `exploring` via the existing `PUT /api/roadmap/:slug` endpoint. This re-enables all interactive elements (chat input, commit button, action buttons).

## Behavior

1. **Visibility**: The Resume button appears in the ponder detail header only when `entry.status === 'parked'`.
2. **Action**: On click, calls `PUT /api/roadmap/:slug` with `{ "status": "exploring" }`.
3. **Result**: The ponder returns to `exploring` status. Chat input, Commit button, and "Start from title & brief" buttons become visible again.
4. **Icon**: Use `Play` (already imported in PonderPage) to convey "resume".
5. **Placement**: In the detail header bar, same position as the Commit button (which is hidden when parked).

## Scope

- Frontend-only change in `PonderPage.tsx` and `DialoguePanel.tsx`.
- No backend changes — the `update_status` API already supports setting any valid status.
- No new API endpoints.

## Out of Scope

- Bulk resume of multiple parked ponders.
- Auto-resume logic or scheduling.
