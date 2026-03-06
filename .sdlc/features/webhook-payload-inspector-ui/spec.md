# Spec: Webhook Payload Inspector UI

## Problem

The webhook query infrastructure (v42, wave 1) added `store_only` routes, `secret_token` verification, and a `GET /api/webhooks/{route}/data` query endpoint. However, the frontend has no way to:

1. **See which routes are store-only** -- the `OrchestratorWebhookRoute` type in the frontend lacks `store_only` and `secret_token` fields, so the ActionsPage renders them identically to dispatch routes.
2. **Browse stored payloads** -- there is no UI to call the query endpoint, view payload bodies, or filter by time window.
3. **Replay a stored payload** -- there is no way to take a stored payload and re-dispatch it through its route's tool for debugging or reprocessing.

## Solution

### 1. Extend frontend types to match the backend

Add `store_only: boolean` and `secret_token?: string` (always masked `"***"` or `null`) to `OrchestratorWebhookRoute` in `frontend/src/lib/types.ts`. The backend already returns these fields; the frontend just ignores them.

### 2. Add API client methods for payload query and replay

Add two methods to `frontend/src/api/client.ts`:

- `queryWebhookPayloads(route, params)` -- calls `GET /api/webhooks/{route}/data?since=&until=&limit=`
- `replayWebhookPayload(route, id)` -- calls `POST /api/webhooks/{route}/replay/{id}` (new backend endpoint)

### 3. Surface store-only status on ActionsPage

In the `WebhookRoutesSection` component, add visual indicators:

- A "Store-only" chip/badge on routes where `store_only === true`
- A "Secret" indicator (lock icon) on routes where `secret_token` is present
- An "Inspect" button on store-only routes that navigates to the payload inspector

### 4. Build the WebhookPayloadInspector component

A two-pane inspector accessible from the ActionsPage:

**Left pane -- Payload list:**
- Time-range filter (quick chips: 1h, 6h, 24h, 7d, plus custom range)
- Scrollable list of payloads showing: timestamp, content type, body size preview
- Click to select a payload for detail view

**Right pane -- Payload detail:**
- Syntax-highlighted JSON viewer for the selected payload body
- Copy-to-clipboard button for the raw payload
- "Replay" button that POSTs to the replay endpoint and shows the result (success/failure with run ID)
- Content-type and received-at metadata header

### 5. Add replay endpoint (backend)

Add `POST /api/webhooks/{route}/replay/{id}` in `crates/sdlc-server/src/routes/webhooks.rs`:

- Loads the stored payload by ID from the backend
- Verifies the payload belongs to the specified route
- Dispatches the raw body through the registered tool (same path as normal webhook dispatch, but sourced from stored data)
- Returns `{ ok: true, run_id: "<id>" }` on success, or an error

## Scope

| Task | Description |
|------|-------------|
| T1 | Add `store_only` and `secret_token` fields to `OrchestratorWebhookRoute` type in `frontend/src/lib/types.ts` |
| T2 | Add `queryWebhookPayloads` and `replayWebhookPayload` API client methods in `frontend/src/api/client.ts` |
| T3 | Update `WebhookRoutesSection` in ActionsPage -- show store-only chip, secret indicator, and Inspect button |
| T4 | Build `WebhookPayloadInspector` component -- two-pane layout with time filter, payload list, JSON viewer, copy/replay controls |
| T5 | Add `POST /api/webhooks/{route}/replay/{id}` endpoint in `routes/webhooks.rs` and register in `lib.rs` |

## Out of Scope

- Payload schema validation or transformation
- Payload TTL / automatic eviction
- Real-time streaming of incoming payloads (SSE for new arrivals)
- Editing or modifying stored payloads
- Bulk replay of multiple payloads

## Acceptance Criteria

- Store-only routes are visually distinguished in the webhook routes table with a badge
- Routes with a secret token show a lock indicator
- Clicking "Inspect" on a store-only route opens the payload inspector
- The payload inspector loads and displays stored payloads with time-range filtering
- Selecting a payload shows its full JSON body with syntax highlighting
- The copy button copies the raw payload JSON to clipboard
- The replay button dispatches the payload and shows the result (run ID or error)
- All existing webhook route functionality (create, delete, dispatch routes) is unchanged
