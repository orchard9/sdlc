# Tick Orchestrator — Commit Plan

## Summary

A game-loop orchestrator that fires tools on a tick rate or via webhook. The
execution model is: Action = trigger (Scheduled | Webhook) + tool. The tick loop
calls `run_tool()` — the same function the UI already uses. Backed by redb
(embedded, ACID, range-scannable, pure Rust).

Two milestones. Phase 1 proves the tick loop. Phase 2 adds webhook ingestion.

---

## Milestone 1: v07-orchestrator-core

**Vision:** A developer can run `sdlc orchestrate` to start a tick-rate daemon
that fires tools on a schedule. Services that need to run `sdlc next` on every
tick can be registered as recurring actions and will advance autonomously without
human intervention.

**Acceptance test:**
1. Run `sdlc orchestrate add my-svc --tool quality-check --input '{}' --at now+5s`
2. Run `sdlc orchestrate --tick-rate 10`
3. Within 10 seconds, quality-check fires and its result is stored
4. Run the add command again; the daemon picks it up on the next tick
5. Kill and restart the daemon; no action fires twice; any Running actions are recovered

### Features

**orchestrator-action-model**
Title: Action struct and ActionDb in sdlc-core with redb backing
Description: Add `crates/sdlc-core/src/orchestrator/mod.rs`, `action.rs`, `db.rs`.
- `Action` struct: id (Uuid), tool_name (String), tool_input (serde_json::Value), trigger (ActionTrigger), status (ActionStatus), recurrence (Option<Duration>), created_at, updated_at
- `ActionTrigger`: Scheduled { next_tick_at: DateTime<Utc> } | Webhook { raw_payload: Vec<u8>, received_at: DateTime<Utc> }
- `ActionStatus`: Pending | Running | Completed { result: serde_json::Value } | Failed { reason: String }
- `ActionDb`: wraps redb Database, table key = [u8; 24] (timestamp_ms big-endian ++ uuid bytes)
- Methods: insert(action), set_status(id, status), range_due(now) → Vec<Action>, startup_recovery(max_age)
- Unit tests: insert + range_due returns correct actions, startup_recovery marks stale Running as Failed
- Add redb to workspace.dependencies in Cargo.toml

**orchestrator-tick-cli**
Title: sdlc orchestrate daemon with tick loop and add subcommand
Description: Add `crates/sdlc-cli/src/cmd/orchestrate.rs`.
- `sdlc orchestrate [--tick-rate <secs>] [--db <path>]` — starts the tick loop daemon
- Tick loop: range_due() → for each: set Running → run_tool() → set Completed/Failed → reschedule if recurrence set
- `sdlc orchestrate add <label> --tool <name> --input <json> [--at <timestamp|now+Ns>] [--every <duration>]` — inserts an action
- `sdlc orchestrate list` — shows all actions with status
- Startup recovery on boot: call startup_recovery(2 * tick_rate)
- DB default path: `.sdlc/orchestrator.db` (add to .gitignore)
- Add orchestrate to the CLI command dispatch in main.rs / cmd/mod.rs

**orchestrator-integration-test**
Title: Integration test — two scheduled actions fire within one tick window
Description: Write integration test in `crates/sdlc-core/tests/orchestrator.rs` or `crates/sdlc-cli/tests/orchestrate.rs`.
- Create a TempDir with a real quality-check tool stub (or use actual quality-check)
- Insert two actions: t+100ms and t+200ms
- Run tick loop once with tick_rate=500ms
- Assert: both actions have status Completed, elapsed < 600ms
- Assert: startup_recovery marks a manually-inserted Running action as Failed

---

## Milestone 2: v08-orchestrator-webhooks

**Vision:** A developer can register a webhook route (`POST /webhooks/deploy → run
deploy-tool`) and any incoming HTTP POST will be stored raw and processed on the
next tick. Services can be triggered externally without polling.

**Acceptance test:**
1. Register: `POST /api/orchestrator/webhooks { path: "/webhooks/deploy", tool_name: "quality-check", input_template: "{}" }`
2. POST to `/webhooks/deploy` with any JSON body
3. Webhook stored in redb
4. On next tick (≤ tick_rate seconds later), quality-check fires with the mapped input
5. `GET /api/orchestrator/actions` shows the completed webhook-triggered action

### Features

**orchestrator-webhook-storage**
Title: HTTP webhook receiver and raw payload storage in redb
Description: Add a `webhooks` redb table (key: uuid bytes, value: raw HTTP body + metadata).
- Add `POST /webhooks/:route` to sdlc-server — accepts any body, stores raw bytes + received_at + route path
- Add `WebhookPayload` struct to sdlc-core
- `ActionDb` gains: insert_webhook(payload), all_pending_webhooks(), delete_webhook(id)
- No transformation on ingress — store exactly what arrived

**orchestrator-webhook-routing**
Title: WebhookRoute registration and tick dispatch
Description: Add `WebhookRoute` struct and registration API.
- `WebhookRoute`: id, path (String), tool_name (String), input_template (String — e.g., "{{payload}}")
- `POST /api/orchestrator/webhooks/routes` — register a route
- `GET /api/orchestrator/webhooks/routes` — list routes
- Tick loop extended: after scheduled actions, read all_pending_webhooks(), match against routes, render template, run_tool(), delete webhook
- Simple template rendering: `{{payload}}` → raw JSON body string
