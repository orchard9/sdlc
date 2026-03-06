---
session: 1
timestamp: 2026-03-04T22:55:00Z
orientation:
  current: "Two-phase design shaped: Phase 1 = autonomous investigation, Phase 2 = Q&A + user verdict. Two open questions remain."
  next: "Decide: per-message vs. persistent Phase 2 chat. Then commit to milestones if design is approved by user."
  commit: "User approves the two-phase design and the minimal data model (no new status field)."
---

# Session 1 — Spike Session Flow Design

## Context

User wants the Spike page to behave more like Ponder — conversational, with the agent
working through the investigation in front of the user, and a verdict the user confirms
from the UI rather than discovers after the fact.

Current state of the Spike page: viewer-only. List of completed spikes, verdict badges,
promote-to-ponder button for ADAPT. No way to create a spike from the UI. No live feed
while running.

## Team

Recruited: Yuki Tanaka · Streaming UX Engineer (Linear, Vercel background)
Recruited: Sam Okeke · Developer Tools PM & Skeptic (JetBrains, indie tools background)

## Exploration

### The real problem

Current spike lifecycle from user perspective:
1. Run `/sdlc-spike slug — question` in Claude Code
2. Wait 10–20 minutes
3. Check the Spikes page. See a verdict.
4. Click a button or copy a command.

The user is never in the room. The investigation happens but they don't see it.
The verdict appears as a fait accompli rather than a conclusion they reached together.

### Three different things "chat while running" could mean

**Yuki Tanaka** clarified the design space:

1. **Watch mode** — see the agent's tool calls stream live (like a deployment log). Frontend-only change, SSE already exists.
2. **Interactive steering** — send messages mid-run that the agent incorporates. Architecturally complex.
3. **Post-run dialogue** — spike finishes, then you ask follow-up questions before calling the verdict.

The third is the sweet spot. It solves the actual problem (verdict uncertainty) without the complexity of mid-run steering.

⚑ Decided: Watch mode (State B in mockup) is Phase 1. Post-run dialogue (State C) is Phase 2. Interactive steering is out of scope.

### The verdict problem

**Sam Okeke** identified a data integrity concern: `findings.md` already contains a `**Verdict:**` line written by the agent. UI verdict buttons would create two sources of truth.

Resolution: the agent proposes a verdict in findings.md. The UI reads it and pre-selects the matching button. User can override — clicking a different verdict calls `PATCH /api/spikes/:slug/verdict` which updates the line in findings.md. If they match, no-op.

⚑ Decided: No new status field. findings.md is the single source of truth for verdict.

### Architecture

Two phases, two separate agent runs:

**Phase 1: Investigation (autonomous)**
- `POST /api/spikes/run` (slug + question + optional reference) → `spawn_agent_run`
- SSE streams to Spike detail pane (monospace log feed, read-only)
- Agent writes findings.md, terminates
- `SpikeRunCompleted { slug }` fires → UI transitions to Phase 2 state

**Phase 2: Q&A + Verdict (interactive)**
- New agent run per question, reads findings.md as context
- User sends follow-up message → `POST /api/spikes/:slug/chat`
- Verdict buttons (ADOPT/ADAPT/REJECT) pre-selected from findings.md
- Override → `PATCH /api/spikes/:slug/verdict`

**Yuki Tanaka** advocated for this separation: "Two-phase, clean. Phase 1 terminates. Phase 2 is fresh. This mirrors the ponder model."

### What it does NOT need

Sam pushed back on scope:
- No session logging (findings.md is the artifact — one file, done)
- No scrapbook
- No team of thought partners
- No status progression beyond implicit (no findings.md = pending, has findings.md = complete)

⚑ Decided: Spike does NOT get the full ponder session machinery. It's focused. The verdict is finite. Keep it that way.

### Mockup produced

Four states captured in spike-session-mockup.html:
- State A: Create Spike modal (question + auto-derived slug + optional reference)
- State B: Running — live SSE feed, phase progress, read-only
- State C: Awaiting verdict — chat with verdict buttons + follow-up input
- State D: Finalized — existing verdict panels + "View investigation session" link

## Open Questions

? Open: Phase 2 chat — per-message spawns (simpler, like advisory) vs. persistent connection (richer, like ponder)? Per-message is safer to start; can upgrade later.

? Open: Create Spike button placement — always in left pane header, or only shown in empty state? Always-visible is more discoverable.

## Product Summary

### What we explored
How to make the Spike page conversational — letting users watch the investigation happen and ask follow-up questions before deciding on a verdict, rather than discovering results after the fact.

### Key shifts
Clarified that "chat while running" is actually three distinct things (watch mode, interactive steering, post-run dialogue) and only the last two matter. Decided on a two-phase architecture: Phase 1 is an autonomous run you can watch; Phase 2 is a Q&A mode that starts after the investigation finishes. Also settled that spikes should NOT get ponder's session machinery — findings.md stays the single artifact and single source of truth for the verdict.

### Implications
The feature is well-scoped and buildable without major structural changes. Three new API endpoints, four new SSE variants, one new modal in the UI. The existing Adopt/Adapt/Reject UI on the detail pane is repurposed as the verdict confirmation controls — no new concept introduced. This directly unblocks running spikes from the UI.

### Still open
1. Phase 2 chat: spawn a fresh agent per user message (simpler) or keep a persistent connection alive (richer)? This is a scope/cost tradeoff decision.
2. Where does the "New Spike" button live in the left pane? Always visible or only in empty state?
