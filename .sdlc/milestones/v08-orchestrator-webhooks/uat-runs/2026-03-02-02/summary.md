# UAT Run — Webhook ingestion and routing — external triggers fire tools on next tick
**Date:** 2026-03-02T07:12:02Z
**Verdict:** Failed
**Tests:** 6/13
**Tasks created:** none (all failures covered by orchestrator-webhook-storage#T7)

## Results
Suite: milestones/v08-orchestrator-webhooks.spec.ts
Duration: 2403ms
Passed: 6 | Failed: 7 | Skipped: 0

## Failures

| Test | Classification | Resolution |
|---|---|---|
| POST /api/orchestrator/webhooks/routes returns 201 | Code bug | T7 — daemon holds ActionDb lock; handler gets 500 |
| registered route appears in GET list | Code bug | T7 — same DB lock |
| POST /webhooks/<registered-path> returns 202 | Code bug | T7 — same DB lock |
| POST /webhooks/unknown-route returns 202 | Code bug | T7 — same DB lock |
| registered route with {{payload}} template stores correctly (201) | Code bug | T7 — same DB lock |
| POST /webhooks/<template-route> returns 202 (mapping stored) | Code bug | T7 — same DB lock |
| GET /api/orchestrator/webhooks/routes returns JSON array | Code bug | T7 — same DB lock |

## Root Cause

`sdlc ui start` spawns an orchestrator daemon thread that calls `ActionDb::open(path)` and holds
the redb exclusive file lock for the server's lifetime. The Playwright `webServer` config calls
`sdlc ui start --port 7777 --no-open` which starts this daemon. HTTP handlers in `webhooks.rs`
and `orchestrator.rs` each call `ActionDb::open(path)` per-request — but redb does not allow
a second exclusive open while the daemon holds the lock, returning "Database already open."

Fix: share a single `Arc<Mutex<ActionDb>>` via `AppState` (tracked in T7).
