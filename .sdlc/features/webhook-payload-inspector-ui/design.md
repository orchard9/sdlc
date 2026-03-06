# Design: Webhook Payload Inspector UI

## Overview

This feature adds payload inspection capabilities to the existing ActionsPage webhook section. The design extends the current `WebhookRoutesSection` with store-only indicators and an expandable inspector panel inline below the routes table, avoiding navigation away from the ActionsPage.

## Component Architecture

```
ActionsPage
  +-- WebhookRoutesSection (extended)
  |     +-- route row: [Path] [Tool] [Template] [store-only chip] [secret icon] [Inspect btn] [Delete btn]
  |     +-- AddRouteModal (existing)
  +-- WebhookPayloadInspector (new, conditionally rendered below routes table)
  |     +-- TimeRangeBar (quick chips + custom range)
  |     +-- PayloadList (left pane, scrollable)
  |     |     +-- PayloadListItem (timestamp, content-type, size)
  |     +-- PayloadDetail (right pane)
  |           +-- metadata header (id, received_at, content_type)
  |           +-- JSON viewer (syntax-highlighted <pre>)
  |           +-- CopyButton + ReplayButton
  |           +-- ReplayResultStrip (success/failure footer)
  +-- WebhookEventsSection (existing, unchanged)
```

## Data Flow

### Querying payloads

```
User clicks "Inspect" on route
  -> WebhookRoutesSection sets inspectingRoute state
  -> WebhookPayloadInspector renders below routes table
  -> calls api.queryWebhookPayloads(route.path, { since, until, limit })
  -> GET /api/webhooks/{route}/data?since=...&until=...&limit=50
  -> displays payload list
```

### Replaying a payload

```
User clicks "Replay" on a selected payload
  -> calls api.replayWebhookPayload(route.path, payload.id)
  -> POST /api/webhooks/{route}/replay/{id}
  -> backend loads stored payload, dispatches through tool
  -> returns { ok: true, run_id: "..." } or { ok: false, error: "..." }
  -> ReplayResultStrip shows result inline
```

## API Additions

### Frontend client (client.ts)

```typescript
queryWebhookPayloads: (route: string, params?: { since?: string; until?: string; limit?: number }) =>
  request<WebhookPayloadItem[]>(`/api/webhooks/${encodeURIComponent(route)}/data`, { params })

replayWebhookPayload: (route: string, id: string) =>
  request<{ ok: boolean; run_id?: string; error?: string }>(
    `/api/webhooks/${encodeURIComponent(route)}/replay/${encodeURIComponent(id)}`,
    { method: 'POST' }
  )
```

### Backend endpoint (webhooks.rs)

```rust
pub async fn replay_webhook(
    State(app): State<AppState>,
    Path((route, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError>
```

- Loads payload by UUID from `query_webhooks` or a new `get_webhook_by_id` method
- Validates payload.route_path matches the route parameter
- Finds the registered route, gets tool_name + input_template
- Dispatches via the existing tool dispatch path
- Returns JSON result

## Type Changes

### OrchestratorWebhookRoute (types.ts)

```typescript
export interface OrchestratorWebhookRoute {
  id: string
  path: string
  tool_name: string
  input_template: string
  created_at: string
  store_only: boolean          // NEW
  secret_token?: string | null // NEW (always "***" or null from backend)
}
```

### New types (types.ts)

```typescript
export interface WebhookPayloadItem {
  id: string
  received_at: string
  content_type: string | null
  body: unknown  // parsed JSON or base64 string
}
```

## UI Layout

### Routes table row (extended)

```
| /hooks/telegram | telegram-recap | {{payload}} | [Store-only] [Lock] | [Inspect] [Delete] |
```

- "Store-only" chip: `bg-blue-500/10 text-blue-400 text-xs px-2 py-0.5 rounded` -- only shown when `store_only === true`
- Lock icon: small lock icon next to path when `secret_token` is present
- "Inspect" button: only shown on `store_only` routes (dispatch routes don't accumulate payloads)

### Inspector panel (two-pane)

```
+------------------------------------------+
| Time: [1h] [6h] [24h] [7d] [Custom]  [X] |
+-------------------+----------------------+
| Payload list      | Payload detail       |
| - 2026-03-05 12:  | ID: abc-def-123      |
|   application/json| Received: 12:34:56   |
|   1.2 KB         | Content-Type: app/json|
| - 2026-03-05 11:  |                      |
|   application/json| { "update_id": 123,  |
|   0.8 KB  (sel)  |   "message": { ... } |
|                   |   ...                |
|                   | }                    |
|                   |                      |
|                   | [Copy] [Replay]      |
+-------------------+----------------------+
| Replay result: OK, run_id: xyz-789       |
+------------------------------------------+
```

- Left pane: 40% width, scrollable, max-height ~400px
- Right pane: 60% width, `<pre>` block with `overflow-auto`
- Time range bar: flex row of chip buttons, default "24h"
- Close button [X] in top-right of time range bar

## State Management

All state is local to the components (useState). No global state needed.

- `inspectingRoute: OrchestratorWebhookRoute | null` -- in WebhookRoutesSection
- `payloads: WebhookPayloadItem[]` -- in WebhookPayloadInspector
- `selectedPayload: WebhookPayloadItem | null` -- in WebhookPayloadInspector
- `timeRange: '1h' | '6h' | '24h' | '7d' | 'custom'` -- in TimeRangeBar
- `replayResult: { ok: boolean; run_id?: string; error?: string } | null` -- in PayloadDetail
- `isReplaying: boolean` -- loading state for replay button

## Error Handling

- Failed payload query: show inline error message in the inspector panel
- Failed replay: show error in ReplayResultStrip with red styling
- Empty results: "No payloads found in this time window" message
- Backend unavailable: standard API error handling from client.ts

## File Changes Summary

| File | Change |
|------|--------|
| `frontend/src/lib/types.ts` | Add fields to `OrchestratorWebhookRoute`, add `WebhookPayloadItem` |
| `frontend/src/api/client.ts` | Add `queryWebhookPayloads`, `replayWebhookPayload` methods |
| `frontend/src/pages/ActionsPage.tsx` | Extend `WebhookRoutesSection` with indicators + inspector toggle |
| `frontend/src/components/webhooks/WebhookPayloadInspector.tsx` | New component |
| `crates/sdlc-server/src/routes/webhooks.rs` | Add `replay_webhook` handler |
| `crates/sdlc-server/src/lib.rs` | Register replay route |
