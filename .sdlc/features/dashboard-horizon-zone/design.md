# Design: Dashboard Horizon Zone

## Overview

Zone 3 of the four-zone dashboard layout. A compact, forward-looking surface that
appears between Zone 2 (Current) and Zone 4 (Archive). Shows upcoming milestones
(not yet in active progress) and active ponder sessions (ideas still being explored).

## Component Tree

```
Dashboard.tsx
  └─ HorizonZone  (frontend/src/components/dashboard/HorizonZone.tsx)
        ├─ [section header: Telescope icon + "Horizon" label]
        ├─ [horizon milestones list]  — rendered when horizonMilestones.length > 0
        │     └─ HorizonMilestoneRow (inline sub-component)
        └─ [active ponders list]     — rendered when activePonders.length > 0
              └─ HorizonPonderRow (inline sub-component)
```

## Wireframe

```
┌─────────────────────────────────────────────────────────────────────┐
│  🔭 Horizon                                                         │
├─────────────────────────────────────────────────────────────────────┤
│  UPCOMING MILESTONES                                                 │
│  ─────────────────────────────────────────────────────────────────  │
│  v23 — Fleet Deploy Pipeline        [active]    4 features           │
│  v24 — Knowledge Base               [active]    0 features           │
├─────────────────────────────────────────────────────────────────────┤
│  ACTIVE PONDERS                                                      │
│  ─────────────────────────────────────────────────────────────────  │
│  Dev Driver Tool         [exploring]   #tooling #dx       [copy cmd] │
│  Release Notes UX        [converging]  #ux                [copy cmd] │
└─────────────────────────────────────────────────────────────────────┘
```

When either sub-list is empty, that sub-section is omitted entirely. When both are
empty, the entire `HorizonZone` returns `null`.

## File Changes

### `frontend/src/components/dashboard/HorizonZone.tsx`

Replace the stub. Full implementation:

```tsx
import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { Telescope } from 'lucide-react'
import { api } from '@/api/client'
import { StatusBadge } from '@/components/shared/StatusBadge'
import type { MilestoneSummary, FeatureSummary, PonderSummary } from '@/lib/types'

interface HorizonZoneProps {
  milestones: MilestoneSummary[]
  featureBySlug: Map<string, FeatureSummary>
}

export function HorizonZone({ milestones, featureBySlug }: HorizonZoneProps) {
  const [activePonders, setActivePonders] = useState<PonderSummary[]>([])

  useEffect(() => {
    api.getRoadmap().then(all => {
      setActivePonders(
        all.filter(p => p.status === 'exploring' || p.status === 'converging')
      )
    }).catch(() => {/* silent — roadmap list is optional context */})
  }, [])

  // Horizon milestones: active milestones where all assigned features are still in draft
  const horizonMilestones = milestones.filter(m => {
    if (m.features.length === 0) return true
    return m.features.every(slug => {
      const f = featureBySlug.get(slug)
      return !f || f.phase === 'draft'
    })
  })

  if (horizonMilestones.length === 0 && activePonders.length === 0) return null

  return (
    <section className="mb-8">
      <div className="flex items-center gap-2 px-1 mb-3">
        <Telescope className="w-4 h-4 text-muted-foreground" />
        <h2 className="text-sm font-semibold text-muted-foreground">Horizon</h2>
      </div>

      <div className="bg-card border border-border rounded-xl overflow-hidden divide-y divide-border/30">
        {/* Upcoming milestones */}
        {horizonMilestones.length > 0 && (
          <div>
            <div className="px-4 py-2 border-b border-border/50 bg-muted/20">
              <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                Upcoming Milestones
              </span>
            </div>
            {horizonMilestones.map(m => (
              <div key={m.slug} className="flex items-center gap-3 px-4 py-2.5">
                <Link
                  to={`/milestones/${m.slug}`}
                  className="text-sm font-medium hover:text-primary transition-colors flex-1 min-w-0 truncate"
                >
                  {m.title}
                </Link>
                <StatusBadge status={m.status} />
                <span className="text-xs text-muted-foreground shrink-0">
                  {m.features.length} feature{m.features.length !== 1 ? 's' : ''}
                </span>
              </div>
            ))}
          </div>
        )}

        {/* Active ponders */}
        {activePonders.length > 0 && (
          <div>
            <div className="px-4 py-2 border-b border-border/50 bg-muted/20">
              <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                Active Ponders
              </span>
            </div>
            {activePonders.map(p => (
              <div key={p.slug} className="flex items-center gap-3 px-4 py-2.5">
                <Link
                  to={`/ponder/${p.slug}`}
                  className="text-sm font-medium hover:text-primary transition-colors flex-1 min-w-0 truncate"
                >
                  {p.title}
                </Link>
                <StatusBadge status={p.status} />
                {p.tags.slice(0, 2).map(tag => (
                  <span
                    key={tag}
                    className="text-xs text-muted-foreground/70 bg-muted/60 px-1.5 py-0.5 rounded font-mono shrink-0"
                  >
                    #{tag}
                  </span>
                ))}
                <CopyButton text={`/sdlc-ponder ${p.slug}`} />
              </div>
            ))}
          </div>
        )}
      </div>
    </section>
  )
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)
  return (
    <button
      onClick={async () => {
        await navigator.clipboard.writeText(text)
        setCopied(true)
        setTimeout(() => setCopied(false), 1500)
      }}
      className="text-xs text-muted-foreground hover:text-foreground shrink-0 px-1.5 py-0.5 rounded border border-border/50 hover:border-border transition-colors"
      title={text}
    >
      {copied ? '✓' : 'copy'}
    </button>
  )
}
```

### `frontend/src/pages/Dashboard.tsx`

Update the `<HorizonZone />` call to pass props:

```tsx
{/* Zone 3 — Horizon */}
<HorizonZone
  milestones={activeMilestones}
  featureBySlug={featureBySlug}
/>
```

`activeMilestones` and `featureBySlug` are already computed in `Dashboard.tsx` scope.
No new state, effects, or imports needed in Dashboard.tsx.

## Styling Decisions

- Section label outside the card (same pattern as other zones)
- Two sub-sections inside one `bg-card` card — cleaner than two separate cards
  because Horizon content tends to be sparse and two separate cards would look like
  unrelated zones
- Sub-section headers use `bg-muted/20` + `uppercase tracking-wider` labels (same
  pattern as `CurrentZone`'s "Ungrouped" header)
- Ponder rows show max 2 tags to prevent overflow — additional tags silently clipped
- Milestone feature count is pluralized correctly

## Data Flow

```
Dashboard.tsx
  activeMilestones  ──────────────────────────────► HorizonZone.horizonMilestones (filtered)
  featureBySlug     ──────────────────────────────► HorizonZone (used for draft-filter)
  
HorizonZone
  useEffect → api.getRoadmap()  ─► filter(exploring|converging) ─► activePonders state
```

No backend changes. No new API endpoints.

## Edge Cases

| Scenario | Behaviour |
|---|---|
| No upcoming milestones, no ponders | Returns `null` — Zone 3 absent from DOM |
| `api.getRoadmap()` fails | Caught silently; ponders section omitted |
| All milestones have in-progress features | `horizonMilestones` empty; only ponders shown |
| Ponder status is `committed` or `parked` | Filtered out — not shown |
| Milestone with zero features | Shown as horizon (not yet started) |
| Very long milestone/ponder title | `truncate` class clips it |

## What Does Not Change

- The four-zone layout structure in `Dashboard.tsx`
- `AttentionZone`, `CurrentZone`, `ArchiveZone` — unchanged
- `MilestoneDigestRow` — not used here
- Any backend routes
- Any other frontend files
