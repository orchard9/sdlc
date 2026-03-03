# Spec: Recent Activity — Scrollable Fixed-Height Block Below Stats

## Problem

`WhatChangedBanner` currently renders at line 218 of `Dashboard.tsx` — **above** the project header (line 220) and the stats bar (line 239). This breaks the natural top-down reading flow: the user sees a change feed before they know what project they're looking at or what its current state is.

Additionally, the event list uses an unconstrained `space-y-0.5` div with a "See more" toggle. When many events exist, the list expands in-place, causing unpredictable page-layout shifts that push all milestone cards down.

## Goal

Move the recent-activity banner to appear immediately **below the stats bar** and convert its event list from an inline-expanding list to a **fixed-height scrollable container**, eliminating layout shift.

## Acceptance Criteria

1. `WhatChangedBanner` renders after the stats bar (`<!-- Stats bar -->`) and before escalations / wave plan in `Dashboard.tsx`.
2. The event list in `WhatChangedBanner` uses `max-h-48 overflow-y-auto` (or equivalent fixed height ≈192 px) instead of the current unbounded `space-y-0.5` div.
3. The "See more" toggle button is removed — the container scrolls natively.
4. `VISIBLE_COUNT` constant is removed from `WhatChangedBanner.tsx` (no longer needed).
5. The `expanded` state variable and `setExpanded` are removed from `WhatChangedBanner.tsx`.
6. Visual hierarchy: project header → stats bar → recent activity → escalations.
7. No other Dashboard sections are reordered.

## Files Affected

- `frontend/src/pages/Dashboard.tsx` — move `<WhatChangedBanner />` from line 218 to after the stats bar block (after line 261).
- `frontend/src/components/layout/WhatChangedBanner.tsx` — replace unbounded list + "See more" toggle with a `max-h`/`overflow-y-auto` container.

## Out of Scope

- No changes to the changelog data layer (`useChangelog`, `/api/changelog`).
- No changes to dismiss logic or event sorting.
- No new tests required (pure layout/CSS change, covered by visual UAT).
