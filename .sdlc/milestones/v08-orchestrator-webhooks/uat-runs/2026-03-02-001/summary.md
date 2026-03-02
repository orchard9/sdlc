# UAT Run — Webhook ingestion and routing — external triggers fire tools on next tick
**Date:** 2026-03-02T07:10:00Z
**Verdict:** Failed
**Tests:** 6/13
**Tasks created:** orchestrator-webhook-storage#T7

## Results
Suite: v08-orchestrator-webhooks.spec.ts (Mode B — spec generated from checklist)
Passed: 6 | Failed: 7 | Skipped: 0

## Failures
| Test | Classification | Resolution |
|---|---|---|
| POST /api/orchestrator/webhooks/routes returns 201 | Code bug | Task orchestrator-webhook-storage#T7 created |
| registered route appears in GET list | Code bug | Task orchestrator-webhook-storage#T7 created |
| POST /webhooks/<registered-path> returns 202 | Code bug | Task orchestrator-webhook-storage#T7 created |
| POST /webhooks/unknown-route returns 202 without crash | Code bug | Task orchestrator-webhook-storage#T7 created |
| registered route with {{payload}} template stores 201 | Code bug | Task orchestrator-webhook-storage#T7 created |
| POST /webhooks/<template-route> returns 202 | Code bug | Task orchestrator-webhook-storage#T7 created |
| GET /api/orchestrator/webhooks/routes returns JSON array | Code bug | Task orchestrator-webhook-storage#T7 created |

## Root cause
All 7 failures share a single root cause: the orchestrator daemon thread (spawned in `ui.rs`) calls `ActionDb::open()` at startup and holds the redb file lock open for the server lifetime. Every HTTP handler that calls `ActionDb::open()` via `spawn_blocking` gets "Database already open. Cannot acquire lock." (500). Fix: share `ActionDb` through `AppState` rather than opening per-request.
