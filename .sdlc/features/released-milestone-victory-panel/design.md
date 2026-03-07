# Design: ReleasedPanel Component

## Component Architecture

### New File: `frontend/src/components/milestones/ReleasedPanel.tsx`

A self-contained React component that receives the milestone slug and renders the victory state.

```
ReleasedPanel
  props: { milestoneSlug: string }
  state:
    - uatRuns: UatRun[] (from api.listMilestoneUatRuns)
    - milestones: MilestoneSummary[] (from useProjectState)
  hooks:
    - useMilestoneUatRun(milestoneSlug) — re-run UAT, running state
    - useProjectState() — access milestones list for "next milestone" link
    - useSSE — refresh UAT runs on milestone_uat events
```

### Layout Structure

```
+------------------------------------------------------------------+
| [CheckCircle icon]  Milestone Released                           |
|  "milestone-title"                                                |
+------------------------------------------------------------------+
| Stats row:                                                        |
|  [N features]  [N UAT runs]  [Latest: PASS badge]  [Date]       |
+------------------------------------------------------------------+
| [Re-run UAT button]  |  [Submit manually link]                   |
+------------------------------------------------------------------+
| Next: [link to next active milestone] ->                         |
+------------------------------------------------------------------+
```

### Visual Design

- **Victory banner**: `bg-green-950/30 border border-green-500/30 rounded-lg p-4`
- **Icon**: `CheckCircle` from lucide-react, `text-green-400 w-5 h-5`
- **Title**: `text-green-400 font-medium text-sm` for "Milestone Released", milestone title in `text-foreground/80 text-xs`
- **Stats**: Row of `text-xs text-muted-foreground` items with `tabular-nums` for numbers
- **UAT verdict badge**: Reuse the `VerdictBadge` pattern from `UatHistoryPanel` — extract or inline
- **Re-run button**: Same style as existing `VerifyingMini` — green-500/20 bg, green-400 text
- **Next milestone link**: `text-xs text-green-400 hover:underline` with `ArrowRight` icon

### Integration into MilestoneDetail

Modify `MilestoneDetail.tsx` to conditionally render `ReleasedPanel` when `milestone.status === 'released'`, placed above the features list (same position as `MilestonePreparePanel`).

The `MilestonePreparePanel` already handles the `verifying` state via `VerifyingMini`. For `released` milestones, we render `ReleasedPanel` instead.

```tsx
// In MilestoneDetail.tsx, replace the single MilestonePreparePanel call:
{milestone.status === 'released' ? (
  <ReleasedPanel milestoneSlug={slug} />
) : (
  <MilestonePreparePanel milestoneSlug={slug} />
)}
```

### Data Flow

1. `ReleasedPanel` mounts, calls `api.listMilestoneUatRuns(milestoneSlug)` to get UAT history
2. Extracts stats: total runs, latest verdict, latest completion date
3. Gets milestone list from `useProjectState()` to find next active milestone
4. `useMilestoneUatRun` provides re-run UAT and running state
5. SSE subscription refreshes UAT runs when new runs complete

### No New API Endpoints

All data sourced from:
- `GET /api/milestones/:slug/uat-runs` — UAT history
- `useProjectState()` — milestones list (already loaded)
- `useMilestoneUatRun` hook — UAT trigger + running state

[Mockup](mockup.html)
