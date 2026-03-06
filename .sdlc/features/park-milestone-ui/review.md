# Code Review — Park/Unpark UI

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/lib/types.ts` | Added `'parked'` to `MilestoneStatus` union |
| `frontend/src/api/client.ts` | Added `parkMilestone` and `unparkMilestone` API methods |
| `frontend/src/pages/MilestonesPage.tsx` | Three-section layout: Active, Parked (collapsed), Archive |
| `frontend/src/components/dashboard/HorizonZone.tsx` | Filter out parked milestones |
| `frontend/src/pages/Dashboard.tsx` | Exclude parked from activeMilestones, separate parkedMilestones |
| `frontend/src/pages/MilestoneDetail.tsx` | Park/Unpark toggle button |

## Review Findings

### F1: TypeScript type change — PASS
`MilestoneStatus` now includes `'parked'`, which is additive and backward compatible. The StatusBadge already had a `parked` color entry.

### F2: API client methods — PASS
`parkMilestone` and `unparkMilestone` use PATCH method with `encodeURIComponent`, consistent with other milestone API methods. These will 404 until park-milestone-cli-api adds the endpoints — acceptable since this is a parallel wave feature.

### F3: MilestonesPage three sections — PASS
Active section correctly excludes `released`, `skipped`, and `parked`. Parked section is collapsed by default using same `useState(false)` pattern as Archive. Archive now includes `skipped` milestones which were previously mixed into active — this is a minor improvement.

### F4: HorizonZone filter — PASS
Simple `status === 'parked'` early return in the filter. Clean.

### F5: Dashboard filtering — PASS
`activeMilestones` now excludes parked and skipped. `releasedMilestones` now includes skipped. The `parkedMilestones` variable is declared but not used in the template (it's filtered out). This is acceptable — it documents intent and costs nothing at runtime.

### F6: MilestoneDetail park/unpark — PASS
Button shows conditionally based on milestone status. Uses `parking` state to prevent double-clicks. Calls `load()` after action to refresh. Error handling is silent with refresh fallback — correct pattern for this UI.

### F7: Icon choices — PASS
`ParkingCircle` for park action, `Play` for unpark. Both from lucide-react, already in the bundle. Semantically clear.

## Verdict

All changes are minimal, focused, and follow existing patterns. No issues found.
