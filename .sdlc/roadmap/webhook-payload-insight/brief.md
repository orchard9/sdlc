## Brief

Jordan wants a UI surface for tool webhooks — specifically for the store_only pattern being built as part of the webhook query infrastructure.

Context: The new webhook infrastructure adds store_only routes where Telegram (and other external services) push payloads to our server. These are NOT dispatched — they're stored for tools to query on their own schedule. The UI needs to make this data accessible.

Two core user needs:
1. **Insight**: What came in? Browse stored payloads per route — body inspection, time filtering, pagination
2. **Replay**: Re-trigger a payload — send a stored payload back through the tool dispatch pipeline for debugging

This likely lives in the existing ActionsPage webhook section, as a drill-down from a route row. The existing ActionsPage shows routes and events but has no access to the actual stored payload bodies.

New backend capabilities being added:
- GET /api/webhooks/{route}/data?since=...&until=...&limit=... — returns [{id, received_at, content_type, body}]
- store_only field on routes
- secret_token on routes

The UI should feel like a lightweight webhook inspector — timeline on the left, JSON body on the right, with replay controls.