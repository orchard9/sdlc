# Tasks: Dashboard WhatChangedBanner

## T1 — Create `useChangelog` hook

**File**: `frontend/src/hooks/useChangelog.ts`

Create the hook that manages all changelog data fetching and localStorage state:
- Read `localStorage.getItem('sdlc_last_visit_at')` on mount to get `lastVisitAt`
- Compute `since`: if `lastVisitAt` set → use it; else → `new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString()`
- Fetch `GET /api/changelog?since=<since>&limit=50`
- On HTTP 404 → silently return `{ events: [], total: 0, loading: false }`; do NOT throw
- Subscribe to SSE via `useSSE(refresh)` so banner re-fetches on any state update
- Expose `dismiss()` function: sets `localStorage.setItem('sdlc_last_visit_at', new Date().toISOString())` and sets local `dismissed = true` state
- Return `{ events, total, lastVisitAt, loading, dismissed, dismiss }`

**Types to define in the hook file**:
```typescript
export interface ChangeEvent {
  id: string
  kind: EventKind
  slug: string
  title: string
  timestamp: string
}

export type EventKind =
  | 'feature_merged'
  | 'run_failed'
  | 'milestone_wave_completed'
  | 'feature_phase_advanced'
  | 'review_approved'
  | 'audit_approved'
  | 'qa_approved'
```

## T2 — Implement WhatChangedBanner component with expand behavior

**File**: `frontend/src/components/layout/WhatChangedBanner.tsx`

Create the banner component:
- Calls `useChangelog()`
- If `loading` → render a single `<Skeleton />` row inside a padded container (matches existing skeleton pattern)
- If `events.length === 0` or `dismissed` → render `null`
- Sort events: `run_failed` first (desc timestamp), then rest (desc timestamp)
- `VISIBLE_COUNT = 7`
- State: `const [expanded, setExpanded] = useState(false)`
- Show `expanded ? sorted : sorted.slice(0, VISIBLE_COUNT)`
- If `!expanded && sorted.length > VISIBLE_COUNT` → show "See {sorted.length - VISIBLE_COUNT} more" button that sets `expanded = true`
- Header:
  - First visit (`lastVisitAt === null`): `"Recent project activity"` with `ClipboardList` icon
  - Returning user: `"{total} changes since {relativeTime(lastVisitAt)}"` with `Clock` icon
- Dismiss button (both modes): `[Dismiss]` text button, calls `dismiss()`

**Relative time helper** (inline, no library):
```typescript
function relativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 60) return `${minutes} minute${minutes === 1 ? '' : 's'} ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} hour${hours === 1 ? '' : 's'} ago`
  const days = Math.floor(hours / 24)
  return `${days} day${days === 1 ? '' : 's'} ago`
}
```

## T3 — Implement dismiss: sets last_visit_at in localStorage; NO beforeunload handler

**Covered by T1** (the `dismiss()` function in `useChangelog`).

Explicit constraint: do NOT add any `beforeunload` event listener. The `last_visit_at` key in localStorage is ONLY updated by the `dismiss()` function call, never automatically on navigation or tab close. This is already captured in the T1 implementation.

No separate code needed — this task verifies and documents the constraint is honored in code review.

## T4 — Wire run_failed events to link to /runs; wire feature_merged to /features/<slug>

**In `WhatChangedBanner.tsx`** — within the event list render:

```typescript
function EventRow({ event }: { event: ChangeEvent }) {
  const link = event.kind === 'run_failed'
    ? '/runs'
    : event.kind === 'feature_merged'
      ? `/features/${event.slug}`
      : null

  const content = (
    <div className="flex items-start gap-2 py-1">
      <EventIcon kind={event.kind} />
      <span className="font-mono text-muted-foreground bg-muted px-1.5 py-0.5 rounded text-[10px]">
        {event.kind}
      </span>
      <span className="text-xs text-muted-foreground">·</span>
      <span className="text-xs font-mono">{event.slug}</span>
      <span className="text-xs text-muted-foreground truncate">{event.title}</span>
    </div>
  )

  if (link) {
    return <Link to={link} className="block hover:bg-accent/50 rounded -mx-1 px-1 transition-colors">{content}</Link>
  }
  return <div>{content}</div>
}
```

Icons:
- `run_failed` → `AlertTriangle` (amber)
- `feature_merged` → `Rocket` (green)
- `review_approved` | `audit_approved` | `qa_approved` → `Check` (primary color)
- `feature_phase_advanced` → `ArrowRight` (muted)
- `milestone_wave_completed` → `Layers` (blue)

## T5 — Add WhatChangedBanner to Dashboard page above main content zones

**File**: `frontend/src/pages/Dashboard.tsx`

Add import and render:
```typescript
import { WhatChangedBanner } from '@/components/layout/WhatChangedBanner'
```

Placement in JSX (inside `<div className="max-w-5xl mx-auto p-4 sm:p-6">`):
```tsx
{/* Vision/Architecture missing banner */}
{missingVisionOrArch && (...)}

{/* What Changed banner */}
<WhatChangedBanner />

{/* Project Overview */}
<div className="mb-6">
```

## T6 — [user-gap] First visit mode: show "Recent project activity" for new developers

**Covered by T1 and T2** — the `lastVisitAt === null` branch in `useChangelog` fetches last 7 days, and `WhatChangedBanner` renders the "Recent project activity" header when `lastVisitAt === null`.

Verify: when there is no `sdlc_last_visit_at` key in localStorage, the header says "Recent project activity" and events shown are from the last 7 days.

## T7 — [user-gap] SPA navigation must NOT update last_visit_at

**Covered by T1** — no `beforeunload` handler, no React Router `useNavigate`/`useLocation` effects that update localStorage. The hook reads `lastVisitAt` only on mount and updates it only via `dismiss()`.

Verify: clicking dashboard menu items or navigating between routes does NOT update `localStorage.getItem('sdlc_last_visit_at')`.
