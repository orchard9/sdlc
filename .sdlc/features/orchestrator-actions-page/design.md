# Design: Orchestrator Actions Page

## Component Architecture

```
ActionsPage.tsx
├── useActionsData() hook          — fetches + manages state for all three sections
├── ScheduledActionsSection        — renders section 1 table + add/edit modals
│   ├── ActionStatusBadge          — status color chip (Pending/Running/Completed/Failed)
│   ├── ScheduleActionModal        — POST /api/orchestrator/actions
│   └── EditActionModal            — PATCH /api/orchestrator/actions/:id
├── WebhookRoutesSection           — renders section 2 table + add modal
│   └── AddRouteModal              — POST /api/orchestrator/webhooks/routes
└── WebhookEventsSection           — renders section 3 events log + "Load more"
    └── OutcomeBadge               — Dispatched/NoRouteMatched/Rejected colors
```

All sub-components are co-located in `frontend/src/pages/ActionsPage.tsx` (single file, no `components/` subdirectory — consistent with similar pages like `AgentsPage.tsx`).

## Data Model

New TypeScript types added to `frontend/src/lib/types.ts`:

```typescript
export interface OrchestratorAction {
  id: string
  label: string
  tool_name: string
  tool_input: unknown
  trigger:
    | { type: 'scheduled'; next_tick_at: string }
    | { type: 'webhook'; received_at: string }
  status:
    | { type: 'pending' }
    | { type: 'running' }
    | { type: 'completed'; result: unknown }
    | { type: 'failed'; reason: string }
  recurrence_secs: number | null
  created_at: string
  updated_at: string
}

export interface OrchestratorWebhookRoute {
  id: string
  path: string
  tool_name: string
  input_template: string
  created_at: string
}

export interface OrchestratorWebhookEvent {
  id: string
  path: string
  received_at: string
  action_id: string | null
  outcome:
    | { type: 'dispatched' }
    | { type: 'no_route_matched' }
    | { type: 'rejected'; reason: string }
}
```

## API Client Additions

New methods appended to the `api` object in `frontend/src/api/client.ts`:

```typescript
// Orchestrator — Actions
listActions: () =>
  request<OrchestratorAction[]>('/api/orchestrator/actions'),
createAction: (body: {
  label: string
  tool_name: string
  tool_input: unknown
  scheduled_at: string
  recurrence_secs?: number | null
}) =>
  request<OrchestratorAction>('/api/orchestrator/actions', {
    method: 'POST',
    body: JSON.stringify(body),
  }),
updateAction: (id: string, patch: { label?: string; recurrence_secs?: number | null }) =>
  request<OrchestratorAction>(`/api/orchestrator/actions/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(patch),
  }),
deleteAction: (id: string) =>
  request<void>(`/api/orchestrator/actions/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  }),

// Orchestrator — Webhook Routes
listWebhookRoutes: () =>
  request<OrchestratorWebhookRoute[]>('/api/orchestrator/webhooks/routes'),
createWebhookRoute: (body: { path: string; tool_name: string; input_template: string }) =>
  request<OrchestratorWebhookRoute>('/api/orchestrator/webhooks/routes', {
    method: 'POST',
    body: JSON.stringify(body),
  }),
deleteWebhookRoute: (id: string) =>
  request<void>(`/api/orchestrator/webhooks/routes/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  }),

// Orchestrator — Webhook Events
listWebhookEvents: (limit = 20) =>
  request<OrchestratorWebhookEvent[]>(
    `/api/orchestrator/webhooks/events?limit=${limit}`
  ),
```

## SSE Integration

`SseContext.tsx` is extended with a new `onActionEvent` callback:

```typescript
// types.ts — new type
export type ActionSseEvent = { type: 'action_state_changed' }

// SseContext.tsx — add to SseCallbacks
onActionEvent?: (event: ActionSseEvent) => void
```

In `SseContext.tsx`'s `dispatch()` function, add a new branch:

```typescript
} else if (type === 'action') {
  try {
    const event = JSON.parse(data) as ActionSseEvent
    for (const sub of subs) sub.onActionEvent?.(event)
  } catch { /* malformed */ }
}
```

In `ActionsPage`, subscribe via `useSseContext`:

```typescript
const { subscribe } = useSseContext()
useEffect(() => {
  return subscribe({
    onActionEvent: () => { refetchActions() },
  })
}, [subscribe, refetchActions])
```

Additionally, a 5-second poll fallback runs via `useInterval` (or `useEffect` + `setInterval`):

```typescript
useEffect(() => {
  const interval = setInterval(refetchActions, 5000)
  return () => clearInterval(interval)
}, [refetchActions])
```

## Utility Functions

`frontend/src/lib/recurrence.ts` (new file):

```typescript
const UNITS: Record<string, number> = { s: 1, m: 60, h: 3600, d: 86400 }

export function parseRecurrence(s: string): number | null {
  const m = s.trim().match(/^(\d+)(s|m|h|d)$/)
  if (!m) return null
  return parseInt(m[1], 10) * UNITS[m[2]]
}

export function formatRecurrence(secs: number): string {
  if (secs % 86400 === 0) return `${secs / 86400}d`
  if (secs % 3600 === 0) return `${secs / 3600}h`
  if (secs % 60 === 0) return `${secs / 60}m`
  return `${secs}s`
}
```

## Sidebar Change

`frontend/src/components/layout/Sidebar.tsx` — add to the `setup` group, after the Agents entry:

```typescript
{ path: '/actions', label: 'Actions', icon: Zap, exact: false }
```

`Zap` is already imported (used for "Fix Right Away" bottom button). No new imports needed.

## App Router Change

`frontend/src/App.tsx` — add import and route:

```tsx
import { ActionsPage } from '@/pages/ActionsPage'
// ...
<Route path="/actions" element={<ActionsPage />} />
```

Route order: insert after the `/agents` route, before `/config`.

## Page Layout Wireframe

```
┌─────────────────────────────────────────────────────────────┐
│  Actions                                                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Scheduled Actions                      [+ Schedule Action] │
│  ┌─────────┬───────────┬────────┬──────────┬───────┬──────┐ │
│  │ Label   │ Tool      │ Status │ Next Run │ Recur │ Edit │ │
│  ├─────────┼───────────┼────────┼──────────┼───────┼──────┤ │
│  │ nightly │ quality-… │ ●Pend  │ in 4h    │ ev 6h │ ✏ 🗑 │ │
│  │ gh-sync │ git-sync  │ ✓Done  │ 2h ago   │ ev 1h │ ✏ 🗑 │ │
│  └─────────┴───────────┴────────┴──────────┴───────┴──────┘ │
│                                                             │
│  Webhook Routes                              [+ Add Route]  │
│  ┌────────────┬────────────┬──────────────────┬───┬──────┐  │
│  │ Path       │ Tool       │ Input Template   │Dt │ Del  │  │
│  ├────────────┼────────────┼──────────────────┼───┼──────┤  │
│  │ /hooks/gh  │ git-sync   │ {"payload": "{{… │ … │  🗑  │  │
│  └────────────┴────────────┴──────────────────┴───┴──────┘  │
│                                                             │
│  Recent Webhook Events                                      │
│  ┌──────────┬─────────────┬──────────────┬─────────────┐   │
│  │ Time     │ Path        │ Outcome      │ Action      │   │
│  ├──────────┼─────────────┼──────────────┼─────────────┤   │
│  │ 2m ago   │ /hooks/gh   │ ●Dispatched  │ nightly     │   │
│  │ 1h ago   │ /hooks/ci   │ ●No match    │ —           │   │
│  └──────────┴─────────────┴──────────────┴─────────────┘   │
│  Load more                                                  │
└─────────────────────────────────────────────────────────────┘
```

## Status Badge Colors

| Status        | Tailwind classes                                              |
|---------------|---------------------------------------------------------------|
| Pending       | `bg-muted text-muted-foreground border border-border`         |
| Running       | `bg-blue-500/10 text-blue-400 border border-blue-500/20 animate-pulse` |
| Completed     | `bg-green-500/10 text-green-400 border border-green-500/20`   |
| Failed        | `bg-red-500/10 text-red-400 border border-red-500/20`         |

## Outcome Badge Colors

| Outcome         | Tailwind classes                                            |
|-----------------|-------------------------------------------------------------|
| Dispatched      | `bg-green-500/10 text-green-400 border border-green-500/20` |
| NoRouteMatched  | `bg-muted text-muted-foreground border border-border`       |
| Rejected        | `bg-red-500/10 text-red-400 border border-red-500/20`       |

## Modal Design

All modals follow the existing modal pattern in the codebase (no modal library — inline `fixed inset-0` overlay with a centered card):

```tsx
{showModal && (
  <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
    <div className="bg-card border border-border rounded-xl p-6 w-full max-w-md shadow-xl">
      <h2 className="text-base font-semibold mb-4">Schedule Action</h2>
      {/* fields */}
      <div className="flex gap-2 justify-end mt-6">
        <button onClick={close} className="...">Cancel</button>
        <button onClick={submit} className="...">Create</button>
      </div>
    </div>
  </div>
)}
```

## Data Fetching Strategy

`useActionsData()` custom hook manages all three fetch streams:

```typescript
function useActionsData() {
  const [actions, setActions] = useState<OrchestratorAction[]>([])
  const [routes, setRoutes] = useState<OrchestratorWebhookRoute[]>([])
  const [events, setEvents] = useState<OrchestratorWebhookEvent[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchAll = useCallback(async () => {
    const [a, r, e] = await Promise.allSettled([
      api.listActions(),
      api.listWebhookRoutes(),
      api.listWebhookEvents(20),
    ])
    // set state from settled results; missing routes returns empty array on 503
    setLoading(false)
  }, [])

  const refetchActions = useCallback(() => {
    api.listActions().then(setActions).catch(() => {})
  }, [])

  useEffect(() => { fetchAll() }, [fetchAll])

  return { actions, routes, events, loading, error, refetchActions, fetchAll }
}
```

`Promise.allSettled` is used so a 503 on one endpoint (DB unavailable) does not block the rest.

## Error Handling

- If the orchestrator DB is unavailable (`503`), show a yellow warning banner at the top of the page: `"Orchestrator DB unavailable — start the daemon to enable full functionality."` Sections still render with empty state (not an error state).
- DELETE errors: show inline toast or row-level error text (revert optimistic removal).
- PATCH errors: revert optimistic update, show inline modal error.
- POST 409 on webhook route: show error below the Path field: `"A route with this path already exists."`

## Files Changed

| File | Change |
|------|--------|
| `frontend/src/pages/ActionsPage.tsx` | **New file** — full page implementation |
| `frontend/src/lib/recurrence.ts` | **New file** — parseRecurrence, formatRecurrence |
| `frontend/src/lib/types.ts` | Add `OrchestratorAction`, `OrchestratorWebhookRoute`, `OrchestratorWebhookEvent`, `ActionSseEvent` |
| `frontend/src/api/client.ts` | Add 7 new orchestrator API methods |
| `frontend/src/contexts/SseContext.tsx` | Add `onActionEvent` callback + `action` dispatch branch |
| `frontend/src/components/layout/Sidebar.tsx` | Add Actions nav entry to `setup` group |
| `frontend/src/App.tsx` | Add `/actions` route + `ActionsPage` import |
