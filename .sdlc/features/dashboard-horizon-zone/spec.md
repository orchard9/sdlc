# Spec: Dashboard Horizon Zone (Zone 3)

## Problem

Zone 3 of the dashboard four-zone layout is currently a stub (`HorizonZone` returns
`null`). The dashboard shows what is happening **now** (Zone 2 — Current) and what
already shipped (Zone 4 — Archive), but provides no forward-looking surface. Users
have no at-a-glance view of what is coming next: upcoming milestones that have not
yet started, and active ponders that represent ideas still being shaped.

## Goal

Implement Zone 3 — Horizon — as a compact, forward-looking surface on the main
Dashboard. It surfaces:

1. **Upcoming milestones** — milestones that exist but have no `active` or `released`
   status (i.e. `status === 'active'` milestones with zero non-draft features, or
   milestones not yet started).
2. **Active ponders** — ponder entries in `exploring` or `converging` status (the
   ones being actively explored, not yet committed or parked).

Zone 3 renders between Zone 2 (Current) and Zone 4 (Archive) on the Dashboard page.
It is omitted from the DOM entirely when there is nothing to show.

## Data Sources

Both data sources are already available via the existing API:

- **Milestones**: `useProjectState()` → `state.milestones` — the same hook used by the
  rest of the Dashboard. Upcoming milestones are those with `status !== 'released'`
  that are **not** in the `activeMilestones` set (i.e., already filtered into Zone 2).
  In practice: a milestone is "horizon" if `status === 'active'` and all its features
  are still in `draft` or very early phases, OR any milestone that exists but is
  effectively not yet in motion. The simplest correct approach: milestones passed to
  Zone 3 are those not passed to Zone 2. The Dashboard already splits milestones into
  `activeMilestones` (non-released, currently in Zone 2) and `releasedMilestones`
  (Zone 4). Zone 3 receives the same `activeMilestones` list and sub-filters to those
  with no actively-worked features.
  
  **Decision**: Rather than double-splitting in Dashboard.tsx, keep it simple —
  `activeMilestones` = all non-released. Zone 2 shows milestones that have at least
  one feature with a non-draft phase or a non-done directive. Zone 3 shows the rest
  (milestones that are all-draft or empty). The split logic lives inside the
  `HorizonZone` component, not in Dashboard.tsx, to keep Dashboard.tsx clean.

- **Ponders**: fetched via `api.getRoadmap()` — returns `PonderSummary[]`. Only
  `exploring` and `converging` status ponders are shown. `committed` and `parked`
  are excluded. This requires a fresh `useState` + `useEffect` fetch inside
  `HorizonZone` (same pattern as Dashboard's `config` fetch).

## Component Structure

### `HorizonZone`

Replace the stub in `frontend/src/components/dashboard/HorizonZone.tsx`.

**Props**:
```ts
interface HorizonZoneProps {
  milestones: MilestoneSummary[]       // active (non-released) milestones
  featureBySlug: Map<string, FeatureSummary>
}
```

**Internal behaviour**:
1. Filter `milestones` to "horizon" milestones — those where all assigned features
   are still in `draft` phase (or the milestone has no assigned features). These are
   milestones not yet meaningfully in progress.
2. Fetch active ponders via `api.getRoadmap()` in a `useEffect`, filter to
   `status === 'exploring' || status === 'converging'`.
3. If both lists are empty, return `null`.
4. Render a labeled section with two sub-lists.

**Milestone rows** (horizon milestones):
- Milestone title linked to `/milestones/<slug>`
- Feature count badge (`N features`)
- Status badge (same `StatusBadge` component used by ArchiveZone)
- Muted/low-contrast styling to signal "not yet started"

**Ponder rows** (active ponders):
- Ponder title linked to `/ponder/<slug>`
- Status badge (`exploring` / `converging`)
- Tag list (first 2-3 tags, clipped)
- `/sdlc-ponder <slug>` copy command (using the copy-button pattern)

**Layout**: Same compact card style as `ArchiveZone` — a single `bg-card border
border-border rounded-xl` container with a header and a `divide-y` list inside.
Two sub-sections (milestones, ponders) within the same card, or two separate cards
if that looks cleaner.

**Section header**: "Horizon" label with a `Telescope` or `Binoculars` lucide icon.

## Behaviour

- Zone 3 is omitted (returns `null`) when there are no horizon milestones and no
  active ponders.
- Ponder data is fetched once on mount — no polling. SSE events from the ponder
  channel could trigger a refetch, but for the initial implementation a single fetch
  is sufficient. The user can navigate away and back to refresh.
- Links use `react-router-dom` `<Link>` — no full page navigation.
- No new API endpoints — uses `api.getRoadmap()` which already exists.

## Dashboard.tsx Changes

Pass the required props to `<HorizonZone>`:

```tsx
<HorizonZone
  milestones={activeMilestones}
  featureBySlug={featureBySlug}
/>
```

The `featureBySlug` map already exists in Dashboard.tsx. The `activeMilestones` array
already exists. No new state or effects needed in Dashboard.tsx.

## Scope

**In scope**:
- `HorizonZone` component — full implementation replacing the stub
- Dashboard.tsx prop threading
- Ponder links route to `/ponder/<slug>` (already navigable in the app)

**Out of scope**:
- Clicking a ponder to start a chat session (use the Ponder page for that)
- SSE-driven ponder list refresh (the roadmap doesn't emit SSE events currently)
- Sorting / filtering controls
- Mobile-specific layout changes beyond what the existing card/list pattern already handles
- Any new API endpoints or backend changes
