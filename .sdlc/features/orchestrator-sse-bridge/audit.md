# Security Audit: orchestrator-sse-bridge

## Scope

Changes introduced by this feature:

1. `SseMessage::ActionStateChanged` variant in `state.rs` — new enum variant, no network surface.
2. Sentinel watcher task in `AppState::new_with_port` — reads file metadata (mtime) of `.sdlc/.orchestrator.state`.
3. `ActionStateChanged` SSE arm in `events.rs` — serializes `{ "type": "action_state_changed" }` to the `orchestrator` SSE channel.
4. `write_tick_sentinel` in `orchestrate.rs` — writes JSON to `.sdlc/.orchestrator.state`.
5. `WebhookEvent` recording in `webhooks.rs` — calls `db.insert_webhook_event` after storing the raw payload.
6. `list_webhook_events` REST handler in `orchestrator.rs` — reads event ring buffer, returns JSON array.

## Findings

### F1 — Sentinel file path is fixed, not user-controlled

**Risk:** None.

The sentinel path is always `<root>/.sdlc/.orchestrator.state` constructed via `root.join(".sdlc").join(".orchestrator.state")`. The `root` value is set at server startup from a validated project root, not from request input. No path traversal risk.

**Action:** Accept.

### F2 — Sentinel file content is written by the daemon process, not from request input

**Risk:** None.

`write_tick_sentinel` writes JSON with: current UTC timestamp (from `chrono::Utc::now()`), and two `usize` counters (`due.len()`, `webhooks.len()`). No user-supplied data is included. The server only reads the file's **mtime** (via `tokio::fs::metadata`) — it never reads the file body.

**Action:** Accept.

### F3 — SSE event payload is a static string

**Risk:** None.

The `ActionStateChanged` arm emits `{"type":"action_state_changed"}` — a constant, no interpolated values. No injection surface.

**Action:** Accept.

### F4 — `list_webhook_events` returns event metadata, not raw payloads

**Risk:** Low.

The handler reads from `ActionDb::list_webhook_events()` and returns structured metadata: `id`, `seq`, `route_path`, `content_type`, `body_bytes`, `received_at`, and `outcome`. Raw payload bytes are **not** returned — they are stored in the separate `WEBHOOKS` table and deleted after dispatch. The `body_bytes` field is a count (usize), not the body itself.

The route is behind the server's standard auth middleware (same as all `/api/orchestrator/*` routes). No additional auth is required.

**Action:** Accept.

### F5 — `WebhookEvent` recording is best-effort

**Risk:** None / Correctness.

`receive_webhook` calls `db.insert_webhook_event` inside the same `spawn_blocking` task. On failure, a `tracing::warn!` is emitted and the 202 response is still returned. The raw payload is already safely stored in the `WEBHOOKS` table before the event write is attempted. Audit log completeness is best-effort, which is the correct trade-off for an ingestion endpoint.

**Action:** Accept.

### F6 — Ring buffer eviction (500 entries) prevents unbounded growth

**Risk:** None.

`insert_webhook_event` enforces a cap of 500 entries via `WEBHOOK_EVENTS_CAP`. Oldest entry is evicted before inserting the new one when at capacity. No DoS vector from an external webhook flood — each ingestion writes exactly one event, and the ring buffer stays bounded.

**Action:** Accept.

## Verdict

**No findings requiring action.** All changes are internal signal propagation (file mtime → SSE broadcast). The only external-facing addition is `list_webhook_events` which returns structured metadata behind the existing auth middleware. The feature introduces no new injection surfaces, no path traversal risks, and no unbounded resource consumption.

**APPROVED.**
