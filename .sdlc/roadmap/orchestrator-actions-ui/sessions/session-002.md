---
session: 2
timestamp: 2026-03-02T07:00:00Z
orientation:
  current: "SSE, webhook history, action editing, and recurrence text input confirmed — four design decisions locked, one architectural tension surfaced (daemon↔server SSE bridge via sentinel file)"
  next: "Build: ActionStateChanged SSE variant → WebhookEvent log table → PATCH action route → recurrence parse util → update ActionsPage with third section + edit modal"
  commit: "All four additions are scoped and designed — design is ready to commit into features"
---

## Context

Session 1 produced a complete Actions page design (two-section: Scheduled Actions + Webhook Routes) and locked sidebar placement. It deferred SSE, webhook history, action editing, and complex recurrence UI. Jordan re-opens the first three deferrals in session 2 and simplifies the fourth (recurrence).

## Team

- Ben Hartley · Developer UX Designer
- Felix Wagner · Developer Tooling Architect
- Dana Cho · Product Skeptic

---

## Session Dialogue

**jordan · Owner**
we need sse, we need webhook history, we need action editing --- its mostly just selecting a tool and understanding when it happens

can skip complex recurrence UI and tell them to type it, something like 1h, 10s

---

**Ben Hartley · Developer UX Designer**

Three things that were "nice to have" have been upgraded to load-bearing. Let me be direct about why.

SSE: The actions page without real-time feedback is a black box. A user schedules an action for "in 2 minutes," switches to the page, and sees `Pending`. The tick daemon fires. Nothing updates. They refresh manually and see `Completed`. That's not a product, that's a log file with better CSS. SSE is the difference between observing a system and reading about it after the fact.

Webhook history: Same argument, stronger. Webhooks are event-driven — the user has zero control over timing. If a GitHub push fires a webhook and the user opens the Actions page, they need to see "webhook received at 14:32, triggered action X, completed." Without that, the page is evidence of configuration, not evidence of behavior.

Action editing: I said last session that delete + recreate is acceptable UX. I was too generous. The specific pain case: someone creates an action with recurrence "every 1h" and wants to dial it to "every 6h." Delete + recreate loses the completion history on the original action. That history has diagnostic value. Editing is load-bearing when history matters.

On recurrence text: Jordan's call is correct. Free text is more expressive than four presets. "10s" for testing, "2h" for custom cadences — you can't enumerate all useful values with a dropdown. The select was a scope-reduction reflex, not a UX decision.

---

**Felix Wagner · Developer Tooling Architect**

Let me work through each one with the minimal primitive question.

**SSE:**

The SSE channel already exists. The question is: how does the tick daemon, which runs as a CLI subprocess, emit into the server's broadcast channel?

The daemon is not in-process with the server. Three options:

1. Daemon writes a sentinel file after each tick (e.g. `.sdlc/orchestrator/.last_tick`) — server's file watcher sees the mtime change, emits `SseMessage::ActionStateChanged`. Zero new API surface.
2. Daemon calls `POST /api/orchestrator/tick_event` — server emits to broadcast. Requires daemon to know the server address (from a port file or config).
3. Frontend polls `GET /api/orchestrator/actions` on a 5-second interval. No SSE at all.

Option 1 is the right call. The file watcher pattern is proven in this codebase (state.rs lines 295–370). The daemon writes `.sdlc/orchestrator/.last_tick` after each status transition. The server watches `.sdlc/orchestrator/` and emits `SseMessage::ActionStateChanged`. Frontend subscribes and refetches `/api/orchestrator/actions`. Decoupled by file system, not API contract.

Add `SseMessage::ActionStateChanged` as a specific variant — not the generic `Update`. `Update` triggers every SSE subscriber including ponder pages and artifact viewers. We don't want every action tick to cause the ponder session list to re-render.

**Webhook history:**

Current: `WEBHOOKS` table holds pending payloads, deleted after dispatch. History is gone.

Add a new table: `WEBHOOK_EVENTS`. Key: 24-byte composite (received_at_ms big-endian ++ UUID), same pattern as `ACTIONS`. Value: JSON `WebhookEvent`:

```rust
pub struct WebhookEvent {
    pub id: Uuid,
    pub path: String,              // matched route path (e.g. "/hooks/github")
    pub received_at: DateTime<Utc>,
    pub action_id: Option<Uuid>,   // action triggered, if any
    pub outcome: WebhookOutcome,
}

pub enum WebhookOutcome {
    Dispatched,
    NoRouteMatched,
    Rejected { reason: String },
}
```

No payload storage — storage bloat from payload bytes is real at scale. Path + outcome is sufficient for forensics. Users who want payload content log it in their tool.

Ring buffer: keep last 500 events. On write, if count > 500, delete the oldest. 500 events at ~200 bytes each = ~100KB. Negligible.

New route: `GET /api/orchestrator/webhooks/events?limit=50`. Returns events sorted by received_at desc.

Write the `WebhookEvent` to `WEBHOOK_EVENTS` inside the existing webhook ingestion handler — same transaction as the `WEBHOOKS` payload write.

**Action editing:**

Editable fields: `label` and `recurrence` only. `tool_name`, `tool_input`, and trigger type define *what* the action does — changing those is creating a new action.

`PATCH /api/orchestrator/actions/:id` body:
```json
{ "label": "new-label", "recurrence_secs": 3600 }
```

Both fields optional. `recurrence_secs: null` clears recurrence. Both absent → 200 with unchanged action (no-op).

`ActionDb::update_action(id: Uuid, label: Option<String>, recurrence: Option<Option<Duration>>)` — full table scan to find record by UUID. The ACTIONS key is `timestamp | uuid`, so UUID-only lookup requires a scan. At the volume of actions in a single project (dozens, maybe hundreds), this is not a hot path. Full scan is acceptable.

**Recurrence text:**

Frontend utility: `parseRecurrence(s: string): number | null` — handles "10s" → 10, "30m" → 1800, "1h" → 3600, "6h" → 21600. Regex: `/^(\d+)(s|m|h|d)$/`. Returns seconds. Returns `null` for anything that doesn't match — inline error below the field.

Reverse: `formatRecurrence(secs: number): string` — 10 → "10s", 3600 → "1h", 3601 → "3601s". No rounding.

API always receives `recurrence_secs: number`. Never sends raw string. Server validates: if present, must be > 0.

---

**Dana Cho · Product Skeptic**

I pushed back hard last session on SSE and webhook history as scope creep. I was wrong about both, and I want to be specific about why.

SSE: I said "static poll or manual refresh is fine for v1." That's true for artifact pages where nothing changes while you're watching. Actions are fundamentally different — they're running processes. The question "did it work?" needs an answer in real time, not on the next refresh. The comparison to a log file with better CSS is accurate and damning.

Webhook history: My concern was gold-plating before the basic model is proven. But Felix's scoping — metadata only, no payload, 500-event ring buffer, one new table, one new route — is already minimal. The value is high: users can immediately answer "did my webhook fire?" without touching the CLI. That's table stakes for a UI that claims to show you what's happening.

Editing: I asked "who changes recurrence without also questioning the tool and input?" The answer: ops teams adjusting cadence after initial setup. "We ran it every 1h for the first week and now it's stable, dial it to 6h." Label editing is real for renaming after the fact. Two fields, one modal, one PATCH route. Defensible.

On recurrence text: the presets were my suggestion, and I made a scope error. Simpler configuration for the agent doesn't mean better configuration for the operator. A developer who wants "every 45m" shouldn't be forced to choose 1h or 6h.

⚑ Decided: I'm not pushing back on any of Jordan's four asks. All are load-bearing, all are properly scoped.

---

**Ben Hartley · Developer UX Designer**

On the webhook history UI — where does it live?

The current design has two sections: Scheduled Actions + Webhook Routes. "Recent Webhook Events" becomes section three — stacked, not tabbed. Same visual pattern as the others.

Table columns for events: `Time | Path | Outcome | Action`. Short timestamps ("2m ago"), compact path display (`/hooks/github`), outcome badge (Dispatched=green, NoRouteMatched=gray, Rejected=red), action column links to the triggered action row if one exists.

Default view: 20 events. "Load more" link if more exist.

On the action edit modal: pencil icon in each row → modal pre-populated with current label and recurrence. Two fields only. Save → PATCH → optimistic update in the table. If PATCH fails, show inline error and revert.

On recurrence in the table: display as "every 1h" next to the trigger information. If recurrence is null, show "—". This gives users the at-a-glance check: "is this a one-shot or recurring?"

Empty state for webhook events: "No webhook events recorded. Events appear here after the first webhook request arrives."

---

**Felix Wagner · Developer Tooling Architect**

One subtle issue worth naming: the sentinel file approach for daemon→server SSE has a race condition. If the daemon fires and completes twice in rapid succession (two actions due at the same tick), the second mtime update may be swallowed by the file watcher's debounce window.

For v1 this is acceptable — the frontend can maintain a 5-second polling fallback, using SSE as the fast path. The sentinel file approach ships; the race is documented and tolerable.

Debounce duration: the existing watcher in state.rs uses 100ms. Verify the new `.sdlc/orchestrator/` watch path uses the same debounce. If the daemon emits faster than 100ms apart, the race window is real but self-correcting via poll fallback.

---

## Decisions

⚑ Decided: SSE is needed. Daemon writes `.sdlc/orchestrator/.last_tick` after each action status transition. Server watches `.sdlc/orchestrator/` directory and emits `SseMessage::ActionStateChanged`. Frontend Actions page re-fetches `/api/orchestrator/actions` on this event.

⚑ Decided: New `SseMessage::ActionStateChanged` variant — not generic `Update` — to avoid triggering unrelated SSE subscribers (ponder pages, artifact viewers, etc.).

⚑ Decided: Webhook history stored in new `WEBHOOK_EVENTS` redb table. `WebhookEvent` struct: id, path, received_at, action_id (Option), outcome (Dispatched | NoRouteMatched | Rejected). No payload storage. Ring buffer: last 500 events. New route: `GET /api/orchestrator/webhooks/events`.

⚑ Decided: Third section "Recent Webhook Events" added to Actions page — stacked below Webhook Routes, not tabbed. Columns: Time | Path | Outcome | Action. Default 20 events shown, "Load more" link.

⚑ Decided: Action editing scoped to `label` and `recurrence` only. `PATCH /api/orchestrator/actions/:id` with optional `label` and `recurrence_secs` fields. `ActionDb::update_action()` does full table scan to find by UUID. Frontend: pencil icon per row → pre-populated two-field modal.

⚑ Decided: Recurrence UI is a text input. Accepts "10s", "30m", "1h", "6h", "24h", etc. Frontend parses to seconds (`/^(\d+)(s|m|h|d)$/`) before sending to API. Table displays "every 1h" computed from stored seconds. Edit modal pre-populates from stored seconds back to human string. Inline error shown on invalid input.

## Open Questions

? Open: Sentinel file watcher debounce — verify `.sdlc/orchestrator/` uses the same 100ms debounce as existing watchers in state.rs. Document the accepted race condition in a code comment.

? Open: "Load more" pagination for webhook events — offset is fine for the 500-event ring buffer. Cursor-based pagination is future work if the buffer grows.

## What Was Deferred in Session 1 (now resolved)

- ~~SSE real-time updates~~ → shipping with sentinel file pattern
- ~~Webhook payload history/inspector~~ → shipping as metadata-only ring buffer
- ~~Action editing~~ → shipping for label + recurrence only
- Complex recurrence UI → replaced by free-text input (simpler, not deferred)
