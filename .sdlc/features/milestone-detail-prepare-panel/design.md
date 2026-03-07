# Design: Milestone Detail — MilestonePreparePanel Integration

## Change Summary

Single-file edit to `frontend/src/pages/MilestoneDetail.tsx`.

## Placement

Insert `<MilestonePreparePanel milestoneSlug={slug} />` as a new `<section>` between the header `<div>` (line ~106) and the Features `<section>` (line ~108). This mirrors the dashboard layout where `PreparePanel` sits above feature lists.

## Component Contract

```tsx
import { MilestonePreparePanel } from '@/components/milestones/MilestonePreparePanel'

// In the JSX, after the header div:
<MilestonePreparePanel milestoneSlug={slug} />
```

The component:
- Accepts `milestoneSlug: string`
- Fetches `api.getProjectPrepare(milestoneSlug)` internally
- Subscribes to SSE `run_finished` events to auto-refresh
- Returns `null` when no wave plan or verifying state applies (no wrapper needed)

## Files Changed

| File | Change |
|---|---|
| `frontend/src/pages/MilestoneDetail.tsx` | Add import + render `<MilestonePreparePanel>` |

## No Backend Changes

All data flows already exist. No new endpoints, types, or server changes.
