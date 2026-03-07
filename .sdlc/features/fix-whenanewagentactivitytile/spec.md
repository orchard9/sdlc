# Spec: Fix Agent Activity Tile URL on Appearance

## Problem

When a new agent activity tile appears in the UI, the URL it links to is wrong. It later corrects itself when the run finishes and a full data refresh occurs, but it must be correct as soon as the tile appears.

## Root Cause

The `SseMessage::RunStarted` event (emitted in `crates/sdlc-server/src/routes/runs.rs:739`) only sends `id`, `key`, and `label`. It does **not** include `run_type` or `target`.

The frontend handler in `frontend/src/contexts/AgentRunContext.tsx:62-76` receives this incomplete event and constructs a `RunRecord` with:
- `run_type: 'feature'` (hardcoded — line 69, comment: "will be corrected on next fetch")
- `target: event.key` (uses the key as the target, which is incorrect for non-feature runs)

`RunCard` then calls `runTargetRoute(run.run_type, run.target)` which produces the wrong route because both `run_type` and `target` are wrong.

The URL only becomes correct when `run_finished` triggers `api.getRuns()` (line 86), which fetches the full `RunRecord` from the server with the correct `run_type` and `target`.

## Fix

### Backend: Add `run_type` and `target` to `RunStarted` SSE event

1. **`crates/sdlc-server/src/state.rs`** — Add `run_type: String` and `target: String` fields to `SseMessage::RunStarted`.

2. **`crates/sdlc-server/src/routes/runs.rs`** — Pass `record.run_type` and `record.target` into the `SseMessage::RunStarted` emit at line 739.

3. **`crates/sdlc-server/src/routes/events.rs`** — Serialize the new `run_type` and `target` fields in the `RunStarted` JSON payload (lines 73-81).

### Frontend: Use the new fields

4. **`frontend/src/lib/types.ts`** — Add `run_type?: string` and `target?: string` to `RunSseEvent`.

5. **`frontend/src/contexts/AgentRunContext.tsx`** — Use `event.run_type` and `event.target` (with existing fallbacks for backward safety) instead of the hardcoded `'feature'` and `event.key`.

## Scope

- 3 backend files (state.rs, runs.rs, events.rs)
- 2 frontend files (types.ts, AgentRunContext.tsx)
- No new dependencies, no migration, no breaking changes

## Verification

After the fix, when any agent run starts (feature, milestone_uat, ponder, etc.), the activity tile should immediately link to the correct page — not require a data refresh on run completion.
