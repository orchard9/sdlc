# UAT Run — Webhook ingestion and routing — external triggers fire tools on next tick
**Date:** 2026-03-02T07:12:02Z
**Agent:** claude-sonnet-4-6
**Verdict:** FAILED

---

## Scenario 1: Register a webhook route
- [ ] ~~POST /api/orchestrator/webhooks/routes returns 201 with route object~~ _(✗ task orchestrator-webhook-storage#T7 — sdlc ui start daemon holds redb lock; handler gets 500)_
- [ ] ~~registered route appears in GET /api/orchestrator/webhooks/routes~~ _(✗ task orchestrator-webhook-storage#T7 — same DB lock)_

## Scenario 2: Fire a webhook and verify processing
- [ ] ~~POST /webhooks/<registered-path> returns 202 with payload id~~ _(✗ task orchestrator-webhook-storage#T7 — daemon holds redb lock; handler gets 500)_

## Scenario 3: Unregistered webhook route
- [ ] ~~POST /webhooks/unknown-route returns 202 without crash~~ _(✗ task orchestrator-webhook-storage#T7 — daemon holds redb lock; handler gets 500)_

## Scenario 4: Payload mapping
- [ ] ~~registered route with {{payload}} template stores correctly (201)~~ _(✗ task orchestrator-webhook-storage#T7 — same DB lock)_
- [ ] ~~POST /webhooks/<template-route> returns 202 with payload id (mapping stored)~~ _(✗ task orchestrator-webhook-storage#T7 — same DB lock)_

## GET /api/orchestrator/webhooks/routes
- [ ] ~~GET /api/orchestrator/webhooks/routes returns JSON array~~ _(✗ task orchestrator-webhook-storage#T7 — daemon holds redb lock; handler gets 500)_

## Source verification (passed)
- [x] orchestrator/webhook.rs defines WebhookPayload with id, route_path, raw_body _(2026-03-02)_
- [x] orchestrator/webhook.rs defines WebhookRoute with render_input method _(2026-03-02)_
- [x] orchestrator/db.rs defines WEBHOOKS and WEBHOOK_ROUTES redb tables _(2026-03-02)_
- [x] sdlc-server lib.rs wires POST /webhooks/{route} and webhook route management _(2026-03-02)_

## API validation (passed)
- [x] POST /api/orchestrator/webhooks/routes with missing path returns 400 _(2026-03-02)_
- [x] POST /api/orchestrator/webhooks/routes with empty tool_name returns 400 _(2026-03-02)_

---

**Tasks created:** none (all failures tracked under orchestrator-webhook-storage#T7)
**6/13 steps passed**

## Root Cause (run 2)

`sdlc ui start` spawns an orchestrator daemon thread (`sdlc-orchestrator`) that holds
`ActionDb::open()` open for the server lifetime. HTTP handlers in `webhooks.rs` and
`orchestrator.rs` each call `ActionDb::open()` per-request. redb enforces a single exclusive
writer, so every handler open attempt fails with "Database already open. Cannot acquire lock."

The fix (T7): share a single `Arc<Mutex<ActionDb>>` via `AppState` so handlers reuse the
daemon's connection rather than fighting for the file lock.
