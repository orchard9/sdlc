# Review: Dashboard Four-Zone Layout

## Verdict: PASS

All four zones implemented correctly. TypeScript clean, build passes, spec coverage
complete.

## Files Changed

- `frontend/src/pages/Dashboard.tsx` — gutted and re-wired to four zone components
- `frontend/src/components/dashboard/AttentionZone.tsx` — new
- `frontend/src/components/dashboard/MilestoneDigestRow.tsx` — new
- `frontend/src/components/dashboard/CurrentZone.tsx` — new
- `frontend/src/components/dashboard/HorizonZone.tsx` — new stub (returns null)
- `frontend/src/components/dashboard/ArchiveZone.tsx` — new

## Findings

### Zone 1 — Attention
Escalations, HITL features, and active directives render in independent gated
sections. WhatChangedBanner and PreparePanel always render (they manage their own
empty state internally). Zone renders no outer wrapper when all three live sections
are empty. ✓

### Zone 2 — Current / MilestoneDigestRow
Progress calculation excludes archived features; counts `next_action === 'done'`
correctly. Dot color logic: all-done → green, any blocked → amber, otherwise →
primary (blue). Expand chevron reveals compact per-feature list (slug + phase badge
+ next_action). ✓

### Zone 3 — Horizon
`<HorizonZone />` returns null. Slot is present in Dashboard render order. ✓

### Zone 4 — Archive
Collapsed by default. Toggle reveals compact milestone rows (title, status badge,
slug, feature count). No feature cards. ✓

### Dashboard.tsx
Ungrouped filter: `!assignedSlugs.has(f.slug) && !f.archived && f.phase !== 'released'` —
correct. Active/released milestone split correct. `featureTitleBySlug` Map passed
to AttentionZone for directive title display. ✓

### FeatureCard
Untouched — still used on `/features/<slug>` detail page. Dashboard no longer
imports or renders it. ✓

## Observations (non-blocking)

- Progress bar width (`w-20`) is compact; acceptable for a digest row.
- Command block shows in collapsed state — good UX per spec.
- `EmptyState` shows only when both `milestones.length === 0` and `features.length === 0`,
  which is correct; the empty state sits outside zone rendering.
