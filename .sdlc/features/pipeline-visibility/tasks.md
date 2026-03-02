# Tasks: Pipeline Visibility Indicator

## T1: Create PipelineIndicator component

**File:** `frontend/src/components/PipelineIndicator.tsx`

Create the component with:
- Five stage pills rendered as `<Link>` elements
- Arrow connectors between pills
- Stage determination logic (greedy — highest reached stage is current)
- Visual states: filled-primary (current), filled-muted with checkmark (completed), ghost (future)
- `title` attribute on each pill for tooltip text
- Props: `ponders: PonderSummary[]`, `milestones: MilestoneSummary[]`

Import `PonderSummary` and `MilestoneSummary` from `@/lib/types`. Import `Link` from `react-router-dom`.

## T2: Integrate PipelineIndicator into Dashboard

**File:** `frontend/src/pages/Dashboard.tsx`

- Import `PipelineIndicator`
- Import `api` from `@/api/client` (already imported)
- Add `useState` + `useEffect` to fetch ponders via `api.getPonders()` (or pass from `state` if available)
- Render `<PipelineIndicator>` between the Project Overview block and the Stats bar
- Ensure it is visible on initial load without scrolling
- Pass `milestones={state.milestones}` and fetched ponders array

## T3: Verify stage logic edge cases

Manual verification (no automated test required) of:
- New project (no ponders, no milestones): Stage 0 (Ponder) highlighted
- Has ponders (exploring/converging): Stage 0 highlighted
- Has committed ponder: Stage 1 (Plan) highlighted
- Has milestones: Stage 2 (Commit) highlighted
- Has active/verifying milestone: Stage 3 (Run Wave) highlighted
- Has released milestone: Stage 4 (Ship) highlighted

Document any adjustments made during implementation in a code comment.
