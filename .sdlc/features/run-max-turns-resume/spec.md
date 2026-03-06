# Spec: Paused (turn limit) UX and Resume action for investigation and ponder runs

## Problem

When an investigation or ponder agent run hits its `max_turns` limit, the run
terminates with `ResultMessage::ErrorMaxTurns`. The current code classifies any
`is_error()` result as `status = "failed"` and stores no session_id on the
RunRecord. The UI shows a red X with the `error` field, which looks identical to
a real failure (crash, API error, timeout).

Users have no way to tell the run paused due to turn budget vs. truly failed, and
no way to continue where the agent left off. The `session_id` emitted by every
`ResultMessage` is discarded, so the `QueryOptions.resume` field can never be
populated.

## Goal

1. Distinguish "paused at turn limit" from "failed" at the RunRecord level.
2. Surface a distinct "Paused" status in the RunCard UI with a Resume button.
3. Allow a user to resume a paused ponder or investigation run from the UI,
   continuing the agent from the same session_id it stopped at.

## Out of Scope

- Resume for feature runs (`sdlc-run:*`), milestone UAT, or wave runs. Those are
  covered by their own orchestration loops. Only `ponder:*` and
  `investigation:*` keys are in scope.
- Auto-resume (reconnect without user action).
- Changing `max_turns` values.

## Data Changes

### RunRecord (state.rs)

Add two optional fields:

```rust
pub struct RunRecord {
    // ... existing fields ...
    /// Claude session ID from the final ResultMessage — enables resume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Stop reason from the final ResultMessage (e.g. "end_turn", "max_turns").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}
```

These fields are `skip_serializing_if = "Option::is_none"` for backward
compatibility — existing persisted records deserialize fine with `#[serde(default)]`.

### ResultMessage helpers (claude-agent/types.rs)

Add helpers to extract `session_id` and `stop_reason` from `ResultMessage`:

```rust
impl ResultMessage {
    pub fn session_id_value(&self) -> &str { /* match arms */ }
    pub fn stop_reason(&self) -> Option<&str> { /* match arms */ }
}
```

`stop_reason` is `Option<String>` on both `ResultSuccess` and `ResultError` (already exists).

## Status Changes

### New status: "paused"

In `spawn_agent_run`, the final status determination changes from:

```rust
let status = if is_error { "failed" } else { "completed" };
```

to:

```rust
let status = match (is_error, is_max_turns) {
    (_, true)  => "paused",
    (true, _)  => "failed",
    _          => "completed",
};
```

Where `is_max_turns` is set to `true` when a `Message::Result(ResultMessage::ErrorMaxTurns(_))` is received.

The `session_id` and `stop_reason` fields are always captured from the final
`ResultMessage` (regardless of variant) and stored on the RunRecord.

### RunStatus type (types.ts)

Add `'paused'` to the union:

```ts
export type RunStatus = 'running' | 'completed' | 'failed' | 'stopped' | 'paused'
```

### load_run_history orphan handling

The existing code promotes `running` → `failed` on server restart. Add
`paused` to the "do not promote" check — paused runs are already terminal and
must be preserved.

## Resume Endpoint

### POST /api/ponder/:slug/chat/resume

Body: `{ "run_id": "<run_id>" }` — identifies the paused run to resume from.

Server-side:
1. Look up RunRecord by `run_id` — must have `status == "paused"` and
   `session_id` set, otherwise 400.
2. Build the same prompt as `start_ponder_chat` but set
   `opts.resume = Some(session_id)` in `QueryOptions`.
3. Call `spawn_agent_run` with the same `run_key = "ponder:{slug}"` — the
   existing 409 guard prevents double-starts.
4. Return `{ "status": "started", "session": N }`.

### POST /api/investigation/:slug/chat/resume

Same pattern as ponder resume. Reuses `start_investigation_chat` prompt
construction logic with `opts.resume = Some(session_id)`.

## Frontend Changes

### RunCard status icon

Add a `'paused'` case to `StatusIcon`:

```tsx
case 'paused':
  return <PauseCircle className="w-4 h-4 text-amber-400 shrink-0" />
```

### RunCard resume button

When `run.status === 'paused'` and the run key starts with `ponder:` or
`investigation:`, show a Resume button alongside the expand chevron:

```tsx
{isPaused && isResumable && (
  <button onClick={handleResume} className="...">
    <Play className="w-3.5 h-3.5" />
  </button>
)}
```

`handleResume` calls `POST /api/ponder/:slug/chat/resume` or
`POST /api/investigation/:slug/chat/resume` with `{ run_id: run.id }`.

### RunCard label suffix

When `run.status === 'paused'`, append `" · paused (turn limit)"` to the label
display to give context without requiring the user to expand.

### AgentRunContext

Add `'paused'` to the `activeRuns` exclusion — paused runs are not active.
The existing `isRunning` check uses `status === 'running'`, which is correct
(no change needed there).

## SSE Events

The existing `RunFinished { id, key, status }` SSE message already carries
`status`, so no new SSE variants are needed. The frontend re-fetches run list
on `RunFinished` and will see `status = "paused"` correctly.

## Acceptance Criteria

1. A ponder or investigation run that hits `max_turns` shows `status = "paused"` (not `"failed"`) in `/api/runs`.
2. The RunRecord for such a run has `session_id` and `stop_reason = "max_turns"` populated.
3. A paused run shows an amber pause icon and `" · paused (turn limit)"` label in the RunCard.
4. Clicking Resume on a paused ponder or investigation run starts a new agent session using `opts.resume = session_id`.
5. Existing `"failed"` runs (crash, timeout, API error) are unaffected.
6. Existing persisted RunRecord JSON without `session_id`/`stop_reason` deserializes without error.
