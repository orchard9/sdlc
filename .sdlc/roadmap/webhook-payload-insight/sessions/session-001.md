---
session: 1
timestamp: 2026-03-05T00:00:00Z
orientation:
  current: "UI design shaped — mockup complete with 3 states, key decisions captured, two open questions flagged"
  next: "Implement: add store_only chip + Inspect button to WebhookRoutesSection in ActionsPage, wire to payload inspector panel"
  commit: "Backend endpoint GET /api/webhooks/{route}/data and POST /api/webhooks/{route}/replay/{id} are implemented and returning data"
---

## Context

Jordan is building webhook query infrastructure (Phase 1) and a Telegram-recap rewrite (Phase 2). The UI request: a surface for **insight and replay** of stored webhook payloads — specifically for store_only routes where tools like telegram-recap accumulate messages for later querying.

## What we read

- `crates/sdlc-server/src/routes/webhooks.rs` — current webhook handler: write-only, no query endpoint
- `frontend/src/pages/ActionsPage.tsx` — existing webhook UI: routes table + events log, no payload body access
- `frontend/src/lib/types.ts` — `OrchestratorWebhookRoute` and `OrchestratorWebhookEvent` types

## Key observations

**The gap:** The existing Actions page shows webhook _events_ (metadata: path, time, outcome) but has no access to the stored _payload bodies_. When a store_only route is configured, nothing in the UI lets you see what actually arrived.

**The opportunity:** With `GET /api/webhooks/{route}/data?since=...&until=...` being added, the data is queryable. The UI just needs to surface it.

## Design exploration

Three approaches considered:
1. **Separate page**: `/webhooks/:route` — cleanest URL, but breaks context from the route config
2. **Modal**: Overlay with list + detail — familiar but cramped for JSON inspection
3. **Inline panel**: Expands below route row or replaces section body — keeps context, no navigation

⚑ Decided: **Inline panel** wins. Actions page is already a scrolling config page. Drilling into a panel without losing the route config context feels right. Mail-client two-pane layout (list left, detail right) is the right mental model for "browse many payloads, read one."

## Design: three states

**State 1 — Route row**: store_only routes get a blue "store-only" chip and an "Inspect" button. Dispatch routes don't.

**State 2 — Panel empty**: Time window filter (1h/6h/24h/7d chips) + empty state explaining Telegram needs the webhook registered.

**State 3 — Panel populated**: Two-pane layout. Payload list left (time, content-type, size, body preview). JSON viewer right with syntax highlighting. Replay button in detail header. Replay result shown as footer strip after dispatch.

## Replay mechanics

⚑ Decided: Replay goes through a new server endpoint `POST /api/webhooks/{route}/replay/{id}` that re-dispatches the stored raw_body through the tool registered on the route. This bypasses the public `/webhooks/{route}` ingress (which would create duplicate storage and hit secret verification).

? Open: Should replay always target the registered tool, or allow picking a different tool? V1: always registered tool. Simpler.

? Open: Retention policy. store_only payloads grow unbounded. V1: warning banner if count > 10,000.

## Artifacts captured

- `brief.md` — user intent + context
- `webhook-payload-inspector-mockup.html` — 3-state HTML design prototype
- `decisions.md` — all design decisions + open questions

## Product Summary

### What we explored
The UI design for browsing and replaying stored webhook payloads — specifically for store_only routes where external services push data that tools query later. We went from "we need a UI" to a concrete three-state design with defined entry points, layout, and replay mechanics.

### Key shifts
The design crystallized around the mail-client mental model (list + detail), not a separate page or modal. Replay was scoped tightly: re-dispatch through the registered tool via a server-side endpoint, not re-ingestion through the public webhook URL.

### Implications
The frontend work is well-defined: extend WebhookRoutesSection in ActionsPage with a store_only chip + Inspect button, build the inline two-pane inspector panel, wire time-range filter to the new query endpoint, and add replay via a new server endpoint. This is a self-contained UI addition — no architectural changes.

### Still open
1. Does replay always target the registered tool, or should V1 allow picking a different one?
2. What's the retention policy for store_only payloads — should the UI surface a warning when the count exceeds a threshold?
