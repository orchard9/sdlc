# Run Max-Turns Recovery — Design Sketch

## Problem
Investigation/ponder runs that hit `max_turns` are classified as 'failed' in the UI.
The session_id needed to resume via `--resume` is silently discarded after each run.
Users have no recovery affordance.

## Root Causes
1. `is_error()` on `ResultMessage` returns `true` for `ErrorMaxTurns` — same as real errors
2. `RunRecord` stores no `session_id` or `stop_reason`
3. `message_to_event` omits `session_id`/`stop_reason` from result events
4. UI shows 'Run failed' with no differentiation and no Resume action

## Solution

### Immediate relief
- Bump investigation + guideline `max_turns` 100 → 150 (one-line change)

### Data fix (foundation)
- Add `session_id: Option<String>` and `stop_reason: Option<String>` to `RunRecord`
- Capture both from `Message::Result(r)` in `spawn_agent_run`
- Add `stop_reason()` method to `ResultMessage`
- Include both in `message_to_event` result JSON

### UI fix
- `stop_reason === 'max_turns'` → label 'Paused (turn limit)' not 'Failed'
- Show turn count in the run entry

### Resume action
- Resume button in activity panel for `stop_reason === 'max_turns'` runs (investigation + ponder only)
- `POST /api/investigation/:slug/chat` accepts optional `resume_from_run_id`
- Server looks up RunRecord, extracts session_id, passes `opts.resume = Some(session_id)`
- New session N+1 created, agent continues with full conversation history

## Feature Slug
`fix-run-max-turns-recovery`

## Decisions
- ⚑ session_id + stop_reason added to RunRecord (foundation for all retry semantics)  
- ⚑ max_turns bump (100→150) ships as immediate relief, not instead of real fix
- ⚑ Resume scoped to investigation + ponder runs only for now
- ⚑ Resume creates new session (N+1), not overwrite of existing session
