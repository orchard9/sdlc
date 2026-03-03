# Design: Dashboard Four-Zone Layout

## Layout Overview

The Dashboard is restructured into four stacked zones. Zones render in order
from top to bottom; empty zones are omitted from the DOM entirely.

```
┌─────────────────────────────────────────────────────────────────┐
│  sdlc               v0.x.y  stats bar                           │
├─────────────────────────────────────────────────────────────────┤
│  ZONE 1 — ATTENTION                          (hidden when empty) │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  WhatChangedBanner (if recent events)                    │   │
│  │  PreparePanel (if active wave)                           │   │
│  │  Escalations (if open)                                   │   │
│  │  HITL / blocked features (if any)                        │   │
│  │  Active directives (if agent running)                    │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  ZONE 2 — CURRENT MILESTONES                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  MilestoneDigestRow                                      │   │
│  │  ┌────────────────────────────────────────────────────┐  │   │
│  │  │ ● v22 — Project Changelog  [in-progress] ██░░ 2/5  │  │   │
│  │  │   Next: implement_task  /sdlc-run changelog-core ⎘ │  │   │
│  │  └────────────────────────────────────────────────────┘  │   │
│  │                           [▾ chevron to expand]           │   │
│  │  Expanded:                                               │   │
│  │  ┌────────────────────────────────────────────────────┐  │   │
│  │  │  changelog-core      implementation  implement_task│  │   │
│  │  │  changelog-api       implementation  implement_task│  │   │
│  │  │  changelog-cli       planned         implement_task│  │   │
│  │  │  changelog-dashboard draft           create_spec   │  │   │
│  │  │  changelog-banner    released        done          │  │   │
│  │  └────────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  Ungrouped features (compact list, if any)                       │
├─────────────────────────────────────────────────────────────────┤
│  ZONE 3 — HORIZON                      <HorizonZone /> stub      │
├─────────────────────────────────────────────────────────────────┤
│  ZONE 4 — ARCHIVE          [▶ Archive (N released)]  (collapsed) │
│  Expanded: compact milestone title + badge rows                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Structure

### `<Dashboard />` (refactored)

Top-level orchestrator. No visual markup of its own — just renders the
four zone sections and passes data down.

```tsx
<div className="max-w-5xl mx-auto p-4 sm:p-6">
  {missingVisionOrArch && <VisionArchBanner />}
  <ProjectHeader state={state} config={config} />
  <StatsBar state={state} />

  {/* Zone 1 */}
  <AttentionZone state={state} />

  {/* Zone 2 */}
  <CurrentZone state={state} featureBySlug={featureBySlug} />

  {/* Zone 3 */}
  <HorizonZone />   {/* stub — returns null until dashboard-horizon-zone ships */}

  {/* Zone 4 */}
  <ArchiveZone milestones={releasedMilestones} />
</div>
```

---

### `<AttentionZone />` (`frontend/src/components/dashboard/AttentionZone.tsx`)

Renders: WhatChangedBanner, PreparePanel, escalations, HITL features, active
directives. Returns `null` when nothing to show.

No structural change to any of these sub-components — they are extracted from
`Dashboard.tsx` and wrapped in this container.

---

### `<CurrentZone />` (`frontend/src/components/dashboard/CurrentZone.tsx`)

Props:
```ts
interface CurrentZoneProps {
  milestones: MilestoneSummary[]  // active (non-released)
  featureBySlug: Map<string, FeatureSummary>
  ungrouped: FeatureSummary[]
}
```

Renders:
1. One `<MilestoneDigestRow />` per active milestone
2. An ungrouped compact list if `ungrouped.length > 0`

---

### `<MilestoneDigestRow />` (`frontend/src/components/dashboard/MilestoneDigestRow.tsx`)

The central new component of this feature.

Props:
```ts
interface MilestoneDigestRowProps {
  milestone: MilestoneSummary
  features: FeatureSummary[]
}
```

**Collapsed state (default):**
```
┌──────────────────────────────────────────────────────────────────┐
│  ● v22 — Project Changelog    [in-progress]  ████░░░░  2 / 5    ▾│
│  Next: implement_task · changelog-core    /sdlc-run …      [⎘]  │
└──────────────────────────────────────────────────────────────────┘
```

- Left: colored dot (green=all-done, blue=active, amber=blocked) + milestone title
  (linked to `/milestones/<slug>`)
- Centre: status badge + progress bar (done features / total features)
- Right: expand chevron (▾/▸)
- Second row: "Next: `<action>` · `<feature-slug>`" + copy-ready command block

Progress calculation:
```ts
const doneCount = features.filter(f => f.next_action === 'done').length
const pct = features.length > 0 ? doneCount / features.length : 0
```

Next feature = first feature where `next_action !== 'done'` and `!f.archived`.

**Expanded state:**

A compact feature list replaces (or extends below) the collapsed row:

| Feature slug | Phase badge | Next action |
|---|---|---|
| changelog-core | implementation | implement_task |
| changelog-api | implementation | implement_task |
| … | … | … |

Each row links to `/features/<slug>`.

**State:** `useState<boolean>(false)` for expanded toggle. No server call.

---

### `<HorizonZone />` (`frontend/src/components/dashboard/HorizonZone.tsx`)

Stub component — returns `null`. Created now so the Dashboard layout slot
exists; `dashboard-horizon-zone` will replace the implementation.

---

### `<ArchiveZone />` (`frontend/src/components/dashboard/ArchiveZone.tsx`)

Props: `milestones: MilestoneSummary[]`

Collapsed by default. Expand toggle reveals a compact list:
```
[▶ Archive  (3 released)]

→ expanded:
  v20 Feedback Threads   [released]   v20-feedback-threads
  v19 …                  [released]   …
```

No feature cards — just title + badge + slug. Each title links to
`/milestones/<slug>`.

---

## Files Changed

| File | Change |
|---|---|
| `frontend/src/pages/Dashboard.tsx` | Gutted and re-wired to four zone components |
| `frontend/src/components/dashboard/AttentionZone.tsx` | New — extracted from Dashboard |
| `frontend/src/components/dashboard/CurrentZone.tsx` | New |
| `frontend/src/components/dashboard/MilestoneDigestRow.tsx` | New — key component |
| `frontend/src/components/dashboard/HorizonZone.tsx` | New stub |
| `frontend/src/components/dashboard/ArchiveZone.tsx` | New |

Existing `FeatureCard`, `PreparePanel`, `WhatChangedBanner`, `DashboardEmptyState`
are unchanged — only their import site moves.

## No API Changes

All data comes from the existing `useProjectState()` hook and `MilestoneSummary` /
`FeatureSummary` types. No new endpoints, no new server-side code.
