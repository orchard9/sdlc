# Spec: Capture session_id and stop_reason in RunRecord and telemetry events

## Problem

`RunRecord` in `crates/sdlc-server/src/state.rs` tracks cost, turns, and error for each agent run but does not capture two fields that `claude-agent` already provides on `ResultMessage`:

- `session_id` — the Claude conversation session identifier, useful for debugging and for session resume (Claude Code's `--resume` flag)
- `stop_reason` — the reason the agent stream ended (e.g. `"end_turn"`, `"max_turns"`, `"error_max_turns"`, `"tool_use"`)

Without these fields:
- Operators cannot distinguish a run that hit the turn limit (`max_turns`) from one that completed normally (`end_turn`) just by inspecting `RunRecord` — they must parse raw events
- Session IDs are not available for resume workflows
- Telemetry dashboards and the `GET /api/runs` endpoint omit this information

## Solution

Add `session_id: Option<String>` and `stop_reason: Option<String>` to `RunRecord`. Capture them from `Message::Result` inside `spawn_agent_run`. Persist them to the `.sdlc/.runs/*.json` sidecar and emit them in the `RunFinished` SSE event. Surface them in the frontend `RunRecord` TypeScript interface.

## Scope

### In scope

1. **`RunRecord` struct** — add two optional fields:
   - `session_id: Option<String>` — the session ID from the `ResultMessage`
   - `stop_reason: Option<String>` — the stop reason from the `ResultMessage`
   - Both fields use `#[serde(skip_serializing_if = "Option::is_none")]` to keep existing persisted JSON backward-compatible

2. **`spawn_agent_run` in `runs.rs`** — extract `session_id` and `stop_reason` from `Message::Result` and store them alongside `final_cost` and `final_turns`

3. **Completion update block** — write the captured values into the `RunRecord` at the point where `status`, `completed_at`, `cost_usd`, and `turns` are already written

4. **Fallback `RunRecord`** — include `session_id` and `stop_reason` in the fallback record constructed when the run is not found in history

5. **`RunFinished` SSE event** — add `session_id` and `stop_reason` to the emitted JSON payload so the frontend receives them in real-time

6. **Frontend `RunRecord` interface** in `frontend/src/lib/types.ts` — add `session_id?: string` and `stop_reason?: string`

### Out of scope

- Resume workflows using `session_id` (separate feature)
- Displaying `stop_reason` in the UI beyond making it available (a subsequent polish pass can decide on UX)
- Changes to telemetry aggregation (`RunSummary`) — raw events already contain the result message; no schema changes needed there

## Acceptance Criteria

1. `RunRecord` has `session_id` and `stop_reason` fields
2. After a completed agent run, the persisted `.sdlc/.runs/<id>.json` contains non-null `stop_reason` (e.g. `"end_turn"`) and a non-empty `session_id`
3. The `RunFinished` SSE payload includes `session_id` and `stop_reason`
4. `GET /api/runs` response includes `session_id` and `stop_reason` for completed runs
5. Existing run JSON files that lack these fields deserialize correctly (backward compat via `Option` with `#[serde(default)]`)
6. All existing tests pass; new tests cover extraction from `ResultMessage`

## Data Flow

```
claude-agent::query() stream
  └─ Message::Result(ResultMessage::Success { session_id, stop_reason, ... })
       │
       ▼
spawn_agent_run (runs.rs)
  ├─ capture: final_session_id = Some(r.session_id().to_string())
  ├─ capture: final_stop_reason = r.stop_reason()
  │
  ├─ RunRecord { ..., session_id: final_session_id, stop_reason: final_stop_reason }
  │    └─ persist_run() → .sdlc/.runs/<id>.json
  │
  └─ SseMessage::RunFinished { ..., session_id, stop_reason }
       └─ SSE stream → frontend RunRecord update
```

## Backward Compatibility

`ResultMessage::Success` and `ResultMessage::ErrorXxx` types already carry `stop_reason: Option<String>`. The new `RunRecord` fields are `Option<String>` with `#[serde(default)]` so existing `.json` files (which lack these keys) will deserialize with `None` and no migration is needed.
