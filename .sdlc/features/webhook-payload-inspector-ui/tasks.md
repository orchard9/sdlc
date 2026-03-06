# Tasks: Webhook Payload Inspector UI

## T1: Add store_only and secret_token fields to OrchestratorWebhookRoute type

**File:** `frontend/src/lib/types.ts`

Add `store_only: boolean` and `secret_token?: string | null` to the `OrchestratorWebhookRoute` interface. Also add a new `WebhookPayloadItem` interface for query results.

## T2: Add queryWebhookPayloads and replayWebhookPayload API client methods

**File:** `frontend/src/api/client.ts`

- `queryWebhookPayloads(route, params)` -> `GET /api/webhooks/{route}/data`
- `replayWebhookPayload(route, id)` -> `POST /api/webhooks/{route}/replay/{id}`

## T3: Update WebhookRoutesSection in ActionsPage

**File:** `frontend/src/pages/ActionsPage.tsx`

- Show "Store-only" chip on routes where `store_only === true`
- Show lock icon on routes with `secret_token` present
- Add "Inspect" button on store-only routes that toggles the inspector panel
- Manage `inspectingRoute` state to control inspector visibility

## T4: Build WebhookPayloadInspector component

**File:** `frontend/src/components/webhooks/WebhookPayloadInspector.tsx`

Two-pane layout:
- Left: time range bar (1h/6h/24h/7d chips) + scrollable payload list
- Right: JSON viewer + copy button + replay button + replay result strip
- Handles loading, empty, and error states

## T5: Add POST /api/webhooks/{route}/replay/{id} endpoint

**Files:** `crates/sdlc-server/src/routes/webhooks.rs`, `crates/sdlc-server/src/lib.rs`

- Load stored payload by ID
- Validate it belongs to the specified route
- Dispatch through the registered tool
- Return `{ ok, run_id }` or error
