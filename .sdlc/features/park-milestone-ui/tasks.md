# Tasks — Park/Unpark UI

## T1: Add parked to MilestoneStatus type and API client methods
- Add `'parked'` to `MilestoneStatus` union in `frontend/src/lib/types.ts`
- Add `parkMilestone(slug)` and `unparkMilestone(slug)` to `frontend/src/api/client.ts`

## T2: Three-section MilestonesPage
- Update `frontend/src/pages/MilestonesPage.tsx` to split into Active, Parked (collapsed), and Archive sections
- Parked section uses same collapsible pattern as Archive

## T3: Filter parked from HorizonZone
- Update `frontend/src/components/dashboard/HorizonZone.tsx` to exclude `status === 'parked'` milestones

## T4: Filter parked from Dashboard CurrentZone
- Update `frontend/src/pages/Dashboard.tsx` to exclude parked milestones from CurrentZone

## T5: Park/Unpark button on MilestoneDetail
- Add park/unpark toggle button to `frontend/src/pages/MilestoneDetail.tsx`
- Show "Park" for active/verifying milestones, "Unpark" for parked ones
- Call API on click and reload milestone data
