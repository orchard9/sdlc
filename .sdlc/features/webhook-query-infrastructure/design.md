# Design: Webhook Query Infrastructure

## Overview

Backend-only feature. No UI components. Three orthogonal capabilities added to the existing webhook subsystem.

## Data Model Changes

### `WebhookRoute` struct (`orchestrator/webhook.rs`)

```
WebhookRoute {
    route:        String,        // existing
    action:       String,        // existing
    secret_token: Option<String>, // NEW — serde default = None
    store_only:   bool,          // NEW — serde default = false
}
```

Serde `#[serde(default)]` ensures existing persisted routes without these fields deserialize cleanly (backward compatible).

### Storage (redb)

Existing `WEBHOOKS_TABLE: TableDefinition<&str, &[u8]>` — `WebhookRoute` serialized as JSON. New fields appear in JSON on next write; existing rows without them deserialize via serde defaults on read. No migration needed for redb.

### Storage (postgres)

Migration `004_webhook_query.sql`:
```sql
ALTER TABLE webhook_routes ADD COLUMN IF NOT EXISTS secret_token TEXT;
ALTER TABLE webhook_routes ADD COLUMN IF NOT EXISTS store_only   BOOLEAN NOT NULL DEFAULT false;
CREATE INDEX IF NOT EXISTS idx_webhook_data_route_received_at
    ON webhook_data(route, received_at DESC);
```

The composite index on `(route, received_at DESC)` makes time-range queries on `webhook_data` efficient.

## Component Interactions

```
POST /webhooks/{route}
    ├── look up WebhookRoute
    ├── if secret_token set → check X-Webhook-Secret header → 401 on mismatch
    ├── store payload in webhook_data (always)
    └── if NOT store_only → dispatch to orchestrator action

GET /api/webhooks/{route}/data?since=&until=&limit=
    ├── look up WebhookRoute (404 if unknown)
    └── query_webhooks(route, since, until, limit) → Vec<WebhookEntry>
```

## `query_webhooks` Trait Method

```rust
fn query_webhooks(
    &self,
    route:  &str,
    since:  Option<DateTime<Utc>>,
    until:  Option<DateTime<Utc>>,
    limit:  Option<usize>,
) -> Result<Vec<WebhookEntry>>;
```

`WebhookEntry` carries: `route`, `received_at`, `payload` (raw bytes or JSON value).

### Redb Implementation

Full table scan of `webhook_data` table, filter by route name and timestamp bounds in Rust, then sort descending and truncate to limit. Acceptable for local use where payload volume is bounded.

### Postgres Implementation

```sql
SELECT route, received_at, payload
FROM webhook_data
WHERE route = $1
  AND ($2::timestamptz IS NULL OR received_at >= $2)
  AND ($3::timestamptz IS NULL OR received_at <= $3)
ORDER BY received_at DESC
LIMIT $4
```

Composite index on `(route, received_at DESC)` makes this an index range scan.

## Secret Verification Flow

```rust
if let Some(expected) = &route.secret_token {
    let provided = req.headers()
        .get("X-Webhook-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if provided != expected {
        return Err(StatusCode::UNAUTHORIZED);
    }
}
```

Constant-time comparison is not required here (secret is not a cryptographic HMAC; it is a shared bearer token). A future hardening task could switch to `subtle::ConstantTimeEq`.

## `store_only` Dispatch Skip

```rust
if !route.store_only {
    dispatch_webhook(&route.action, &payload, &app).await?;
}
```

The payload is always stored before this check, ensuring no data loss even when dispatch is skipped.

## REST Layer Changes

`RegisterRouteBody` gains two optional fields:
```rust
struct RegisterRouteBody {
    route:        String,
    action:       String,
    secret_token: Option<String>,
    store_only:   Option<bool>,   // mapped to bool with unwrap_or(false)
}
```

## Error Handling

| Scenario | HTTP status |
|----------|-------------|
| Unknown route (query endpoint) | 404 |
| Missing/wrong secret | 401 |
| Backend I/O error | 500 |
| Invalid `since`/`until` format | 400 |

## Testing Strategy

- Unit tests in `db.rs`: `query_webhooks_filters_by_route_and_time`, `query_webhooks_respects_limit`, `route_with_store_only_and_secret_round_trips`
- Integration tests in `sdlc-server/tests/integration.rs` cover the REST endpoint and secret rejection path
- Postgres impl tested against live DB in CI via `DATABASE_URL`
