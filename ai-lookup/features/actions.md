# Feature: Actions (Orchestrator)

**Traced:** 2026-03-02
**Confidence:** High

---

## Summary

The Actions feature is the orchestrator control plane: a UI page (`ActionsPage`) that manages **scheduled actions** (run a tool at a time, optionally recurring), **webhook routes** (map an incoming HTTP path to a tool), and **webhook event history** (audit log of received payloads). The backend is a pluggable-storage daemon: redb by default (zero external dependencies), postgres when `DATABASE_URL` is set. (`sdlc orchestrate`) with a REST API and SSE push.

---

## Entry Points

| Type | Location | Purpose |
|------|----------|---------|
| UI | `frontend/src/pages/ActionsPage.tsx:747` | Main page: scheduled actions, webhook routes, event history |
| CLI | `crates/sdlc-cli/src/cmd/orchestrate.rs:1` | `sdlc orchestrate` daemon + `sdlc orchestrate add/list` |
| API | `crates/sdlc-server/src/routes/orchestrator.rs:192` | `GET /api/orchestrator/actions` |
| API | `crates/sdlc-server/src/routes/orchestrator.rs:232` | `POST /api/orchestrator/actions` |
| API | `crates/sdlc-server/src/routes/orchestrator.rs:288` | `DELETE /api/orchestrator/actions/{id}` |
| API | `crates/sdlc-server/src/routes/orchestrator.rs:351` | `PATCH /api/orchestrator/actions/{id}` |
| API | `crates/sdlc-server/src/routes/orchestrator.rs:40` | `POST /api/orchestrator/webhooks/routes` |
| API | `crates/sdlc-server/src/routes/orchestrator.rs:113` | `GET /api/orchestrator/webhooks/routes` |
| API | `crates/sdlc-server/src/routes/orchestrator.rs` | `DELETE /api/orchestrator/webhooks/routes/{id}` |
| API | `crates/sdlc-server/src/routes/orchestrator.rs` | `GET /api/orchestrator/webhooks/events?limit=N` |
| API | `crates/sdlc-server/src/routes/webhooks.rs:37` | `POST /webhooks/{route}` — external webhook ingestion |
| Data | `crates/sdlc-core/src/orchestrator/db.rs:1` | `ActionDb` — redb storage layer (default; postgres when `DATABASE_URL` set) |
| SSE | `crates/sdlc-server/src/state.rs:406` | Sentinel file watcher → `ActionStateChanged` |

---

## Execution Flow

### Path 1: Create Scheduled Action

```
ScheduleActionModal → api.createAction({ next_tick_at, ... })
  → POST /api/orchestrator/actions
    → orchestrator::create_action
      → Action::new_scheduled(...)
      → ActionDb::insert → redb ACTIONS table
  → 201 Created { id, status: {type:"pending"}, trigger: {type:"scheduled", next_tick_at}, ... }
```

### Path 2: Orchestrator Daemon Tick

```
sdlc orchestrate (daemon, default 60s tick)
  → tick(root, db_path)
    → Phase 1: ActionDb::range_due(now) [composite key: timestamp_ms_be || uuid]
      → for due Pending actions: dispatch()
        → db.set_status(Running)
        → run .sdlc/tools/<name>/tool.ts with tool_input via stdin
        → db.set_status(Completed|Failed)
        → if recurrence: db.insert(next scheduled Action)
    → Phase 2: db.all_pending_webhooks()
      → match payload against registered routes (WebhookRoute.path)
      → WebhookRoute::render_input ({{payload}} template substitution)
      → dispatch tool
      → db.delete_webhook(payload.id)
    → write_tick_sentinel(.sdlc/.orchestrator.state)
      → state watcher detects mtime change → tx.send(SseMessage::ActionStateChanged)
        → SSE emits event("action") { type: "action_state_changed" }
        → ActionsPage.onActionEvent → refetchActions()
```

### Path 3: Webhook Ingestion

```
External service POST /webhooks/github
  → receive_webhook(route="github")
    → normalize route_path = "/github"
    → WebhookPayload::new(route_path, raw_body, content_type)
    → db.insert_webhook(payload)   [WEBHOOKS table, UUID key]
    → db.insert_webhook_event(outcome=Received)  [WEBHOOK_EVENTS ring buffer, cap=500]
    → 202 Accepted { id: uuid }
[On next daemon tick]
  → matched against registered WebhookRoute
  → template rendered, tool dispatched
```

---

## API Contract

### Action response shape (from `action_to_json`)
```json
{
  "id": "uuid",
  "label": "nightly-audit",
  "tool_name": "my-tool",
  "tool_input": {},
  "status": { "type": "pending" },
  "trigger": { "type": "scheduled", "next_tick_at": "ISO8601" },
  "recurrence_secs": 3600,
  "created_at": "ISO8601",
  "updated_at": "ISO8601"
}
```

### Webhook event response shape
```json
{
  "id": "uuid",
  "seq": 1,
  "route_path": "/github",
  "content_type": "application/json",
  "body_bytes": 512,
  "received_at": "ISO8601",
  "outcome": { "kind": "received" }
}
```

Outcome `kind` values: `received` | `no_route` | `routed` | `dispatch_error`

---

## Storage (redb tables — default backend)

| Table | Key | Purpose |
|-------|-----|---------|
| `ACTIONS` | `timestamp_ms_be ++ uuid_bytes` (24B) | Scheduled/webhook actions, ordered by due time |
| `WEBHOOKS` | `uuid_bytes` (16B) | Pending webhook payloads awaiting dispatch |
| `WEBHOOK_ROUTES` | `uuid_bytes` (16B) | Registered path→tool mappings |
| `WEBHOOK_EVENTS` | `seq_be ++ uuid_bytes` (24B) | Audit ring buffer, capped at 500 entries |

DB file: `.sdlc/orchestrator.db`
Sentinel: `.sdlc/.orchestrator.state` — written after each tick, watched by SSE watcher

---

## SSE

- Server emits: `event("action") { type: "action_state_changed" }`
- `SseContext.tsx` handles: `type === 'action'` → calls `onActionEvent`
- `ActionsPage.tsx` subscribes: `onActionEvent → refetchActions()`
- Fallback: 5-second polling interval as safety net

---

## Known Limitations

- Webhook events only record `Received` outcome on arrival. Dispatch outcomes (`Routed`, `DispatchError`) are defined in the model but not yet written by the daemon tick — so the event log currently shows all events as `Received`.
- `startup_recovery` (recover stuck `Running` actions to `Failed`) is called in CLI daemon mode but not on server startup.
