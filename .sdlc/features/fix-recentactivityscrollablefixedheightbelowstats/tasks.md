# Tasks: Recent Activity Scrollable Fixed-Height Below Stats

## T1 — Move WhatChangedBanner below stats bar in Dashboard.tsx

In `frontend/src/pages/Dashboard.tsx`, move the `<WhatChangedBanner />` call from line 218 (before the Project Overview block) to immediately after the closing `</div>` of the stats bar block (after line 261, before the escalations section).

## T2 — Replace unbounded event list with scrollable container in WhatChangedBanner.tsx

In `frontend/src/components/layout/WhatChangedBanner.tsx`:
- Remove the `VISIBLE_COUNT = 7` constant.
- Remove the `expanded` state and `setExpanded`.
- Remove the `sorted.slice(0, VISIBLE_COUNT)` slicing — use all `sorted` events.
- Replace the `<div className="space-y-0.5">` event list wrapper with `<div className="space-y-0.5 max-h-48 overflow-y-auto">` (or `max-h-56` if preferred for visual balance).
- Remove the "See more" button block entirely.
