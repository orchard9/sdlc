# Plan: Webhook Payload Inspector

## Source ponder
`webhook-payload-insight` — Webhook Payload Insight & Replay UI

## Context
The existing webhook infrastructure (v08) is write-only: payloads arrive via POST /webhooks/{route} and are dispatched on the next tick. There's no way to query stored payloads, no secret verification, and the tick loop deletes payloads after dispatch. The telegram-recap rewrite (v39-slack-bot-tool work) requires a store_only mode where Telegram pushes messages and the tool queries them on its own schedule. The UI needs a matching inspector so developers can browse what arrived and replay it for debugging.

## Scope assessment
**Medium idea** — two tightly coupled layers (backend infrastructure + UI). One milestone, two features. The backend must land before the UI can be wired.

---

## Milestone: v42-webhook-payload-inspector

**Vision:** A developer working with store_only webhook routes can browse every stored payload for any time window, inspect its JSON body, and replay it through the registered tool — all from the Actions page. Debugging a misconfigured telegram-recap or any other webhook-driven tool takes seconds, not a curl session.

**Acceptance criteria:** After configuring a store_only webhook route and receiving payloads, a developer opens Actions → clicks Inspect on the route → selects a time window → sees the payload list → clicks a payload → reads the JSON body → clicks Replay → gets a run ID confirming the tool was dispatched with that payload's content.

---

### Feature 1: webhook-query-infrastructure

**Title:** Webhook Query Infrastructure — store_only routes, secret verification, and payload query API

**Summary:** Extend the orchestrator backend to support store_only routes (payloads retained, not dispatched) and secret_token verification on ingress. Add query_webhooks to OrchestratorBackend trait with redb and postgres implementations. Add GET /api/webhooks/{route}/data endpoint. Update route registration to accept new fields.

**Tasks:**
- Add `secret_token: Option<String>` and `store_only: bool` to WebhookRoute in webhook.rs with builder methods
- Add `query_webhooks(path, since, until, limit)` to OrchestratorBackend trait in backend.rs
- Implement query_webhooks for redb in db.rs — scan WEBHOOKS table, filter by path + time range, sort ASC, truncate to limit
- Implement query_webhooks for postgres in pg_orchestrator.rs — SELECT with WHERE route_path=$1 AND received_at BETWEEN $2 AND $3 ORDER BY received_at ASC LIMIT $4
- Write migration 004_webhook_query.sql — ADD COLUMN secret_token, store_only; CREATE INDEX on (route_path, received_at)
- Update dispatch_webhook in orchestrate.rs — skip dispatch and return early if route.store_only
- Add secret verification to receive_webhook in routes/webhooks.rs — check X-Telegram-Bot-Api-Secret-Token or X-Webhook-Secret headers against route.secret_token; reject 403 on mismatch
- Add GET /api/webhooks/{route}/data with WebhookQueryParams (since, until, limit) to routes/webhooks.rs — returns [{id, received_at, content_type, body}]; JSON body parsed when content_type contains "json", else base64
- Register /api/webhooks/{route}/data in lib.rs
- Update RegisterRouteBody in routes/orchestrator.rs to accept secret_token and store_only; redact secret_token in list responses

### Feature 2: webhook-payload-inspector-ui

**Title:** Webhook Payload Inspector UI — browse, inspect, and replay stored payloads

**Summary:** Add an inline two-pane inspector panel to the Actions page for store_only webhook routes. The panel shows a time-range-filtered payload list on the left and a JSON viewer with replay controls on the right. Also adds the server-side replay endpoint that re-dispatches a stored payload through the route's registered tool.

**Tasks:**
- Add store_only and secret_token fields to OrchestratorWebhookRoute type in frontend/src/lib/types.ts
- Add api.queryWebhookPayloads(route, since, until, limit) method to frontend/src/api/client.ts
- Add api.replayWebhookPayload(route, id) method to frontend/src/api/client.ts
- Update WebhookRoutesSection in ActionsPage — show "store-only" chip and "Inspect" button on routes where store_only=true; dispatch routes unchanged
- Build WebhookPayloadInspector component — two-pane: list (time, content-type, size, body preview) left; JSON viewer right; time window chips (1h/6h/24h/7d/custom); copy and replay controls in detail header; replay result footer strip
- Add POST /api/webhooks/{route}/replay/{id} endpoint in routes/webhooks.rs — loads stored payload by id, dispatches raw_body to the tool registered on the route via the existing tool dispatch path; returns {ok, run_id}
- Register replay endpoint in lib.rs
- Wire replay button in WebhookPayloadInspector to api.replayWebhookPayload; show run ID in footer strip on success
