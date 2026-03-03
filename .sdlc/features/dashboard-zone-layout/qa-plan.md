# QA Plan: Dashboard Four-Zone Layout

## TC-1 — Zone 1 hidden when empty

**Precondition:** No open escalations, no HITL features, no active directives,
no recent WhatChanged events.

1. Navigate to Dashboard (`/`)
2. Verify: no Attention zone card/panel is rendered — DOM contains no escalation
   rows, no HITL warnings, no active directive rows
3. Verify: the page renders cleanly with only the project header, stats bar,
   Zone 2 (current milestones), and Zone 4 (archive)

## TC-2 — Zone 1 visible when escalation exists

**Precondition:** At least one open escalation in project state.

1. Navigate to Dashboard
2. Verify: AttentionZone renders with the escalation card visible
3. Resolve the escalation via the inline resolve form
4. Verify: on SSE update, escalation disappears; if Zone 1 is otherwise empty,
   it collapses from view

## TC-3 — MilestoneDigestRow collapsed default

1. Navigate to Dashboard with at least one active milestone
2. Verify: each active milestone renders as a compact digest row (single card)
3. Verify: progress bar fraction is correct (doneCount / total features)
4. Verify: "Next" row shows the first non-done feature's action and a copy command
5. Verify: no FeatureCard grid is visible anywhere on the Dashboard

## TC-4 — MilestoneDigestRow expand/collapse

1. Click the expand chevron on a MilestoneDigestRow
2. Verify: a compact feature list appears (one row per feature: slug link, phase
   badge, next_action text)
3. Click the chevron again
4. Verify: list collapses, back to single digest row

## TC-5 — MilestoneDigestRow navigation

1. Click the milestone title in a digest row
2. Verify: navigates to `/milestones/<slug>`
3. Expand a digest row, click a feature slug link
4. Verify: navigates to `/features/<slug>`

## TC-6 — Zone 4 archive toggle

1. Verify: "Archive" section is collapsed by default
2. Click to expand
3. Verify: released milestones appear as compact rows (title + badge + slug)
4. Verify: no feature card grid in archive rows
5. Click to collapse
6. Verify: archive list hidden again

## TC-7 — Progress calculation accuracy

**Setup:** Milestone with 5 features; 2 are in `released` phase (next_action=done),
3 are in flight.

1. Navigate to Dashboard
2. Verify: MilestoneDigestRow shows "2 / 5" and progress bar at ~40%

## TC-8 — Ungrouped features compact list

**Precondition:** At least one feature not assigned to any milestone.

1. Navigate to Dashboard
2. Verify: ungrouped features appear in Zone 2 as a compact list (not a card grid)
3. Verify: each row links to `/features/<slug>`

## TC-9 — HorizonZone renders no content

1. Navigate to Dashboard
2. Verify: no visible content or spacing where Zone 3 would be
   (HorizonZone stub returns null — should leave no visual gap)

## TC-10 — SSE live updates in digest rows

1. Navigate to Dashboard
2. Trigger a feature state change (e.g. approve a task via CLI)
3. Verify: within ~2s the MilestoneDigestRow updates its progress fraction without
   a page refresh (SSE delivery)
