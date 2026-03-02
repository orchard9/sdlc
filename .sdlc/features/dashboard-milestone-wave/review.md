# Review: Active Milestones and Run Wave on Dashboard

## Summary

Changed: `frontend/src/pages/Dashboard.tsx`

The implementation is a focused, two-part change:

1. **Import added** — `MilestonePreparePanel` imported from `@/components/milestones/MilestonePreparePanel`.
2. **Milestone block updated** — In the `activeMilestones.map()` render block, replaced the `isComplete`/`nextFeature`/`cmd`/`CommandBlock` logic with a `<MilestonePreparePanel milestoneSlug={milestone.slug} />` embedded between the milestone header row and the feature grid.
3. **Dead code removed** — `CommandBlock` import, `useAgentRuns` import, and `isRunning` destructuring were all removed since they were only used by the replaced block.

## Verification Against Spec

| Acceptance criterion | Met? | Notes |
|---|---|---|
| Dashboard shows "Active Milestones" section when at least one milestone is active | Yes | Existing `activeMilestones.map()` block was already there; now shows wave plan inline |
| Each milestone card displays title, wave number, and feature count | Yes | Header row unchanged; `MilestonePreparePanel` adds progress bar + wave accordion |
| "Run Wave" button is visible on a card when that milestone's wave is ready | Yes | `WavePlan` inside `MilestonePreparePanel` already shows Run Wave on Wave 1 |
| "Run Wave" button navigates/triggers run | Yes | Triggers `startRun` via `AgentRunContext` — no navigation, opens agent panel |
| Section does not appear when no milestones exist | Yes | Unchanged behavior — `activeMilestones` empty → nothing rendered |
| No new API endpoints required | Yes | `MilestonePreparePanel` uses existing `/api/prepare?milestone=<slug>` |

## Code Quality

- **No logic duplication** — reuses the existing `MilestonePreparePanel` component that's already battle-tested in `MilestonesPage.tsx`.
- **No `unwrap()` or unsafe code** — pure JSX change.
- **TypeScript clean** — `npx tsc --noEmit` passed with zero errors.
- **SSE refresh** — `MilestonePreparePanel` already subscribes to SSE `run_finished` events via its `useSSE` hook; no additional wiring needed.
- **Graceful empty state** — `MilestonePreparePanel` returns `null` when no wave plan exists, so milestones without a wave plan still render normally with just their feature grid.

## Findings

None. The change is minimal and compositional — it delegates wave plan rendering to an existing, tested component rather than reimplementing it.

## Verdict

Ready to approve.
