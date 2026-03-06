# Spec: Webhook Query Infrastructure

## Problem

Webhook ingestion in SDLC was write-only: payloads were dispatched to orchestrator actions but never stored for later retrieval. This created two gaps:

1. **No queryability** — there was no way to inspect what payloads arrived on a given route, when they arrived, or filter by time window.
2. **No store-only routes** — every registered webhook route triggered action dispatch. There was no way to register a route purely for payload capture without side-effects.
3. **No secret verification** — registered webhook routes had no authentication; any caller could POST to a route.

## Solution

Extend the webhook infrastructure with three complementary capabilities:

### 1. `store_only` flag on `WebhookRoute`
A boolean field (`store_only`, default `false`) that, when `true`, causes the webhook receiver to store the payload in the database but skip action dispatch entirely. Useful for capture-only integrations (e.g., Slack event subscriptions, GitHub audit logs) where the data will be queried rather than acted upon in real time.

### 2. `secret_token` verification
An optional `secret_token` field on `WebhookRoute`. When set, the receiver checks the `X-Webhook-Secret` header on every incoming POST. Requests with a missing or incorrect secret are rejected with HTTP 401. This gates route access without requiring full auth middleware.

### 3. `query_webhooks` API — payload retrieval
A new `query_webhooks` method on `OrchestratorBackend` and a `GET /api/webhooks/{route}/data` endpoint that returns stored payloads with filtering by:
- `route` — the webhook route slug (required, path param)
- `since` — ISO 8601 timestamp lower bound (optional query param)
- `until` — ISO 8601 timestamp upper bound (optional query param)
- `limit` — maximum number of results (optional, default 100)

Results are returned in descending chronological order (newest first).

## Scope

| Task | Description |
|------|-------------|
| T1 | Add `secret_token` and `store_only` fields to `WebhookRoute` with serde defaults |
| T2 | Add `query_webhooks` to `OrchestratorBackend` trait |
| T3 | Implement `query_webhooks` in redb backend (full scan + filter) |
| T4 | Implement `query_webhooks` in postgres backend (index-assisted query) |
| T5 | Write migration `004_webhook_query.sql` — add columns + composite index |
| T6 | Skip action dispatch in `dispatch_webhook` when `store_only` is true |
| T7 | Verify `X-Webhook-Secret` header in `receive_webhook` when `secret_token` is set |
| T8 | Implement `GET /api/webhooks/{route}/data` endpoint |
| T9 | Add `secret_token` and `store_only` to `RegisterRouteBody` in REST layer |

## Out of Scope

- UI for browsing webhook payloads (deferred to a future UI milestone)
- Payload schema validation
- Webhook replay/retry
- Payload TTL / eviction

## Acceptance Criteria

- `store_only` routes store payloads but do not trigger action dispatch
- Routes with `secret_token` set reject requests with wrong or missing `X-Webhook-Secret`
- `GET /api/webhooks/{route}/data` returns payloads with correct filtering and ordering
- All existing webhook behavior is unchanged for routes without the new fields
- Tests pass: `query_webhooks_filters_by_route_and_time`, `query_webhooks_respects_limit`, `route_with_store_only_and_secret_round_trips`
