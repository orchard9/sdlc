# Review: milestones-unified-page

## Summary

Implementation complete. Three files changed as planned:

- `frontend/src/pages/MilestonesPage.tsx` — unified page; `filter` prop removed; active milestones at top; collapsible Archive section (default collapsed) with released milestones; PrepareResult fetched via `api.getProjectPrepare()`; Run Wave button on active milestone card when waves exist
- `frontend/src/components/layout/Sidebar.tsx` — Archive nav item removed; `Archive` icon import removed
- `frontend/src/App.tsx` — `/milestones/archive` route removed

TypeScript compiles clean (`tsc --noEmit`).

## Findings

None. The implementation matches the spec exactly:
- Unified `/milestones` page renders active + archive sections
- Archive collapsed by default, toggle shows released count
- Run Wave button uses same `useAgentRuns` + start/stop URL pattern as `WavePlan.tsx`
- Run Wave only shows on the card matching `prepareResult.milestone` when waves exist
- No backend changes
- MilestoneDetail unchanged

## Verdict: Approved
