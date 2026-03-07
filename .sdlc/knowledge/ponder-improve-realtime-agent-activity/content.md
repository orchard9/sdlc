---
session: 1
timestamp: 2026-03-07T07:10:00Z
orientation:
  current: "Root cause identified — frontend renders active runs with primitive AgentLog instead of rich RunActivityFeed. Data is already streaming. Fix is frontend-only."
  next: "Commit to milestones — this is a small, well-scoped frontend change ready to execute."
  commit: "Problem diagnosed, solution designed, no open blockers — ready to commit."
---

**Xist - Owner**
improve-realtime-agent-activity

when i start a ponder, a new agent appears in the agent activity ui section. there is almost no information available regarding what the agent is actually doing while it's running. after the agent completes, the agent activity ui contains detailed info. we need to get that detailed info to appear while the agent is actually running.

---

## Investigation

Explored the full data flow from backend to frontend:

**Backend (already correct):**
- `spawn_agent_run` in `runs.rs` streams every message through `message_to_event()` which produces full structured JSON with timestamps, tool_use_ids, subagent fields
- Events are broadcast in real-time via per-run SSE channel (512 slots)
- Events are also persisted to telemetry store via `append_raw()` during execution
- The `/api/run/{key}/events` endpoint streams these as `event: agent` SSE events

**Frontend (the problem):**
- `RunCard.tsx:164` forks rendering: active runs get `AgentLog`, completed runs get `RunActivityFeed`
- `AgentLog` is a primitive monospace log using `AgentEventLine` — renders tool names as yellow arrows, text as plain strings, no structure
- `RunActivityFeed` renders rich paired event cards: `RunInitCard`, `ToolCallCard`, `AssistantTextBlock`, `RunResultCard`, plus `ActivityTimeSeries` chart
- The SSE data is cast as `AgentEvent` (narrow type) instead of `RawRunEvent` (full type), losing timestamps and subagent fields

**Key insight:** `RunActivityFeed` already accepts an `events` prop and has an `isRunning` mode with a running spinner. It was designed to work during runs — it's just never called during one.

## Discussion

**Aria Chen** identified the type mismatch: SSE data IS `RawRunEvent` but typed as `AgentEvent`, dropping fields the rich renderer needs.

**Tobias Krenn** confirmed the fix is minimal: change the type, swap the component, done. No backend changes.

**Ben Hartley** flagged the spawning-state UX gap: `RunActivityFeed` shows "No activity recorded yet" when events are empty, while `AgentLog` shows "Spawning agent..." — need to handle that.

**Dan Reeves** noted `AgentLog` and `AgentEventLine` become dead code after the swap — only used by RunCard's active-run path.

## Decisions

- **Frontend-only fix** — no backend changes needed, data already streams correctly
- **Swap `AgentLog` for `RunActivityFeed`** in active run rendering
- **Fix type**: `AgentEvent[]` → `RawRunEvent[]` for live events
- **Add auto-scroll** to `RunActivityFeed` when `isRunning`
- **Handle spawning state** in `RunActivityFeed` (spinner when running + empty)
- **Delete `AgentLog.tsx` and `AgentEventLine.tsx`** as dead code
- **Consider unifying** `AgentEvent` and `RawRunEvent` types (AgentEvent becomes unused)

## Open Questions

- Should `ActivityTimeSeries` render during active runs? Likely yes — partial data is fine
- Unify `AgentEvent`/`RawRunEvent` types? Likely yes as cleanup
