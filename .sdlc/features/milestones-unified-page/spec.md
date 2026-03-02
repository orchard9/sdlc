# Spec: Milestones Page — Unified View with Archive Section and Run Wave Button

## Problem

The milestones page is split across two separate routes (`/milestones` and `/milestones/archive`) with two nav items in the sidebar ("Milestones" and "Archive"). This fragments the view and "Archive" is a confusing label for completed milestones. Additionally, the milestones page has no way to kick off a wave run directly — you have to go to the dashboard.

## Solution

Unify into a single `/milestones` page with:
1. Active milestones at the top (status ≠ released)
2. A collapsible "Archive" section at the bottom for released milestones (collapsed by default)
3. A "Run Wave" button on the active milestone's card (when a wave plan exists)

Remove the `/milestones/archive` route and the "Archive" sidebar nav entry.

## Acceptance Criteria

- Navigating to `/milestones` shows all milestones — active ones first, then a collapsible archive section
- Archive section is collapsed by default; toggle shows count ("Archive · N released")
- Active milestones render as before (title link, status badge, vision, feature chips)
- Released milestones render in the archive section with same card UI
- The "Archive" sidebar nav item is removed; only "Milestones" remains
- `/milestones/archive` route is removed from App.tsx
- A "Run Wave" button appears on the card for the currently active milestone (from PrepareResult) when waves exist
  - Shows "Running" + spinner if the wave run is in-flight
  - Shows "Run Wave" + play icon otherwise
  - Same key pattern as WavePlan.tsx: `milestone-run-wave:${slug}`
  - Uses `/api/milestone/${slug}/run-wave` and `/api/milestone/${slug}/run-wave/stop`
- No backend changes; no new API endpoints; no changes to MilestoneSummary type

## Files Changed

- `frontend/src/pages/MilestonesPage.tsx` — refactor; remove `filter` prop; add archive section; add run wave button
- `frontend/src/components/layout/Sidebar.tsx` — remove Archive nav item
- `frontend/src/App.tsx` — remove `/milestones/archive` route

## Out of Scope

- Per-milestone wave data in the API
- Changes to MilestoneDetail page
- Changes to BottomTabBar
- Backend changes
