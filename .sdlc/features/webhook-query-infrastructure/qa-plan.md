# QA Plan: Webhook Query Infrastructure

## Build Verification

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Both must exit 0 with no errors or warnings.

## Unit Tests (db.rs)

| Test | Location | Verifies |
|------|----------|---------|
| `query_webhooks_filters_by_route_and_time` | `crates/sdlc-core/src/orchestrator/db.rs` | Filters payloads by route + time range |
| `query_webhooks_respects_limit` | `crates/sdlc-core/src/orchestrator/db.rs` | Limit param truncates results |
| `route_with_store_only_and_secret_round_trips` | `crates/sdlc-core/src/orchestrator/db.rs` | New fields persist and deserialize correctly |

## Integration Tests (integration.rs)

| Scenario | Expected |
|----------|----------|
| POST to store_only route → no action dispatched | 200, action count unchanged |
| POST with correct `X-Webhook-Secret` | 200, payload stored |
| POST with wrong `X-Webhook-Secret` | 401, payload not stored |
| POST with no secret header when secret_token set | 401 |
| GET `/api/webhooks/{route}/data` no filters | 200, all payloads for route |
| GET with `?since=` filter | Only payloads after timestamp |
| GET with `?limit=2` | At most 2 results |
| GET unknown route | 404 |
| Register route with secret_token and store_only via POST `/api/orchestrator/routes` | 200, fields persisted |

## Regression Check

- Routes without `secret_token` or `store_only` continue to work exactly as before
- Existing serialized `WebhookRoute` records (without new fields) deserialize cleanly via serde defaults
- No `unwrap()` calls introduced in new code
