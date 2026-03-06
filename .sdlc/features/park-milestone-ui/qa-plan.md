# QA Plan — Park/Unpark UI

## QA-1: MilestoneStatus type includes parked
- Verify `MilestoneStatus` type in `types.ts` includes `'parked'`

## QA-2: API client methods exist
- Verify `api.parkMilestone` and `api.unparkMilestone` exist in `client.ts`
- Verify they call the correct endpoints with PATCH method

## QA-3: MilestonesPage three sections
- With a mix of active, parked, and released milestones, verify:
  - Active section shows only active/verifying milestones
  - Parked section exists, is collapsed by default, shows count
  - Parked section expands on click to reveal parked milestones
  - Archive section shows released/skipped milestones

## QA-4: HorizonZone excludes parked
- Verify parked milestones do not appear in the Horizon zone on the dashboard

## QA-5: CurrentZone excludes parked
- Verify parked milestones do not appear in the CurrentZone on the dashboard

## QA-6: MilestoneDetail park/unpark button
- Active milestone shows "Park" button
- Parked milestone shows "Unpark" button
- Released/skipped milestone shows no park button
- Clicking park calls `api.parkMilestone` and refreshes
- Clicking unpark calls `api.unparkMilestone` and refreshes

## QA-7: Build passes
- `SDLC_NO_NPM=1 cargo test --all` passes
- `cd frontend && npx tsc --noEmit` passes (TypeScript check)
