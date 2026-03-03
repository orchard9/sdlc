# Tasks: Dashboard Four-Zone Layout

## T1 — Create `AttentionZone` component
Extract existing Dashboard content (WhatChangedBanner, PreparePanel, escalations,
HITL features, active directives) into `frontend/src/components/dashboard/AttentionZone.tsx`.
The component returns `null` when nothing to show — no rendering side-effects on existing logic.

## T2 — Create `MilestoneDigestRow` component
New file: `frontend/src/components/dashboard/MilestoneDigestRow.tsx`.
- Collapsed default: dot + title link + status badge + progress bar (doneCount / total) + expand chevron
- Second row: "Next: `<action>` · `<slug>`" + CommandBlock (copy button)
- Expand toggle: compact per-feature table (slug link, phase badge, next_action)
- Dot colour: green=all-done, amber=any blocked, blue=active

## T3 — Create `CurrentZone` component
New file: `frontend/src/components/dashboard/CurrentZone.tsx`.
- Maps active milestones to `<MilestoneDigestRow />` instances
- Renders ungrouped features below as a compact list (title link + phase badge)
- No feature card grid

## T4 — Create `HorizonZone` stub
New file: `frontend/src/components/dashboard/HorizonZone.tsx`.
- Returns `null` — layout slot reserved for `dashboard-horizon-zone` feature.

## T5 — Create `ArchiveZone` component
New file: `frontend/src/components/dashboard/ArchiveZone.tsx`.
- Collapsed by default with expand toggle
- Shows released milestones as compact title + badge + slug rows (no feature cards)

## T6 — Refactor `Dashboard.tsx` to four-zone layout
- Import and render AttentionZone, CurrentZone, HorizonZone, ArchiveZone in order
- Remove the old milestone section (FeatureCard grid) and ungrouped grid
- Keep: project header, stats bar, missingVisionOrArch banner
- No functional change to ProjectHeader or StatsBar content

## T7 — Smoke test: verify no regressions
Manual verification:
- Dashboard loads without errors in browser
- Zone 1 (Attention) hidden when no escalations/HITL/active directives
- Zone 2 shows MilestoneDigestRow for each active milestone; expands to feature list
- Zone 4 archive toggle works
- All links navigate correctly (milestone → /milestones/<slug>, feature → /features/<slug>)
- CommandBlock copy button works in MilestoneDigestRow
