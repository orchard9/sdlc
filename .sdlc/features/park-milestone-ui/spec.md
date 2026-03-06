# Park/Unpark UI — Three-Section Milestones Page, Filter from Horizon

## Problem

The milestones page currently shows two sections: Active (all non-released) and Archive (released). When milestones are parked via the backend (park-milestone-core), the UI has no way to distinguish between actively-worked milestones and parked ones. Parked milestones also appear in the dashboard Horizon zone, creating noise.

## Solution

Update the frontend to recognize the `parked` milestone status and provide three visual treatments:

1. **MilestonesPage** — three sections: Active, Parked (collapsed by default), Archive
2. **HorizonZone** — filter out milestones with `status === 'parked'`
3. **MilestoneDetail** — park/unpark toggle button
4. **TypeScript types** — add `'parked'` to `MilestoneStatus`
5. **API client** — add `parkMilestone` and `unparkMilestone` methods

## Scope

### In scope
- Add `'parked'` to `MilestoneStatus` type union in `types.ts`
- Add `parkMilestone(slug)` and `unparkMilestone(slug)` to API client (`client.ts`)
- MilestonesPage: split active milestones into Active (non-parked) and Parked sections; Parked section is collapsed by default with a toggle, matching the Archive pattern
- HorizonZone: exclude milestones where `status === 'parked'`
- MilestoneDetail: add a park/unpark button that calls the REST endpoint
- Dashboard CurrentZone: exclude milestones where `status === 'parked'`

### Out of scope
- Backend data model changes (park-milestone-core)
- CLI commands and REST endpoints (park-milestone-cli-api)
- Park reason field, park history, bulk park operations
- Feature-level parking

## Dependencies

This feature depends on park-milestone-core (for the `parked` status in `compute_status()`) and park-milestone-cli-api (for the REST endpoints `PATCH /api/milestones/:slug/park` and `PATCH /api/milestones/:slug/unpark`). The UI changes are safe to implement ahead of the backend — the `parked` status simply won't appear until the backend supports it, and the API calls will be ready to use once the endpoints exist.

## Acceptance Criteria

1. MilestonesPage shows three sections: Active milestones, Parked milestones (collapsed), Archive (released/skipped)
2. Parked section shows a count and is expandable, matching the Archive UX pattern
3. HorizonZone on the dashboard does not show parked milestones
4. CurrentZone on the dashboard does not show parked milestones
5. MilestoneDetail page shows a park button (for active milestones) or unpark button (for parked milestones)
6. `MilestoneStatus` type includes `'parked'`
7. StatusBadge already handles `parked` status (confirmed — already present)
8. API client has `parkMilestone` and `unparkMilestone` methods
