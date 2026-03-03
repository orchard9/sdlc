# Review: Recent Activity Scrollable Fixed-Height Below Stats

## Changes Made

### `frontend/src/pages/Dashboard.tsx`
- Removed `<WhatChangedBanner />` from before the Project Overview section (was line 218).
- Inserted `<WhatChangedBanner />` immediately before the "Needs Your Attention" escalations block — placing it after the stats bar as specified.

### `frontend/src/components/layout/WhatChangedBanner.tsx`
- Removed `import { useState }` (no longer needed).
- Removed `const VISIBLE_COUNT = 7`.
- Removed `const [expanded, setExpanded] = useState(false)`.
- Removed `sorted.slice(0, VISIBLE_COUNT)` — all events are now rendered.
- Replaced `<div className="space-y-0.5">` with `<div className="space-y-0.5 max-h-48 overflow-y-auto">`.
- Removed the "See more" button block entirely.

## Findings

| # | Finding | Action |
|---|---------|--------|
| 1 | `hiddenCount` variable also removed (was derived from `VISIBLE_COUNT`) | Fixed — variable is gone |
| 2 | `visible` variable renamed away — all events now used directly as `sorted` | Fixed — clean |

## Verdict

All AC from the spec are satisfied:
- ✅ Banner renders after stats bar, before escalations
- ✅ Event list uses `max-h-48 overflow-y-auto` — no layout shift
- ✅ "See more" button removed
- ✅ `VISIBLE_COUNT` and `expanded` state removed
- ✅ Visual hierarchy: header → stats → recent activity → escalations

No regressions introduced. Approve.
