# QA Plan: dashboard-empty-states

## Test Cases

### TC-1: Global empty state — no vision, no architecture

**Setup:** Project with 0 features and 0 milestones; vision missing, architecture missing.

**Expected:**
- "Define Vision" chip renders, links to /setup
- "Define Architecture" chip renders, links to /setup
- "Start a Ponder" chip does NOT render
- "Create a Feature directly" chip renders, links to /features?new=1
- No generic "New Ponder" button present

### TC-2: Global empty state — vision exists, no architecture

**Setup:** Project with 0 features and 0 milestones; vision exists, architecture missing.

**Expected:**
- "Define Vision" chip does NOT render
- "Define Architecture" chip renders, links to /setup
- "Start a Ponder" chip does NOT render
- "Create a Feature directly" chip renders

### TC-3: Global empty state — both vision and architecture exist

**Setup:** Project with 0 features and 0 milestones; both vision and architecture defined.

**Expected:**
- "Define Vision" chip does NOT render
- "Define Architecture" chip does NOT render
- "Start a Ponder" chip renders, links to /ponder?new=1
- "Create a Feature directly" chip renders

### TC-4: Global empty state NOT shown when features exist

**Setup:** Project with at least 1 feature.

**Expected:**
- `DashboardEmptyState` does not render
- Dashboard shows normal zone content

### TC-5: CurrentZone empty state

**Setup:** Project has features and milestones = 0, ungrouped features = 0.

**Expected:**
- CurrentZone renders the soft empty prompt card
- Card contains "No active work" text
- "Milestones" link navigates to /milestones
- "+ Feature" link navigates to /features?new=1

### TC-6: CurrentZone hides empty state when content exists

**Setup:** Project has at least one active milestone.

**Expected:**
- `CurrentZoneEmpty` does not render
- Milestone rows render normally

### TC-7: Chip navigation

**Expected:**
- Clicking any chip navigates to the correct route (uses react-router Link, no full reload)

### TC-8: No new API calls

**Expected:**
- Network tab shows no additional API requests beyond the existing vision/architecture fetch
- `hasVision` and `hasArch` derive from the existing `useEffect` in Dashboard.tsx

## Pass Criteria

All 8 test cases pass. No TypeScript errors. No console errors.
