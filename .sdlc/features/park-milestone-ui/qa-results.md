# QA Results — Park/Unpark UI

## QA-1: MilestoneStatus type includes parked — PASS
`types.ts` line 38: `export type MilestoneStatus = 'active' | 'verifying' | 'released' | 'skipped' | 'parked'`

## QA-2: API client methods exist — PASS
`client.ts` includes `parkMilestone` (PATCH `/api/milestones/:slug/park`) and `unparkMilestone` (PATCH `/api/milestones/:slug/unpark`).

## QA-3: MilestonesPage three sections — PASS
- Active: filters `status !== 'released' && status !== 'skipped' && status !== 'parked'`
- Parked: filters `status === 'parked'`, collapsed by default (`useState(false)`), shows count
- Archive: filters `status === 'released' || status === 'skipped'`, collapsed by default

## QA-4: HorizonZone excludes parked — PASS
`HorizonZone.tsx` adds `if (m.status === 'parked') return false` at the top of the filter.

## QA-5: CurrentZone excludes parked — PASS
`Dashboard.tsx` filters `activeMilestones` to exclude `parked` and `skipped`, so CurrentZone never receives parked milestones.

## QA-6: MilestoneDetail park/unpark button — PASS
- `canPark` is true for `active` or `verifying` status
- `canUnpark` is true for `parked` status
- Park button: muted border style with ParkingCircle icon
- Unpark button: primary-colored border style with Play icon
- Both disabled during API call (`parking` state)
- Calls `load()` after action to refresh

## QA-7: Build passes — PASS
- `SDLC_NO_NPM=1 cargo test --all` — all tests pass
- `npx tsc --noEmit` — zero errors
