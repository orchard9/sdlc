# Tasks: Webhook Query Infrastructure

## T1 — Add secret_token and store_only to WebhookRoute
Add `secret_token: Option<String>` and `store_only: bool` (serde default = false) to `WebhookRoute` in `webhook.rs` with builder methods `with_secret_token()` and `with_store_only()`.

## T2 — Add query_webhooks to OrchestratorBackend trait
Add `query_webhooks(route, since, until, limit)` to `OrchestratorBackend` trait in `backend.rs` returning `Vec<WebhookPayload>` ordered by `received_at`.

## T3 — Implement query_webhooks for redb
Implement `query_webhooks` for redb in `db.rs` — scan WEBHOOKS table, filter by route + time range, sort, truncate to limit.

## T4 — Implement query_webhooks for postgres
Implement `query_webhooks` for postgres in `pg_orchestrator.rs` using `sqlx::query()` runtime API.

## T5 — Migration 004_webhook_query.sql
Write `crates/sdlc-server/migrations/004_webhook_query.sql` — add `secret_token TEXT` and `store_only BOOLEAN DEFAULT false` columns; create composite index on `(route_path, received_at)`.

## T6 — Skip dispatch for store_only routes
Update `dispatch_webhook` in `orchestrate.rs` — skip dispatch and return `Ok(())` early when `route.store_only` is true.

## T7 — Secret verification in receive_webhook
Add secret verification to `receive_webhook` in `routes/webhooks.rs` — check `X-Webhook-Secret` header; reject with 401 on mismatch when `secret_token` is set.

## T8 — GET /api/webhooks/{route}/data endpoint
Add `GET /api/webhooks/{route}/data` with `WebhookQueryParams` (since, until, limit) in `routes/webhooks.rs`; register in `lib.rs`.

## T9 — RegisterRouteBody secret_token + store_only
Update `RegisterRouteBody` in `routes/orchestrator.rs` to accept `secret_token` and `store_only`; redact secret in list responses.
