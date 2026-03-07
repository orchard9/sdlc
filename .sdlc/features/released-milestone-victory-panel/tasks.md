# Tasks: ReleasedPanel Component

## Task 1: Create ReleasedPanel component
Create `frontend/src/components/milestones/ReleasedPanel.tsx` with victory banner, stats summary, re-run UAT button, manual submit link, and next milestone link. Use existing hooks (`useMilestoneUatRun`, `useProjectState`, `useSSE`) and API client. Follow patterns from `MilestonePreparePanel` and `UatHistoryPanel`.

## Task 2: Integrate ReleasedPanel into MilestoneDetail page
Modify `frontend/src/pages/MilestoneDetail.tsx` to conditionally render `ReleasedPanel` when `milestone.status === 'released'`, and `MilestonePreparePanel` otherwise.

## Task 3: Verify build compiles cleanly
Run `npm run build` in `frontend/` to ensure no TypeScript or build errors. Fix any issues found.
