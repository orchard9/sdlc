# Design: Fix Committed Ponder Action Button

## Change Summary

Single-file frontend change in `PonderPage.tsx`. Replace the "Prepare" `<button>` (lines 509-533) with a navigation link to the milestone page.

## Implementation

In the committed-state block (`entry.status === 'committed' && entry.committed_to.length > 0`):

1. Remove the `startRun` / `prepareKey` / `prepareRunning` logic.
2. Replace the `<button>` with a React Router `<Link>` (or `useNavigate` + `<button onClick={navigate}>`) pointing to `/milestone/${entry.committed_to[0]}`.
3. Change label from "Prepare" to "View Milestone".
4. Change icon from `Play` to an appropriate navigation icon (e.g. `ExternalLink` or `ArrowRight` from lucide-react).
5. Keep the same visual styling (emerald border/text) for consistency.

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/pages/PonderPage.tsx` | Replace committed-state Prepare button with View Milestone link |

## No Backend Changes

This is a pure UI fix. No API endpoints or server logic affected.
