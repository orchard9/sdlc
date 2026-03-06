# Architecture Notes

## Two-Phase Design

**Phase 1: Investigation (autonomous)**
- User clicks "Run Spike" with slug + question + optional reference URL
- `POST /api/spikes/run` → `spawn_agent_run` → SSE streams to Spike detail pane
- Agent runs 5-phase workflow, writes `.sdlc/spikes/<slug>/findings.md`
- Agent terminates. `SpikeRunCompleted { slug }` event fires. UI transitions to Phase 2 state.
- Status indicator: pulsing dot → "Awaiting verdict"

**Phase 2: Q&A + Verdict (interactive)**
- Spike detail pane transitions: shows findings summary, verdict suggestion pre-populated
- User can ask follow-up questions → `POST /api/spikes/:slug/chat` → spawn_agent_run (reads findings.md)
- User confirms verdict via Adopt/Adapt/Reject buttons → `PATCH /api/spikes/:slug/verdict`
- Verdict updates the `**Verdict:**` line in findings.md directly

## Data model changes (minimal)

No new status field. Existing implicit states:
- no findings.md = not started / running
- findings.md with verdict = complete (awaiting user confirmation or already confirmed)

New API endpoints:
- `POST /api/spikes/run` — start Phase 1 investigation
- `POST /api/spikes/:slug/chat` — Phase 2 Q&A (reads findings.md, answers question)
- `PATCH /api/spikes/:slug/verdict` — override verdict (updates findings.md)

## UI changes

**Left pane header**: + button → "New Spike" modal (slug + question + optional reference)
**Detail pane — running state**: SSE stream feed (monospace log-style, read-only), "Stop" button
**Detail pane — awaiting verdict**: Chat history (Phase 2), Verdict buttons (ADOPT/ADAPT/REJECT pre-selected from findings.md), follow-up input
**Detail pane — finalized**: Existing verdict-specific panels + "View investigation session" link

## SSE variants needed
- `SpikeRunStarted { slug }`
- `SpikeRunCompleted { slug }`
- `SpikeRunStopped { slug }`
- `SpikeChatCompleted { slug }` (Phase 2 Q&A response done)

## Open questions
1. Phase 2 chat: per-message spawns (simpler) vs. persistent connection (richer)?
2. Create Spike button: always visible in left pane header, or only in empty state?
