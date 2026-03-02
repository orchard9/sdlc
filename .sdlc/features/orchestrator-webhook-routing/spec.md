# Spec: WebhookRoute Registration and Tick Dispatch

## Problem

The orchestrator tick loop can process `ActionTrigger::Webhook` actions, but there is no way to register a persistent mapping from a webhook path to a tool. Each inbound webhook payload must be manually turned into an `Action` record — there is no routing layer that says "when `/hooks/my-service` fires, call tool `my-tool` with this template".

This means webhook-driven automation requires out-of-band code to receive payloads and inject `Action` records, which defeats the purpose of a self-contained orchestrator.

## Goal

Add a first-class **WebhookRoute** registry: a persistent mapping from an HTTP path to a tool name + input template. When the tick loop sees a pending webhook action, it looks up the matching route, renders the template, and dispatches the tool. No external glue code required.

## Data Model

```rust
pub struct WebhookRoute {
    pub id: Uuid,
    pub path: String,           // e.g. "/hooks/my-service"
    pub tool_name: String,      // matches .sdlc/tools/<name>/
    pub input_template: String, // JSON template; {{payload}} → raw JSON string of webhook body
    pub created_at: DateTime<Utc>,
}
```

Routes are stored in a separate redb table (`webhook_routes`) in the same orchestrator database file (`.sdlc/orchestrator.db`). Using the same file avoids introducing a second database and keeps route lookup and webhook action reads within a single atomic transaction boundary when needed.

## Webhook Action Dispatch

The existing `ActionTrigger::Webhook { raw_payload, received_at }` variant carries the raw bytes of the inbound payload. Currently the tick loop processes all due `Pending` actions uniformly — it calls `run_tool()` directly using the action's stored `tool_name` and `tool_input`.

With webhook routing, the dispatch logic changes for webhook-triggered actions:

1. After processing all scheduled actions (existing loop), read all `Pending` webhook actions from the DB via a new `all_pending_webhooks()` method.
2. For each webhook action:
   a. Match the action against the route table using `action.label` as the route path key (the label is set to the path when the webhook is received).
   b. If no route matches, mark the action `Failed` with reason "no route registered for path".
   c. If a route matches, render the template: replace `{{payload}}` with the raw JSON string of the webhook payload bytes.
   d. Parse the rendered template as JSON to produce `tool_input`.
   e. Call `run_tool()` with the route's `tool_name` and rendered `tool_input`.
   f. Mark the action `Completed` or `Failed` per the tool result.
   g. After dispatch (success or failure), delete the webhook action from the DB to prevent reprocessing.

The `all_pending_webhooks()` method returns all `Pending` actions whose trigger is `ActionTrigger::Webhook`, regardless of timestamp — webhook actions are always processed on the next tick after receipt.

## REST API

### POST /api/orchestrator/webhooks/routes

Register a new webhook route.

**Request body:**
```json
{
  "path": "/hooks/my-service",
  "tool_name": "my-tool",
  "input_template": "{\"event\": {{payload}}}"
}
```

**Response:** `201 Created` with the created `WebhookRoute` as JSON.

**Errors:**
- `400` if `path` is empty or does not start with `/`
- `400` if `tool_name` is empty
- `400` if `input_template` is empty
- `409` if a route with the same `path` already exists

### GET /api/orchestrator/webhooks/routes

List all registered webhook routes.

**Response:** `200 OK` with a JSON array of `WebhookRoute` objects, ordered by `created_at` ascending.

### POST /api/orchestrator/webhooks/:path*

Receive an inbound webhook payload. This is the public-facing endpoint that external services call.

- Reads the raw request body as bytes.
- Creates an `ActionTrigger::Webhook { raw_payload, received_at: Utc::now() }` action with `label = path` and inserts it into the DB.
- The tick loop will process it on the next tick.
- Returns `202 Accepted` immediately.

**Note:** Path matching for routing happens at tick time, not at receipt time — this endpoint does not validate that a route exists for the path. This keeps the ingestion path fast and decoupled from routing configuration.

## Storage

- Table name: `webhook_routes`
- Key: UUID bytes (16 bytes, big-endian)
- Value: JSON-encoded `WebhookRoute`

The table is created (if absent) when `ActionDb::open()` is called, alongside the existing `actions` table. This is a backward-compatible change — existing databases simply gain a new empty table.

## Template Rendering

The `input_template` field is a string containing valid JSON with `{{payload}}` as a placeholder. During rendering, `{{payload}}` is replaced with the raw webhook body serialized as a JSON string (i.e. the bytes are interpreted as UTF-8 and then JSON-escaped). The resulting string must be valid JSON after substitution; if parsing fails, the action is marked `Failed`.

Example:
- Template: `{"event": {{payload}}, "source": "github"}`
- Payload bytes: `{"action":"push","ref":"refs/heads/main"}`
- Rendered input: `{"event": "{\"action\":\"push\",\"ref\":\"refs/heads/main\"}", "source": "github"}`

## CLI

No new CLI subcommands in this feature. Route management is done via the REST API. The existing `sdlc orchestrate` daemon command picks up the new webhook dispatch logic automatically at startup.

Future work may add `sdlc orchestrate routes list` and `sdlc orchestrate routes add` subcommands.

## Acceptance Criteria

1. `POST /api/orchestrator/webhooks/routes` with valid body → `201` with `WebhookRoute` JSON including `id` and `created_at`.
2. `GET /api/orchestrator/webhooks/routes` → `200` with array of registered routes.
3. `POST /api/orchestrator/webhooks/hooks/my-service` with payload → `202 Accepted`, and an `Action` with `ActionTrigger::Webhook` is inserted into the DB.
4. On the next `run_one_tick`, the webhook action is matched to its route, the template is rendered, `run_tool()` is called, and the action is removed from the DB.
5. If no route is registered for the path, the action is marked `Failed` with an appropriate message and removed.
6. Registering a duplicate path returns `409`.
7. The `webhook_routes` table is created automatically when the DB is opened; existing DBs are unaffected.
8. All new code passes `cargo clippy --all -- -D warnings` and has unit tests for route CRUD and webhook dispatch logic.

## Out of Scope

- Authentication/signing for inbound webhooks (e.g. HMAC verification) — add as a follow-on feature.
- CLI subcommands for route management.
- Template languages beyond `{{payload}}` substitution.
- Multiple routes per path (first-match wins is sufficient for now).
