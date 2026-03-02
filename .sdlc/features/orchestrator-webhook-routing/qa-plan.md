# QA Plan: WebhookRoute Registration and Tick Dispatch

## Scope

This QA plan covers the `orchestrator-webhook-routing` feature. It validates the data layer, tick loop extension, and REST API surface for webhook route registration and dispatch.

## Test Approach

Primary: automated Rust unit tests and integration tests run via `cargo test`. No UI component — no browser or Playwright tests required.

Secondary: manual smoke test of the REST API using `curl` against a running `sdlc ui` instance to validate end-to-end flow.

---

## Unit Tests

### `orchestrator/webhook.rs`

| Test | Description | Expected |
|---|---|---|
| `render_input_substitutes_payload` | Call `render_input` with a template containing `{{payload}}` and a JSON payload | Returns `serde_json::Value` with `{{payload}}` replaced by the JSON-escaped payload string |
| `render_input_invalid_json_after_render` | Template produces invalid JSON after substitution | Returns `Err(SdlcError::OrchestratorDb(...))` |
| `render_input_no_placeholder` | Template with no `{{payload}}` placeholder | Parses the template as-is; ignores the raw_payload |
| `render_input_binary_payload` | Raw payload with non-UTF8 bytes | Replaced with lossy UTF-8 conversion; result is valid JSON string |

### `orchestrator/db.rs` — webhook route table

| Test | Description | Expected |
|---|---|---|
| `insert_and_find_route` | Insert a route, call `find_route_by_path` | Returns `Some(route)` with matching fields |
| `duplicate_path_returns_error` | Insert two routes with the same path | Second insert returns `Err` |
| `list_routes_sorted_by_created_at` | Insert routes in non-chronological order | `list_routes` returns them in `created_at` ascending order |
| `find_route_not_found` | Call `find_route_by_path` with a path not in the table | Returns `Ok(None)` |

### `orchestrator/db.rs` — `all_pending_webhooks` and `delete_action`

| Test | Description | Expected |
|---|---|---|
| `all_pending_webhooks_returns_only_webhook_pending` | DB contains scheduled + webhook actions, one webhook is Running | `all_pending_webhooks` returns only the Pending webhook action |
| `delete_action_removes_record` | Insert action, call `delete_action`, then `list_all` | Action no longer in `list_all` |
| `delete_action_nonexistent_is_noop` | Call `delete_action` with an action not in the DB | Returns `Ok(())` without error |

### `orchestrator/action.rs` — `new_webhook`

| Test | Description | Expected |
|---|---|---|
| `new_webhook_sets_correct_fields` | Call `new_webhook("path", payload)` | `label = "path"`, `tool_name = "_webhook"`, `tool_input = Null`, trigger is `Webhook`, status is `Pending` |

---

## Tick Loop Integration Tests

### `cmd/orchestrate.rs` (or `tests/orchestrator_webhook.rs`)

| Test | Description | Expected |
|---|---|---|
| `webhook_dispatch_calls_tool` | Set up temp dir with tool.ts that writes `{}` to stdout; insert route + webhook action; call `run_one_tick` | Tool is invoked, webhook action is deleted from DB |
| `webhook_dispatch_no_route_deletes_action` | Insert webhook action with no matching route; call `run_one_tick` | Action is deleted from DB; no tool called |
| `webhook_dispatch_missing_tool_deletes_action` | Route registered but tool.ts does not exist; call `run_one_tick` | Action is deleted from DB |
| `webhook_dispatch_template_error_deletes_action` | Route with invalid template (produces non-JSON after substitution); call `run_one_tick` | Action is deleted from DB |
| `scheduled_actions_still_run_after_webhook_phase` | Both scheduled and webhook actions due; call `run_one_tick` | Both dispatched in correct phases |

---

## REST API Tests (server integration tests if applicable)

The server does not currently have HTTP integration tests for orchestrator routes. These are validated via manual smoke test below. If server-level integration tests exist in future, add:

| Endpoint | Scenario | Expected |
|---|---|---|
| `POST /api/orchestrator/webhooks/routes` | Valid body | `201` + `WebhookRoute` JSON |
| `POST /api/orchestrator/webhooks/routes` | Duplicate path | `409 Conflict` |
| `POST /api/orchestrator/webhooks/routes` | Missing `path` | `400 Bad Request` |
| `POST /api/orchestrator/webhooks/routes` | Path without leading `/` | `400 Bad Request` |
| `POST /api/orchestrator/webhooks/routes` | Empty `tool_name` | `400 Bad Request` |
| `GET /api/orchestrator/webhooks/routes` | Routes registered | `200` + array |
| `GET /api/orchestrator/webhooks/routes` | No routes | `200` + empty array |
| `POST /api/orchestrator/webhooks/hooks/my-service` | Any body | `202 Accepted`; action inserted in DB |

---

## Manual Smoke Test

**Prerequisite:** `sdlc init` in a temp project, `sdlc ui` running on port 3141.

**Step 1:** Register a route.
```bash
curl -s -X POST http://localhost:3141/api/orchestrator/webhooks/routes \
  -H "Content-Type: application/json" \
  -d '{"path":"/hooks/test","tool_name":"quality-check","input_template":"{\"payload\":{{payload}}}"}'
# Expect: 201 Created with WebhookRoute JSON including id and created_at
```

**Step 2:** List routes.
```bash
curl -s http://localhost:3141/api/orchestrator/webhooks/routes
# Expect: JSON array with one entry matching the registered route
```

**Step 3:** Post a webhook payload.
```bash
curl -s -X POST http://localhost:3141/api/orchestrator/webhooks/hooks/test \
  -H "Content-Type: application/json" \
  -d '{"event":"test"}'
# Expect: 202 Accepted
```

**Step 4:** Verify the webhook action was inserted.
```bash
sdlc orchestrate list --status pending
# Expect: one action with label "/hooks/test" in Pending state
```

**Step 5:** Start the daemon for one tick.
```bash
sdlc orchestrate --tick-rate 1
# Expect: log line "orchestrate: [/hooks/test] webhook completed" (or failed if quality-check not installed)
```

**Step 6:** Verify the action is removed.
```bash
sdlc orchestrate list
# Expect: no Pending webhook action for /hooks/test
```

---

## Clippy and Test Execution

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Both must pass with zero warnings and zero test failures.

---

## Pass Criteria

- All unit tests pass.
- All tick loop integration tests pass.
- Manual smoke test steps 1–6 produce expected outputs.
- `cargo clippy --all -- -D warnings` exits clean.
- No `unwrap()` calls in library code (enforced by code review).
