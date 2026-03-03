---
session: 1
timestamp: 2026-03-02T23:50:00Z
orientation:
  current: "Idea fully shaped: event log backbone + dashboard banner for V1; CLI and notification API for V2/V3"
  next: "Run /sdlc-ponder-commit release-notes-since-last-look to crystallize into milestones and features"
  commit: "Event schema finalized, V1 scope agreed — ready to commit"
---

# Session 1 — Release Notes: What Changed Since You Last Looked

## Setup
Created ponder. Captured brief. Recruited Ben Hartley (UX), Dana Cho (skeptic), Priya Nair (state/data model).

Read codebase context:
- `.sdlc/.runs/*.json` — run files already exist, capture agent run history with timestamps
- `SseMessage` enum — rich SSE events, but fire-and-forget (not persisted)
- No persistent event log exists for: feature merges, phase transitions, artifact approvals

---

## Interrogating the Brief

The phrase "works in all scenarios" signals the user already knows this is multi-dimensional.
Identified 5 distinct scenarios, each with a different identity model for "you."

---

**Ben Hartley · Developer Productivity UX Designer**

> "Stop. Before you design the feature, ask: what does the user DO when they see 'what changed'? If the answer is 'they read it and close it,' you've built a status page. If the answer is 'they decide what to do next,' you've built a navigation tool.
>
> In my GitHub work, the 'viewed' file state wasn't about information — it was about *resumption*. 'Where do I pick back up?' That's what your users need. The 'since your last visit' framing is the right one for web UI, and it's actually simple: localStorage. Store last_visit_at on unload. On load, query everything newer. Show count badge. Click to expand."

---

**Dana Cho · Product Skeptic**

> "I'll be direct: 'what changed since you last looked' is a solution. What's the problem?
>
> Three distinct problems are hiding here: re-orientation (returning human), context-loading (agent), daily digest (notification). These have different answers. Don't build a unified system expecting it to serve all three — you'll get a lukewarm product.
>
> My call: build the web UI banner. It's the highest pain, clearest solution. Agent changelog is a non-problem — agents have `sdlc next` and the oracle. Don't build infrastructure for a problem that doesn't exist."

---

**Priya Nair · Distributed Systems Engineer**

> "The 'since you last looked' model has a fundamental identity problem: 'you' is different in every scenario. Web UI: localStorage, simple. CLI: --since flag. Agent: run.started_at. Webhook consumer: their own cursor.
>
> Don't build unified last-seen tracking. Each caller tracks its own cursor.
>
> What DOES need to be unified is the event log — the data source. Right now, run files exist but feature phase transitions and artifact changes are NOT captured as discrete events. That's the gap."

---

## Key Decisions

⚑ Decided: The core problem is a missing event log
Run files capture agent runs. Feature transitions, artifact approvals, merges are only inferable from file state. A changelog needs a proper event source.

⚑ Decided: Per-consumer 'since' tracking, not unified
- Web UI → localStorage last_visit_at
- CLI → --since flag (date or relative)
- Agents → run.started_at as the natural cursor (not our problem to build)
- Notification consumers → their own cursor

⚑ Decided: .sdlc/changelog.yaml as single append-only file
Single file. Bounded size (<500KB for any real project lifecycle). Simplest.

⚑ Decided: V1 scope — web UI only, event log + dashboard banner
1. event_log.rs in sdlc-core — append-only, YAML, high-signal events only
2. Emit events in CLI paths (merge, approve, phase transition, run failure)
3. GET /api/changelog?since=<timestamp> endpoint
4. Web UI: localStorage tracking + dashboard "what changed" banner with expandable feed

⚑ Decided: V2 = sdlc changelog CLI, V3 = notification consumer API, Agent changelog = never

## Event Triage

High signal (always emit):
- feature_merged — highest signal
- run_failed — actionable, needs attention
- milestone_wave_completed — positive signal
- feature_phase_advanced (to IMPLEMENTATION or beyond)

Medium signal (emit on significant approvals only):
- review_approved, audit_approved, qa_approved

Never emit (noise):
- artifact draft, artifact reject, ponder sessions, advisory runs, knowledge research

## UX Model (Ben Hartley's design)

Dashboard banner: "7 changes since you were last here (2 days ago) [Expand ▼] [Dismiss]"

Expanded feed, ordered: ⚠️ Failed runs first (most actionable), 🚀 Merges, ✅ Approvals, 🔄 Phase transitions

Dismiss → updates last_visit_at without reading items.

## Open Questions

? Open: Should failed runs show a [Retry →] action button inline in the feed?
Ben: yes, if we have the run ID we can wire it to the existing retry mechanism.

? Open: How many events to show in the banner before "show all"?
Recommendation: 5 in the banner, "See all X changes →" link to a full feed page or expanded view.

? Open: Does the feed need its own page or is the banner + expand pattern enough?
Dana: Banner + expand is enough for V1. Don't build a page until users ask for it.

## Artifacts Captured
- scenarios.md — 5 scenarios, priority order, identity model per scenario
- event-schema.md — event kinds, file format, query API, what to emit/not emit
- ux-model.md — dashboard banner design, CLI model, notification API
- v1-implementation-sketch.md — Rust types, file locations, frontend component sketch, ~3 day estimate

## Status
READY TO COMMIT. V1 scope is clear and bounded. Event schema is finalized. Implementation path is unambiguous.
