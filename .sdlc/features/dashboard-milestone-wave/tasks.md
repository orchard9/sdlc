# Tasks: Active Milestones and Run Wave on Dashboard

## T1 — Import MilestonePreparePanel into Dashboard

In `frontend/src/pages/Dashboard.tsx`, add:
```tsx
import { MilestonePreparePanel } from '@/components/milestones/MilestonePreparePanel'
```

This is the only import change needed.

## T2 — Embed MilestonePreparePanel in active milestone sections

In `Dashboard.tsx`, inside the `activeMilestones.map()` render block, insert `<MilestonePreparePanel milestoneSlug={milestone.slug} />` between the heading row and the feature grid.

Remove the existing `CommandBlock` for active milestones (the `cmd` variable and its conditional render block) since `MilestonePreparePanel` already surfaces run commands through its wave plan UI.

The released milestones archive section is unchanged.
