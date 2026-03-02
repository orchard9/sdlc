# QA Plan: HTTP Webhook Receiver and Raw Payload Storage in redb

## Automated Tests

All tests run via `SDLC_NO_NPM=1 cargo test --all`.

### Unit Tests (`sdlc-core`)

| Test | Expected |
|---|---|
| `webhook_insert_and_retrieve_round_trip` | Inserted payload returned by `all_pending_webhooks` with exact `id`, `route_path`, `raw_body`, `content_type` |
| `webhook_delete_removes_record` | After `delete_webhook(id)`, `all_pending_webhooks` returns empty |
| `webhook_multiple_payloads_sorted_by_received_at` | Two payloads returned in ascending `received_at` order |
| `empty_db_all_pending_webhooks_returns_empty` | Fresh `ActionDb` returns empty vec from `all_pending_webhooks` |
| `existing_db_open_adds_webhooks_table` | Re-opening an existing DB without WEBHOOKS table succeeds and `all_pending_webhooks` works |

### Integration Tests (`sdlc-server`)

| Test | Expected |
|---|---|
| `post_webhook_returns_202_with_id` | `POST /webhooks/test` returns 202, JSON body with a valid `id` UUID string |

## Linting

- `cargo clippy --all -- -D warnings` passes with zero warnings

## Manual Smoke Test

After `cargo build`:

```bash
# Start the server
cargo run -p sdlc-cli -- ui --no-open

# POST a webhook
curl -s -X POST http://localhost:3141/webhooks/github \
  -H "Content-Type: application/json" \
  -d '{"event": "push"}' | jq .

# Expected: { "id": "<some-uuid>" }
```

## Pass Criteria

- All automated unit and integration tests pass
- `cargo clippy --all -- -D warnings` exits 0
- Manual smoke test returns `202` with a JSON `{ "id": "..." }` response
- Raw body bytes are preserved exactly as sent (no encoding transformation)
