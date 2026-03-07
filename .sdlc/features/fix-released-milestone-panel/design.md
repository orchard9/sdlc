# Design: Fix released milestone showing verifying UI

## Overview

Add status-aware routing to `MilestonePreparePanel` so it renders the correct UI based on milestone status rather than solely deriving state from progress numbers.

## Component Changes

### MilestonePreparePanel.tsx

**Props change:**
```tsx
interface Props {
  milestoneSlug: string
  milestoneStatus: MilestoneStatus  // NEW
}
```

**Routing logic (added before existing `isVerifying` check):**
```tsx
// Released milestones get a simple released indicator
if (milestoneStatus === 'released') {
  return <ReleasedMini />
}
```

**New `ReleasedMini` component** (local to this file, minimal placeholder):
```tsx
function ReleasedMini() {
  return (
    <div className="flex items-center gap-2 bg-green-950/20 border border-green-500/20 rounded-lg px-3 py-2">
      <CheckCircle className="w-4 h-4 text-green-400 shrink-0" />
      <span className="text-xs text-green-400 font-medium">Released</span>
    </div>
  )
}
```

This is intentionally minimal — the companion feature `released-milestone-victory-panel` will replace this with the full `ReleasedPanel` component (victory banner, stats, re-run UAT, next milestone link).

### MilestoneDetail.tsx

**One-line change** — pass milestone status through:
```tsx
<MilestonePreparePanel milestoneSlug={slug} milestoneStatus={milestone.status} />
```

## State Routing Table

| `milestoneStatus` | `isVerifying` (derived) | Rendered Component |
|---|---|---|
| `released` | (not checked) | `ReleasedMini` |
| `verifying` | `true` | `VerifyingMini` (existing) |
| `active` | `false`, waves > 0 | `WavePlan` (existing) |
| `active` | `false`, waves = 0 | `null` (existing) |
| `skipped` | (any) | Falls through to existing logic |

## Files Modified

1. `frontend/src/components/milestones/MilestonePreparePanel.tsx` — add prop, add routing, add `ReleasedMini`
2. `frontend/src/pages/MilestoneDetail.tsx` — pass `milestone.status` as new prop
