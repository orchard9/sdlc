# Design: Ponder Status Step Indicator

## Component: `PonderStepIndicator`

**File:** `frontend/src/components/ponder/PonderStepIndicator.tsx`

### Props

```tsx
interface PonderStepIndicatorProps {
  status: PonderStatus   // 'exploring' | 'converging' | 'committed' | 'parked'
  compact?: boolean      // Mini variant for list rows (dots instead of labels)
}
```

### Step Mapping

| Status | Step Index | Completed Steps |
|--------|-----------|-----------------|
| exploring | 0 | none |
| converging | 1 | exploring |
| committed | 2 (done) | exploring, converging |
| parked | special | all muted + parked badge |

### Rendering Logic

Follows the same pattern as `PhaseStrip.tsx`:

```
const STEPS = ['Exploring', 'Converging', 'Committed'] as const
```

1. Compute `currentIdx` from status: exploring=0, converging=1, committed=3 (all done)
2. For each step:
   - **Completed** (`i < currentIdx`): check icon + muted text (`text-muted-foreground/60`)
   - **Current** (`i === currentIdx`): status-colored highlight badge
   - **Upcoming** (`i > currentIdx`): dim text (`text-muted-foreground/30`)
3. Arrow separator (`→`) between steps, same dimming rules as PhaseStrip
4. **Parked** override: all steps render muted, append a neutral "Parked" badge

### Color Mapping (Current Step)

Reuse existing StatusBadge colors:
- exploring: `bg-violet-600/20 text-violet-400`
- converging: `bg-amber-600/20 text-amber-400`
- committed: `bg-emerald-600/20 text-emerald-400`

### Compact Variant (`compact={true}`)

For list rows — three small circles with connecting lines:
- Completed: filled circle (status color)
- Current: filled circle with ring
- Upcoming: hollow circle (muted)
- No text labels, tooltip on hover for accessibility

## Integration Points

### Detail View (`PonderPage.tsx`)

In the entry detail panel, replace the standalone `StatusBadge` with `<PonderStepIndicator status={entry.status} />`. Place it prominently near the title area.

### List Row (`PonderPage.tsx`)

Add `<PonderStepIndicator status={entry.status} compact />` to each list row, positioned after the title. Remove the existing `StatusBadge` from list rows since the step indicator subsumes it.

## No Backend Changes

All data is derived from the existing `PonderStatus` field — no new API endpoints or struct fields needed.

[Mockup](mockup.html)
