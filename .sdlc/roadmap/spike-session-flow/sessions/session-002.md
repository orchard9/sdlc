---
session: 2
timestamp: 2026-03-04T23:05:00Z
orientation:
  current: "All decisions resolved. Design is complete and ready to commit."
  next: "Run /sdlc-ponder-commit spike-session-flow to distribute into milestones and features."
  commit: "READY — all open questions resolved, architecture approved."
---

# Session 2 — Resolving Open Questions

## Context restored from Session 1

Two-phase design:
- Phase 1: autonomous investigation, SSE stream, user watches
- Phase 2: Q&A + verdict, chat interface, persistent connection

Two open questions from Session 1:
1. Phase 2 chat: per-message spawns vs. persistent connection?
2. "New Spike" button: always visible vs. empty state only?

## Decisions

⚑ Decided: **Phase 2 = persistent connection** (like ponder chat)

User chose persistent over per-message. The reasons are aligned with ponder's model:
- Agent remembers conversation context across follow-up questions naturally
- No need to re-pass conversation history as context to each fresh spawn
- The pattern already exists: `POST /api/spikes/:slug/chat` → `spawn_agent_run`, stays alive, `DELETE /api/spikes/:slug/chat/current` to stop
- For a bounded Q&A (1–5 questions before verdict) the cost is acceptable
- Consistent with ponder's philosophy: workspaces are live, not batch

⚑ Decided: **Button matches ponder — `+` icon in left pane header, `NewSpikeModal` component**

User wants UI alignment — consistent workspace components across ponder and spikes.
The pattern:
- `Plus` icon button next to "Spikes" heading in left pane header
- `NewSpikeModal` component mirrors `NewIdeaModal` exactly in structure
- Field mapping:
  - "The Question" ← ponder's "Title" (required, what the spike answers)
  - "Slug" ← auto-derived from question, editable (same `titleToSlug` utility)
  - "Context" ← ponder's "Description" (optional, why this matters, what to look for)
  - "References" ← ponder's "References" (optional URLs/repos as starting points)
- On submit: create spike directory + trigger `api.startSpikeRun(slug)` (Phase 1)
- Navigate to `/spikes/:slug` after creation

## Final architecture (complete)

### API endpoints

| Method | Path | Purpose |
|---|---|---|
| GET | /api/spikes | List spikes (existing) |
| GET | /api/spikes/:slug | Spike detail + findings content (existing) |
| POST | /api/spikes/:slug/promote | Promote ADAPT → ponder (existing) |
| POST | /api/spikes/run | Start Phase 1 investigation |
| POST | /api/spikes/:slug/chat | Start/continue Phase 2 Q&A (persistent) |
| DELETE | /api/spikes/:slug/chat/current | Stop Phase 2 session |
| PATCH | /api/spikes/:slug/verdict | Override verdict in findings.md |

### SSE events

| Event | When |
|---|---|
| SpikeRunStarted { slug } | Phase 1 begins |
| SpikeRunCompleted { slug } | Phase 1 ends (findings.md written) |
| SpikeRunStopped { slug } | Phase 1 stopped by user |
| SpikeChatStarted { slug } | Phase 2 session begins |
| SpikeChatCompleted { slug } | Phase 2 session ends |
| SpikeChatStopped { slug } | Phase 2 stopped by user |

### UI components

| Component | Description |
|---|---|
| NewSpikeModal | Mirrors NewIdeaModal: question + slug + context + references |
| SpikeRunFeed | SSE stream display (Phase 1, read-only) |
| SpikeChatPane | Persistent chat (Phase 2), mirrors ponder's chat panel |
| SpikeVerdictBar | ADOPT/ADAPT/REJECT buttons, pre-selected from findings.md |

### Data model (no new fields)

findings.md remains the single artifact. Verdict pre-populated from `**Verdict:**` line.
PATCH /api/spikes/:slug/verdict updates that line in-place.

No new status field. Implicit: no findings.md = running/pending; findings.md exists = complete.

## Product Summary

### What we explored
Resolved the two remaining design decisions for the spike session flow feature, and confirmed the final architecture is complete and ready to build.

### Key shifts
Both open questions from Session 1 are closed. Persistent chat was chosen over per-message for Phase 2, keeping consistency with ponder's live workspace model. Button placement matches ponder exactly — same left pane header Plus icon, same modal component structure — so the two workspaces feel unified rather than divergent.

### Implications
The feature is fully designed and buildable. The work decomposes cleanly into backend (3 new routes, 6 SSE variants) and frontend (NewSpikeModal + SpikeRunFeed + SpikeChatPane + SpikeVerdictBar). No data model changes beyond an in-place verdict patch. This can ship as a single milestone.

### Still open
Nothing. Design is complete and approved.
