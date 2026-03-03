# Design: Dashboard WhatChangedBanner

## Overview

This is a pure frontend feature — a new React component added to the Dashboard page. It reads from `localStorage` to determine the user's last-dismissed timestamp, fetches events from `GET /api/changelog`, and renders a dismissible summary banner. No backend changes are required beyond what `changelog-api` and `changelog-core` provide.

## Component Architecture

```
Dashboard.tsx
└── WhatChangedBanner        (new: frontend/src/components/layout/WhatChangedBanner.tsx)
    └── useChangelog hook    (new: frontend/src/hooks/useChangelog.ts)
```

### `useChangelog` Hook

```typescript
// frontend/src/hooks/useChangelog.ts

const STORAGE_KEY = 'sdlc_last_visit_at'

interface ChangeEvent {
  id: string
  kind: EventKind
  slug: string
  title: string
  timestamp: string
}

type EventKind =
  | 'feature_merged'
  | 'run_failed'
  | 'milestone_wave_completed'
  | 'feature_phase_advanced'
  | 'review_approved'
  | 'audit_approved'
  | 'qa_approved'

interface ChangelogResult {
  events: ChangeEvent[]
  total: number
  lastVisitAt: string | null   // from localStorage, null = first visit
  loading: boolean
  dismiss: () => void          // sets last_visit_at = now, collapses banner
}
```

The hook:
1. Reads `localStorage.getItem(STORAGE_KEY)` on mount
2. Derives `since`: if `lastVisitAt` is set → use it; else → `new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString()`
3. Fetches `GET /api/changelog?since=<since>&limit=50`
4. On 404 (changelog not implemented yet) → returns `{ events: [], total: 0 }` silently
5. Subscribes to SSE via `useSSE` — on `ChangelogUpdated` event, re-fetches
6. `dismiss()` writes `new Date().toISOString()` to `localStorage[STORAGE_KEY]`

**Critical**: The hook does NOT register any `beforeunload` listener — that would cause `last_visit_at` to update on tab close, defeating the "only dismiss button sets it" requirement. SPA navigation also does not affect `last_visit_at`.

### `WhatChangedBanner` Component

**Props**: none (self-contained, reads own state via hook)

**Render logic**:
```
if loading → render skeleton row
if events.length === 0 → render null (no banner)
else → render banner
```

**Banner layout** (two mode variants):

**Returning user mode** (`lastVisitAt !== null`):
```
┌─────────────────────────────────────────────────────────────────────┐
│ 🕐  12 changes since 2 hours ago                        [Dismiss]   │
│                                                                      │
│  ⚠️  run_failed  · my-feature  · Agent run failed                   │
│  ⚠️  run_failed  · other-feature  · Agent run failed                │
│  🚀  feature_merged  · changelog-core  · Changelog event log core   │
│  ✓  review_approved  · changelog-cli  · Changelog CLI               │
│  →  feature_phase_advanced  · changelog-api  · Changelog REST…      │
│  ✓  qa_approved  · some-feature  · Some Feature Title               │
│  ✓  audit_approved  · other  · Other Feature                        │
│  See 5 more                                                          │
└─────────────────────────────────────────────────────────────────────┘
```

**First visit mode** (`lastVisitAt === null`):
```
┌─────────────────────────────────────────────────────────────────────┐
│ 📋  Recent project activity                             [Dismiss]   │
│                                                                      │
│  (same event list, last 7 days)                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Event Sorting

```typescript
function sortEvents(events: ChangeEvent[]): ChangeEvent[] {
  const failed = events.filter(e => e.kind === 'run_failed')
    .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
  const rest = events.filter(e => e.kind !== 'run_failed')
    .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
  return [...failed, ...rest]
}
```

### Event Icons & Links

| kind | Icon | Link |
|---|---|---|
| `run_failed` | ⚠️ (AlertTriangle, amber) | `/runs` |
| `feature_merged` | 🚀 (Rocket, green) | `/features/<slug>` |
| `review_approved` | ✓ (Check, primary) | none |
| `audit_approved` | ✓ (Check, primary) | none |
| `qa_approved` | ✓ (Check, primary) | none |
| `feature_phase_advanced` | → (ArrowRight, muted) | none |
| `milestone_wave_completed` | ◈ (Layers, blue) | none |

### "See X more" Expansion

- Initially show `VISIBLE_COUNT = 7` events
- State: `const [expanded, setExpanded] = useState(false)`
- When `!expanded && sorted.length > VISIBLE_COUNT`: show first 7 + "See X more" button
- When `expanded`: show all events (no collapse button needed)

### Dismiss Behavior

```typescript
const dismiss = useCallback(() => {
  const now = new Date().toISOString()
  localStorage.setItem(STORAGE_KEY, now)
  // Component re-renders; since events is now empty for the new "since",
  // banner will be hidden on next fetch. We optimistically collapse.
}, [])
```

After dismiss, the hook should hide the banner immediately (optimistic) — set local `dismissed` state to `true` which causes `null` render.

## Styling

Follows existing Dashboard banner patterns (Vision/Architecture missing banner):
- Container: `bg-primary/5 border border-primary/20 rounded-xl p-4 mb-6`
- Header row: `flex items-center justify-between`
- Header text: `text-sm font-semibold` with icon `w-4 h-4 text-primary`
- Dismiss button: `text-xs text-muted-foreground hover:text-foreground transition-colors`
- Event rows: `flex items-start gap-2 py-1 text-xs`
- Kind badge: `font-mono text-muted-foreground bg-muted px-1.5 py-0.5 rounded text-[10px]`
- Linked text: `hover:text-primary transition-colors`
- "See X more" button: `text-xs text-primary hover:underline mt-1`
- Skeleton: single `<Skeleton />` row while loading

## SSE Integration

The `useChangelog` hook uses `useSSE` — specifically, it needs a new `onChangelogEvent` callback or can piggyback on the generic `onUpdate` callback. Since `ChangelogUpdated` is a new SSE event type not currently in `SseContext`, we have two options:

**Option A** (simpler): Subscribe to the generic `onUpdate` SSE callback — re-fetch on any state update. This is fine for now since changelog events come from the same change stream.

**Option B** (precise): Add `ChangelogSseEvent` type and `onChangelogEvent` callback to `SseContext`. Required if we want to re-fetch only on changelog changes.

**Decision**: Use Option A for this feature. The `useChangelog` hook passes `refresh` as the `onUpdate` callback to `useSSE`. If a more targeted event is needed later, that's a follow-up enhancement.

## Placement in Dashboard

```tsx
// Dashboard.tsx — inside the returned JSX
<div className="max-w-5xl mx-auto p-4 sm:p-6">
  {/* Vision/Architecture missing banner */}
  {missingVisionOrArch && (...)}

  {/* What Changed banner — NEW */}
  <WhatChangedBanner />

  {/* Project Overview */}
  <div className="mb-6">...
```

## Graceful Degradation

If `GET /api/changelog` returns 404 (feature not deployed yet):
- `useChangelog` catches the error and returns `events: [], total: 0`
- `WhatChangedBanner` renders `null`
- No error state visible to the user — silent graceful absence

## Files to Create/Modify

| File | Change |
|---|---|
| `frontend/src/hooks/useChangelog.ts` | New hook |
| `frontend/src/components/layout/WhatChangedBanner.tsx` | New component |
| `frontend/src/pages/Dashboard.tsx` | Import + render `<WhatChangedBanner />` |

## ASCII Wireframe: Returning User

```
╔══════════════════════════════════════════════════════════════════════╗
║  🕐  8 changes since 3 hours ago                        [Dismiss ×] ║
║  ─────────────────────────────────────────────────────────────────  ║
║  ⚠ run_failed      · changelog-core  · Agent run failed             ║
║  🚀 feature_merged · telemetry-ts    · Telemetry wallclock…         ║
║  ✓ review_approved · concurrency-hm  · Concurrency heatmap          ║
║  → feature_phase_advanced · changelog-api · Changelog REST          ║
║  ✓ audit_approved  · telemetry-ts    · Telemetry wallclock           ║
║  ✓ qa_approved     · telemetry-ts    · Telemetry wallclock           ║
║  ✓ review_approved · changelog-core  · Changelog event log           ║
║  See 1 more ↓                                                        ║
╚══════════════════════════════════════════════════════════════════════╝
```

## ASCII Wireframe: First Visit

```
╔══════════════════════════════════════════════════════════════════════╗
║  📋  Recent project activity                            [Dismiss ×] ║
║  ─────────────────────────────────────────────────────────────────  ║
║  🚀 feature_merged · telemetry-ts  · Telemetry wallclock timestamps ║
║  🚀 feature_merged · concurrency-hm · Concurrency heatmap           ║
║  ✓ review_approved · telemetry-ts  · Telemetry wallclock             ║
║  ✓ audit_approved  · telemetry-ts  · Telemetry wallclock             ║
╚══════════════════════════════════════════════════════════════════════╝
```
