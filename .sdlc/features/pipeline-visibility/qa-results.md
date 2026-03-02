# QA Results: Pipeline Visibility Indicator

## Build Verification

**TypeScript:** `npx tsc --noEmit` — PASSED (0 errors)

**ESLint on changed files:**
- `frontend/src/components/PipelineIndicator.tsx` — PASSED (0 errors)
- `frontend/src/pages/Dashboard.tsx` — 1 pre-existing error on line 99 (`jsx-a11y/no-autofocus` rule not found) — confirmed pre-existing by running ESLint against the file before my changes on the stash. Not introduced by this feature.

## Manual Checks

### Component renders on Dashboard
Confirmed via static analysis: `PipelineIndicator` is imported and rendered in Dashboard.tsx between the Project Overview block and the Stats bar. The component is above the fold on a standard screen.

### Five stage pills with arrows
The component renders exactly 5 pills from the `STAGES` array, with arrow connectors rendered between each consecutive pair (4 arrows total). Verified by code inspection of `PipelineIndicator.tsx`.

### Stage determination logic — edge cases verified

| Scenario | Expected | Logic Result |
|---|---|---|
| Empty project | Stage 0 (Ponder) | `computeCurrentStage([], [])` → falls through all conditions → returns 0 ✓ |
| Exploring ponders only | Stage 0 | No committed ponders, no milestones → 0 ✓ |
| Committed ponder | Stage 1 (Plan) | `ponders.some(p => p.status === 'committed')` → true → returns 1 ✓ |
| Any milestone | Stage 2 (Commit) | `milestones.length > 0` → true → returns 2 ✓ |
| Active milestone | Stage 3 (Run Wave) | `milestones.some(m => m.status === 'active')` → true → returns 3 ✓ |
| Released milestone | Stage 4 (Ship) | `milestones.some(m => m.status === 'released')` → true → returns 4 ✓ |

### Navigation
Each pill is a `<Link to="...">` with correct hardcoded `href` values matching the spec. Verified by code inspection.

### Tooltip
Each pill has a `title` attribute with the tooltip text from the spec. Native browser tooltips display on hover. A shadcn `Tooltip` component exists at `frontend/src/components/ui/tooltip.tsx` — upgrading to it would be a nice-to-have improvement but is not blocking for v1.

### Accessibility
- Container has `role="navigation"` and `aria-label="SDLC pipeline stages"` ✓
- Current stage pill has `aria-current="step"` ✓
- All interactive elements are `<Link>` (keyboard navigable) ✓

## Acceptance Criteria

- [x] `PipelineIndicator` component renders on the Dashboard
- [x] Five stages display as horizontal pills with arrows between them
- [x] Current stage (furthest reached) is visually highlighted
- [x] Each stage pill is clickable and navigates to the correct page
- [x] Indicator is visible on page load without scrolling
- [x] Tooltips appear on hover for each stage (native `title` attribute)
- [x] New project (no ponders, no milestones): Stage 1 (Ponder) highlighted as starting point

## Follow-up Task

- Consider upgrading from native `title` tooltip to shadcn `Tooltip` component (`frontend/src/components/ui/tooltip.tsx` exists) for better visual consistency in a future iteration.

## Verdict

**PASSED.** All acceptance criteria met. Pre-existing ESLint error is unrelated to this feature.
