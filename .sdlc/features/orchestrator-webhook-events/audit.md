# Security Audit: Webhook Event History

**Feature:** orchestrator-webhook-events
**Scope:** `WebhookEvent` data model, ring-buffer storage (`db.rs`), event emission in `webhooks.rs`, GET handler in `orchestrator.rs`, route registration in `lib.rs`.

---

## Surface Analysis

This feature adds two concerns to the attack surface:

1. **New read endpoint** — `GET /api/orchestrator/webhooks/events` exposes the ring-buffered event history to callers.
2. **Side-channel write on every webhook arrival** — `POST /webhooks/*path` now writes a `WebhookEvent` in addition to the `WebhookPayload`.

---

## Finding 1: GET endpoint is covered by tunnel auth middleware

**Severity:** Informational (no action required)

`GET /api/orchestrator/webhooks/events` starts with `/api/` — the tunnel auth middleware gates all `/api/*` paths when a tunnel is active (including app-tunnel-host requests, per auth.rs line 94). Local access (localhost / 127.0.0.1) always passes through, which is correct for a developer tool.

No unauthenticated read exposure via the tunnel path.

**Action:** Accept.

---

## Finding 2: Webhook ingestion route leaks metadata into event history

**Severity:** Low (accepted design decision)

`POST /webhooks/*path` is intentionally public — it must be reachable by external senders (GitHub, Stripe, CI systems). The new feature records a `WebhookEvent` on every arrival, storing: route path, content-type, body byte count, and timestamp. It does **not** store the request body in the event record (body bytes is only a count, not the bytes themselves). The raw body lives only in `WebhookPayload`.

The event record could reveal that a particular service is sending webhooks (route path) and approximately how large the payloads are (byte count). This is visible to anyone who can read `GET /api/orchestrator/webhooks/events` — i.e., authenticated users only (see Finding 1).

**Action:** Accept. The metadata exposure is intentional and limited. Raw body bytes are not stored in events.

---

## Finding 3: No size limit on `body_bytes` field (theoretical integer overflow)

**Severity:** Note (no action required)

`body_bytes: usize` is captured as `body.len()` before the spawn_blocking closure. On 64-bit platforms `usize` cannot overflow from a single HTTP body. axum enforces a default body size limit at the service layer. No truncation or validation of `body_bytes` is needed.

**Action:** Accept.

---

## Finding 4: Best-effort event insert does not retry on failure

**Severity:** Note (no action required)

If `insert_webhook_event` fails (e.g., redb lock contention), the event is silently dropped after logging a `tracing::warn!`. The webhook payload itself is already committed. This means the event history could have gaps under extreme contention. Gaps in audit history are acceptable for a best-effort ring buffer; the payload store is the authoritative record.

**Action:** Accept. The design doc explicitly calls for best-effort event logging.

---

## Finding 5: WEBHOOK_EVENTS_CAP is a hard-coded constant, not configurable

**Severity:** Note (no action required)

The ring buffer is capped at 500 events. This is not user-configurable. At 500 small JSON objects (each typically < 512 bytes), the maximum storage is well under 1 MB. No DoS risk from unbounded growth. If a future requirement needs a configurable cap, it can be added as a config field; no design change is needed now.

**Action:** Accept.

---

## Finding 6: Route ordering — events route registered before wildcard

**Severity:** Informational (confirmed correct)

`/api/orchestrator/webhooks/events` is registered before any wildcard webhook ingestion path. axum matches the most specific route first, so GET requests to `/api/orchestrator/webhooks/events` will not be swallowed by a broader pattern. Confirmed by inspecting `lib.rs`.

**Action:** Accept.

---

## Summary

| # | Severity | Finding | Disposition |
|---|---|---|---|
| 1 | Info | GET endpoint protected by tunnel auth | Accept |
| 2 | Low | Route/metadata visible to authenticated users | Accept — by design |
| 3 | Note | body_bytes usize on 64-bit is safe | Accept |
| 4 | Note | Best-effort insert can drop events under contention | Accept — by design |
| 5 | Note | Cap is not configurable | Accept |
| 6 | Info | Route ordering confirmed correct | Accept |

**No open findings. No code changes required.**

---

## Verdict

**APPROVED.** The feature introduces no new unauthenticated read surface, does not store raw request bodies in the event log, and the ring-buffer write path is best-effort with appropriate warning logging. All findings are accepted with documented rationale.
