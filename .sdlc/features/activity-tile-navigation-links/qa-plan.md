# QA Plan: Activity Tile Navigation Links

## Test Cases

### TC1: Feature run shows link to feature detail page
1. Observe a RunCard for a `feature` type run in the agent panel.
2. Verify a navigation link appears below the meta line showing `features/{slug}`.
3. Click the link.
4. **Expected**: Browser navigates to `/features/{slug}` without full page reload. The RunCard does not toggle expand/collapse.

### TC2: Milestone run types show link to milestone detail page
1. Observe RunCards for `milestone_uat`, `milestone_prepare`, or `milestone_run_wave` type runs.
2. Verify each shows a link to `milestones/{target}`.
3. Click the link.
4. **Expected**: Navigates to `/milestones/{target}`.

### TC3: Ponder run shows link to ponder detail page
1. Observe a RunCard for a `ponder` type run.
2. Verify the link shows `ponder/{target}`.
3. Click.
4. **Expected**: Navigates to `/ponder/{target}`.

### TC4: Investigation run shows link to investigation detail page
1. Observe a RunCard for an `investigation` type run.
2. Verify the link shows `investigations/{target}`.
3. Click.
4. **Expected**: Navigates to `/investigations/{target}`.

### TC5: Project-level runs show no link
1. Observe a RunCard for a `vision_align` or `architecture_align` type run.
2. **Expected**: No navigation link is rendered.

### TC6: Click does not toggle card expand/collapse
1. Click the navigation link on any RunCard.
2. **Expected**: Card expand/collapse state does not change.

### TC7: TypeScript build passes
1. Run `npx tsc --noEmit` in `frontend/`.
2. **Expected**: Zero errors.
