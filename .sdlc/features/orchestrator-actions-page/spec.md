# Spec: Orchestrator Actions Page

## Summary

Add a full-featured Actions page to the SDLC frontend that surfaces the orchestrator's scheduled actions, webhook routes, and webhook event history. The page lives under Setup in the sidebar nav and provides real-time status via SSE.

## Background

The orchestrator backend is complete: `Action` model, `ActionDb` (redb), tick daemon CLI, and webhook ingestion + routing REST API. The frontend has zero UI for it. Users currently have no way to see what actions are scheduled, whether they succeeded or failed, which webhook routes are active, or what recent webhook events arrived — without dropping to the CLI.

This feature delivers the complete Actions page with three stacked sections, sidebar navigation, SSE real-time updates, and action editing.

## Scope

This feature covers only the **frontend page** and the **minimal backend additions** needed to serve it. The `orchestrator-actions-routes` feature covers the Rust REST routes (`GET /api/orchestrator/actions`, `POST /api/orchestrator/actions`, `PATCH /api/orchestrator/actions/:id`, `GET /api/orchestrator/webhooks/events`) and the new `WEBHOOK_EVENTS` redb table.

This feature assumes those routes are available before or alongside implementation.

## Functional Requirements

### Page Layout

`ActionsPage.tsx` renders three stacked sections (not tabs):

1. **Scheduled Actions** — list of all `Action` records from `GET /api/orchestrator/actions`
2. **Webhook Routes** — list of all `WebhookRoute` records from `GET /api/orchestrator/webhooks/routes`
3. **Recent Webhook Events** — log of the last 20 `WebhookEvent` records from `GET /api/orchestrator/webhooks/events`

### Section 1: Scheduled Actions

- Header: "Scheduled Actions" + `[+ Schedule Action]` button (right-aligned)
- Table columns: Label | Tool | Status | Next Run | Recurrence | Actions (edit pencil + delete trash)
- Status badge colors:
  - `Pending` — gray
  - `Running` — blue (pulsing)
  - `Completed` — green
  - `Failed` — red
- Recurrence column: display "every 1h" (computed from `recurrence_secs`), or "—" if null
- Next Run column: formatted as relative time ("in 5m", "2h ago") for `Scheduled` trigger, "webhook-triggered" for `Webhook` trigger
- Empty state: `"No actions scheduled. Use the CLI: sdlc orchestrate add"`
- Delete icon: calls `DELETE /api/orchestrator/actions/:id` — no confirmation required (consistent with other delete patterns in the UI)
- Edit pencil: opens the Edit Action modal

### Section 2: Webhook Routes

- Header: "Webhook Routes" + `[+ Add Route]` button (right-aligned)
- Table columns: Path | Tool | Input Template (truncated at 60 chars with ellipsis) | Created | Delete
- Empty state: `"No webhook routes configured."`
- Delete icon: calls `DELETE /api/orchestrator/webhooks/routes/:id`
- `[+ Add Route]` opens the Add Webhook Route modal

### Section 3: Recent Webhook Events

- Header: "Recent Webhook Events"
- Table columns: Time | Path | Outcome | Action
  - Time: relative ("2m ago"), tooltip with absolute timestamp
  - Path: `/hooks/github`
  - Outcome badge: Dispatched=green, NoRouteMatched=gray, Rejected=red
  - Action: link to the triggered action row (by `action_id`) if present, otherwise "—"
- Default: show 20 events
- "Load more" link appears if the response contains 20 events (implying more may exist)
- Empty state: `"No webhook events recorded. Events appear here after the first webhook request arrives."`

### Modals

#### Schedule Action Modal

Fields:
- Label (text, required)
- Tool (select from `GET /api/tools`, required)
- Tool Input (JSON textarea, defaults to `{}`)
- Scheduled At (datetime-local, required, defaults to now + 1 minute)
- Recurrence (text input, optional — accepts "10s", "30m", "1h", "6h", "24h"; validated with regex `/^(\d+)(s|m|h|d)$/`)

Behavior:
- Inline validation error under Recurrence if input does not match the pattern
- Submit: `POST /api/orchestrator/actions` with `recurrence_secs` derived from parsed recurrence (or null if empty)
- On success: close modal, refresh actions list

#### Edit Action Modal

Pre-populated from the selected action row. Fields:
- Label (text, required)
- Recurrence (text input — same validation as above)

Behavior:
- Submit: `PATCH /api/orchestrator/actions/:id` with `{ label, recurrence_secs }` — `recurrence_secs: null` clears recurrence
- Optimistic update: update row immediately, revert on error with inline error message
- On success: close modal

#### Add Webhook Route Modal

Fields:
- Path (text, required — must start with `/`)
- Tool (select from `GET /api/tools`, required)
- Input Template (textarea, required — e.g. `{"payload": "{{payload}}"}`)

Behavior:
- Submit: `POST /api/orchestrator/webhooks/routes`
- On `409 Conflict`: show inline error "A route with this path already exists"
- On success: close modal, refresh routes list

### Sidebar Navigation

Add to the `setup` group in `Sidebar.tsx`, after Agents:

```tsx
{ path: '/actions', label: 'Actions', icon: Zap, exact: false }
```

Note: `Zap` is already imported in `Sidebar.tsx` (used for the "Fix Right Away" button). The import needs a separate named entry for the nav item.

### App.tsx Route

```tsx
<Route path="/actions" element={<ActionsPage />} />
```

### SSE Real-Time Updates

The Actions page subscribes to the global SSE stream. When `SseMessage::ActionStateChanged` is received, refetch `GET /api/orchestrator/actions`. This is the fast path; the page also polls every 5 seconds as a fallback (in case daemon tick events are debounced or missed by the watcher).

The SSE event arrives via the existing `SseContext`. A new event key `"action_state_changed"` is added to the SSE event type union and handled by `ActionsPage`.

### Recurrence Parsing

Frontend utility functions (co-located with `ActionsPage.tsx` or in a `lib/recurrence.ts` file):

```typescript
function parseRecurrence(s: string): number | null {
  const m = s.trim().match(/^(\d+)(s|m|h|d)$/)
  if (!m) return null
  const n = parseInt(m[1], 10)
  const units: Record<string, number> = { s: 1, m: 60, h: 3600, d: 86400 }
  return n * units[m[2]]
}

function formatRecurrence(secs: number): string {
  if (secs % 86400 === 0) return `${secs / 86400}d`
  if (secs % 3600 === 0) return `${secs / 3600}h`
  if (secs % 60 === 0) return `${secs / 60}m`
  return `${secs}s`
}
```

## API Client Methods

Add to `client.ts` (or the project's API client module):

```typescript
api.listActions()                           // GET /api/orchestrator/actions
api.createAction(body)                      // POST /api/orchestrator/actions
api.updateAction(id, patch)                 // PATCH /api/orchestrator/actions/:id
api.deleteAction(id)                        // DELETE /api/orchestrator/actions/:id
api.listWebhookRoutes()                     // GET /api/orchestrator/webhooks/routes
api.createWebhookRoute(body)                // POST /api/orchestrator/webhooks/routes
api.deleteWebhookRoute(id)                  // DELETE /api/orchestrator/webhooks/routes/:id
api.listWebhookEvents(limit?)               // GET /api/orchestrator/webhooks/events?limit=20
```

## TypeScript Types

```typescript
interface OrchestratorAction {
  id: string
  label: string
  tool_name: string
  tool_input: unknown
  trigger: { type: 'scheduled'; next_tick_at: string } | { type: 'webhook'; received_at: string }
  status: { type: 'pending' } | { type: 'running' } | { type: 'completed'; result: unknown } | { type: 'failed'; reason: string }
  recurrence_secs: number | null
  created_at: string
  updated_at: string
}

interface WebhookRoute {
  id: string
  path: string
  tool_name: string
  input_template: string
  created_at: string
}

interface WebhookEvent {
  id: string
  path: string
  received_at: string
  action_id: string | null
  outcome: { type: 'dispatched' } | { type: 'no_route_matched' } | { type: 'rejected'; reason: string }
}
```

## Out of Scope

- Webhook payload storage/inspection — no payload content is persisted on the backend
- Complex recurrence UI (dropdowns) — free-text input covers all cases
- Action status history / audit log beyond the single current status
- Pagination beyond "Load more" on webhook events

## Acceptance Criteria

1. Sidebar shows "Actions" under Setup, navigates to `/actions`
2. Actions page renders three stacked sections without errors
3. Status badges render with correct colors for all four `ActionStatus` variants
4. `[+ Schedule Action]` modal creates an action via POST and refreshes the list
5. Edit modal PATCHes label/recurrence and reflects changes without full page reload
6. `[+ Add Route]` modal creates a webhook route via POST; 409 shows inline error
7. Webhook events section shows last 20 events with correct outcome badge colors
8. Recurrence text input accepts "10s", "30m", "1h", "6h", "24h" and rejects "foo"
9. SSE `ActionStateChanged` events trigger a refetch of the actions list
10. 5-second poll fallback runs in background regardless of SSE events
