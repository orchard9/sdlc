# QA Plan: Webhook Payload Inspector UI

## Build Verification

- [ ] `SDLC_NO_NPM=1 cargo test --all` passes
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cd frontend && npm run build` succeeds without errors
- [ ] `cd frontend && npx tsc --noEmit` type-checks clean

## Type Correctness (T1)

- [ ] `OrchestratorWebhookRoute` includes `store_only: boolean` and `secret_token?: string | null`
- [ ] `WebhookPayloadItem` interface exists with `id`, `received_at`, `content_type`, `body` fields
- [ ] Existing usages of `OrchestratorWebhookRoute` still compile (non-breaking addition)

## API Client Methods (T2)

- [ ] `api.queryWebhookPayloads` calls correct endpoint with route parameter
- [ ] Query params (since, until, limit) are properly URL-encoded
- [ ] `api.replayWebhookPayload` calls POST to correct endpoint
- [ ] Both methods handle error responses gracefully

## WebhookRoutesSection Updates (T3)

- [ ] Store-only routes show a "Store-only" badge/chip
- [ ] Routes with secret_token show a lock indicator
- [ ] Dispatch routes (store_only=false) do NOT show Inspect button
- [ ] Store-only routes show an "Inspect" button
- [ ] Clicking Inspect renders the WebhookPayloadInspector below the table
- [ ] Clicking Inspect again (or a close button) hides the inspector

## WebhookPayloadInspector Component (T4)

- [ ] Time range chips (1h, 6h, 24h, 7d) filter payload results correctly
- [ ] Default time range is 24h
- [ ] Payload list shows timestamp, content type, and size for each entry
- [ ] Clicking a payload in the list selects it and shows detail in right pane
- [ ] JSON payloads are syntax-highlighted in the detail view
- [ ] Non-JSON payloads display as plain text
- [ ] Copy button copies the raw payload JSON to clipboard
- [ ] Empty state shows "No payloads found" message
- [ ] Loading state shows a spinner or skeleton
- [ ] Error state shows an inline error message

## Replay Functionality (T4 + T5)

- [ ] Replay button is present in the payload detail view
- [ ] Clicking Replay sends POST to `/api/webhooks/{route}/replay/{id}`
- [ ] Success shows run_id in the result strip
- [ ] Failure shows error message in the result strip
- [ ] Replay button shows loading state while request is in flight
- [ ] Replay button is disabled during replay request

## Backend Replay Endpoint (T5)

- [ ] `POST /api/webhooks/{route}/replay/{id}` is registered in the router
- [ ] Returns 404 if payload ID not found
- [ ] Returns 400 if payload route doesn't match the URL route parameter
- [ ] Returns 404 if no registered route exists for the path
- [ ] Returns 200 with `{ ok: true, run_id: "..." }` on successful dispatch
- [ ] Returns 500 with error details on dispatch failure

## Regression

- [ ] Existing webhook route creation still works
- [ ] Existing webhook route deletion still works
- [ ] Existing webhook event history still displays
- [ ] Dispatch routes continue to function normally (no store_only interference)
