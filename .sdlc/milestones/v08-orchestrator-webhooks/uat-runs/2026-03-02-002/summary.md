# UAT Run — Webhook ingestion and routing
**Date:** 2026-03-02T07:40:40Z
**Run ID:** 2026-03-02-002
**Verdict:** Failed
**Tests:** 12/13
**Tasks created:** none (failure tracked under orchestrator-webhook-storage#T7)

## Environment

- Server: `sdlc ui start --port 7777 --no-open --no-orchestrate`
- Playwright config updated: `--no-orchestrate` added to webServer command
- Workers: 2 (fullyParallel: true)

## Results

Suite: `milestones/v08-orchestrator-webhooks.spec.ts`
Duration: 1292ms
Passed: 12 | Failed: 1 | Skipped: 0

## Failures

| Test | Classification | Resolution |
|---|---|---|
| POST /api/orchestrator/webhooks/routes returns 201 with route object | Code bug | T7 — per-request `ActionDb::open()` races under concurrent Playwright workers |

## Root Cause

The `register_route` handler calls `ActionDb::open()` per-request inside `spawn_blocking`.
With `workers: 2` in Playwright, two tests that both POST to `/api/orchestrator/webhooks/routes`
run concurrently on different workers. Both call `ActionDb::open()` at the same moment.
redb enforces a single exclusive writer — one request gets the lock, the other gets:
`"Database already open. Cannot acquire lock."` → 500 response.

This is the same root cause as the run-2 failure (T7): per-request open/close works only
under serial load. The fix (share `Arc<Mutex<ActionDb>>` via `AppState`) is tracked as T7.

## API Verification (manual, serial)

The following manual curl calls all succeeded (201/202/200) on the clean server, confirming
the implementation is correct under serial load:

```
POST /api/orchestrator/webhooks/routes  → 201 ✓ (two separate calls)
GET  /api/orchestrator/webhooks/routes  → 200 ✓
```

## Tests Passing (12/13)

- registered route appears in GET /api/orchestrator/webhooks/routes
- POST /webhooks/<registered-path> returns 202 with payload id
- POST /webhooks/unknown-route returns 202 without crash
- registered route with {{payload}} template stores correctly (201)
- POST /webhooks/<template-route> returns 202 with payload id (mapping stored)
- orchestrator/webhook.rs defines WebhookPayload with id, route_path, raw_body
- orchestrator/webhook.rs defines WebhookRoute with render_input method
- orchestrator/db.rs defines WEBHOOKS and WEBHOOK_ROUTES redb tables
- sdlc-server lib.rs wires POST /webhooks/{route} and webhook route management
- POST /api/orchestrator/webhooks/routes with missing path returns 400
- POST /api/orchestrator/webhooks/routes with empty tool_name returns 400
- GET /api/orchestrator/webhooks/routes returns JSON array

## Next

Implement T7: pass `Arc<Mutex<ActionDb>>` through `AppState` so all handlers share
one open connection. Then re-run `/sdlc-milestone-uat v08-orchestrator-webhooks`.
