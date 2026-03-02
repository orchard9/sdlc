# Design: SSE State Consistency — UatHistoryPanel and SettingsPage gaps

## Overview

Three focused changes across five files. No new backend routes. No new SSE
channels on the Rust side beyond what already exists (`milestone_uat`). The
work is purely frontend wiring.

## Change 1 — Add `MilestoneUatSseEvent` type (`types.ts`)

```typescript
export interface MilestoneUatSseEvent {
  type: 'milestone_uat_completed'
  slug: string
}
```

The server already emits this event on the `milestone_uat` channel
(see `events.rs` line 141–148):

```json
{ "type": "milestone_uat_completed", "slug": "<milestone-slug>" }
```

## Change 2 — Handle `milestone_uat` channel in `SseContext.tsx`

Add `onMilestoneUatEvent` to `SseCallbacks` and to the `dispatch` function:

```typescript
interface SseCallbacks {
  // ... existing fields ...
  onMilestoneUatEvent?: (event: MilestoneUatSseEvent) => void
}

// In dispatch():
} else if (type === 'milestone_uat') {
  try {
    const event = JSON.parse(data) as MilestoneUatSseEvent
    for (const sub of subs) sub.onMilestoneUatEvent?.(event)
  } catch { /* malformed */ }
}
```

This follows the identical pattern used for `ponder`, `investigation`, `docs`,
and `advisory` channels.

## Change 3 — Add `onMilestoneUatEvent` parameter to `useSSE.ts`

```typescript
export function useSSE(
  onUpdate: () => void,
  onPonderEvent?: (event: PonderSseEvent) => void,
  onRunEvent?: (event: RunSseEvent) => void,
  onInvestigationEvent?: (event: InvestigationSseEvent) => void,
  onDocsEvent?: (event: DocsSseEvent) => void,
  onAdvisoryEvent?: (event: AdvisorySseEvent) => void,
  onMilestoneUatEvent?: (event: MilestoneUatSseEvent) => void,  // NEW
)
```

Add ref + wire through to `subscribe`, matching the existing pattern exactly.

## Change 4 — Wire `UatHistoryPanel.tsx` to SSE

Import `useSSE`. On `milestone_uat_completed` for the correct slug, re-fetch:

```typescript
import { useSSE } from '@/hooks/useSSE'

export function UatHistoryPanel({ milestoneSlug }: UatHistoryPanelProps) {
  const [runs, setRuns] = useState<UatRun[]>([])
  const [loading, setLoading] = useState(true)

  const load = useCallback(() => {
    api.listMilestoneUatRuns(milestoneSlug)
      .then(data => setRuns(sortRunsDescending(data)))
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [milestoneSlug])

  useEffect(() => { load() }, [load])

  useSSE(
    () => {},                      // no generic update subscription needed
    undefined,                     // no ponder events
    undefined,                     // no run events
    undefined,                     // no investigation events
    undefined,                     // no docs events
    undefined,                     // no advisory events
    (event) => {                   // milestone_uat events
      if (event.slug === milestoneSlug) load()
    },
  )
  // ...
}
```

By passing a no-op `onUpdate` and only a `onMilestoneUatEvent`, the panel
re-fetches precisely and only when a UAT run for *this* milestone completes.

## Change 5 — Fix `SettingsPage.tsx` error state on SSE refresh

Current `refresh`:

```typescript
const refresh = useCallback(() => {
  api.getConfig()
    .then(setConfig)
    .catch(err => setError(err.message))
}, [])
```

Fixed `refresh` — clear error before fetching, and clear it on success:

```typescript
const refresh = useCallback(() => {
  setError(null)
  api.getConfig()
    .then(data => { setConfig(data); setError(null) })
    .catch(err => setError(err.message))
}, [])
```

`setError(null)` before the call prevents a stale error from lingering while
the new fetch is in-flight. `setError(null)` in the `.then` is belt-and-
suspenders. No change to the SSE subscription — firing on every `update` event
is acceptable for the settings page.

## No-op for `milestone_uat` → `SseContext` generic `update` fallthrough

The `milestone_uat` channel is currently not handled, so events on it silently
vanish. After this change they are handled and routed. No other components
subscribe to `onMilestoneUatEvent` today, so there is no blast radius.

## Call-site compatibility

The new `onMilestoneUatEvent` parameter is optional and positioned last in the
`useSSE` signature. All existing call sites pass fewer arguments than the new
maximum; none are affected.

## File summary

| File | Lines changed (est.) |
|---|---|
| `frontend/src/lib/types.ts` | +5 |
| `frontend/src/contexts/SseContext.tsx` | +10 |
| `frontend/src/hooks/useSSE.ts` | +6 |
| `frontend/src/components/milestones/UatHistoryPanel.tsx` | +15 |
| `frontend/src/pages/SettingsPage.tsx` | +2 |

Total: ~38 lines net new.
