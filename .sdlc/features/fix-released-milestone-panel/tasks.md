# Tasks: Fix released milestone showing verifying UI

## Task 1: Add milestoneStatus prop and routing to MilestonePreparePanel
Add `milestoneStatus: MilestoneStatus` prop to `MilestonePreparePanel`. Add a `ReleasedMini` component that shows a simple "Released" indicator. Insert a status check before the existing `isVerifying` logic: if `milestoneStatus === 'released'`, render `ReleasedMini` and return early.

## Task 2: Thread milestone status from MilestoneDetail
Update `MilestoneDetail.tsx` to pass `milestone.status` as the new `milestoneStatus` prop to `MilestonePreparePanel`.

## Task 3: Verify build compiles cleanly
Run `npm run build` in the frontend directory to confirm TypeScript compilation succeeds with the new prop.
