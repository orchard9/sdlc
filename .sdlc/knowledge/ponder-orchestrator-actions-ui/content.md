---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Design complete — two-section Actions page with backend gaps identified"
  next: "Build: 2 new Rust routes → 4 API client methods → ActionsPage.tsx → sidebar + route"
  commit: "Design is ready to commit into a feature; no open questions remain"
---

## Context

The orchestrator backend is complete (Action model, ActionDb redb, tick daemon, webhook ingestion + routing).
The frontend has zero UI. Three gaps: Actions page, sidebar nav entry, API client methods.

## Team

- Ben Hartley · Developer UX Designer
- Felix Wagner · Developer Tooling Architect
- Dana Cho · Product Skeptic

## Brief

Build a UI for the orchestrator under Setup → Actions. Users need to see action health at a glance,
schedule new actions, and manage webhook routes.

---

## Session Dialogue

**Ben Hartley · Developer UX Designer**

Scheduled actions and webhook routes are different mental models — temporal vs event-driven. Don't mix
them in one table. Two sections on the page. Status badges are the key signal: Pending/Running/Completed/
Failed needs to be scannable immediately.

**Felix Wagner · Developer Tooling Architect**

Two API holes: `GET /api/orchestrator/actions` (ActionDb::list_all() exists, no HTTP route) and
`POST /api/orchestrator/actions` (CLI does this, server doesn't). Two thin handlers, no business logic.
Also flagged: what happens when the daemon isn't running? UI should be honest — stale data is worse than no data.

**Dana Cho · Product Skeptic**

Challenged scope: CLI already works, who uses a UI? The only defensible MVP is visibility — seeing action
health at a glance. Did anything fail? What's stuck in Running? That's real value the CLI can't give easily.
Add/delete is secondary.

---

## Decisions

⚑ Decided: MVP is visibility-first, CRUD second. The status table is the core deliverable.

⚑ Decided: Two sections (not tabs) — Scheduled Actions + Webhook Routes.

⚑ Decided: Actions goes in the `setup` sidebar group. Icon: Zap.

⚑ Decided: Delete is v1 scope. Editing is not (delete + recreate is acceptable).

## Open Questions

None — design is complete and ready to build.

## What's Deferred

- SSE real-time updates (static poll or manual refresh is fine for v1)
- Webhook payload history/inspector (backend doesn't persist after dispatch)
- Action editing
- Complex recurrence UI (just None / 1h / 6h / 24h presets)

---

---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Design complete — two-section Actions page with backend gaps identified"
  next: "Build: 2 new Rust routes → 4 API client methods → ActionsPage.tsx → sidebar + route"
  commit: "Design is ready to commit into a feature; no open questions remain"
---

## Context

The orchestrator backend is complete (Action model, ActionDb redb, tick daemon, webhook ingestion + routing).
The frontend has zero UI. Three gaps: Actions page, sidebar nav entry, API client methods.

## Team

- Ben Hartley · Developer UX Designer
- Felix Wagner · Developer Tooling Architect
- Dana Cho · Product Skeptic

## Brief

Build a UI for the orchestrator under Setup → Actions. Users need to see action health at a glance,
schedule new actions, and manage webhook routes.

---

## Session Dialogue

**Ben Hartley · Developer UX Designer**

Scheduled actions and webhook routes are different mental models — temporal vs event-driven. Don't mix
them in one table. Two sections on the page. Status badges are the key signal: Pending/Running/Completed/
Failed needs to be scannable immediately.

**Felix Wagner · Developer Tooling Architect**

Two API holes: `GET /api/orchestrator/actions` (ActionDb::list_all() exists, no HTTP route) and
`POST /api/orchestrator/actions` (CLI does this, server doesn't). Two thin handlers, no business logic.
Also flagged: what happens when the daemon isn't running? UI should be honest — stale data is worse than no data.

**Dana Cho · Product Skeptic**

Challenged scope: CLI already works, who uses a UI? The only defensible MVP is visibility — seeing action
health at a glance. Did anything fail? What's stuck in Running? That's real value the CLI can't give easily.
Add/delete is secondary.

---

## Decisions

⚑ Decided: MVP is visibility-first, CRUD second. The status table is the core deliverable.

⚑ Decided: Two sections (not tabs) — Scheduled Actions + Webhook Routes.

⚑ Decided: Actions goes in the `setup` sidebar group. Icon: Zap.

⚑ Decided: Delete is v1 scope. Editing is not (delete + recreate is acceptable).

## Open Questions

None — design is complete and ready to build.

## What's Deferred

- SSE real-time updates (static poll or manual refresh is fine for v1)
- Webhook payload history/inspector (backend doesn't persist after dispatch)
- Action editing
- Complex recurrence UI (just None / 1h / 6h / 24h presets)
