# Code Review: Fix Agent Activity Tile URL

## Changes Summary

5 files changed across backend (Rust) and frontend (TypeScript).

## File-by-File Review

### `crates/sdlc-server/src/state.rs`
- Added `run_type: String` and `target: String` to `SseMessage::RunStarted` variant.
- Clean, matches existing field naming convention. **No issues.**

### `crates/sdlc-server/src/routes/runs.rs`
- Updated the `SseMessage::RunStarted` emit to pass `record.run_type.clone()` and `record.target.clone()`.
- The `record` is already constructed with the correct `run_type` and `target` earlier in `spawn_agent_run`, so these values are accurate. **No issues.**

### `crates/sdlc-server/src/routes/events.rs`
- Destructured the two new fields and included them in the JSON payload.
- Consistent with existing serialization pattern. **No issues.**

### `frontend/src/lib/types.ts`
- Added optional `run_type?: string`, `target?: string`, `session_id?: string`, `stop_reason?: string` to `RunSseEvent`.
- Optional fields ensure backward compatibility. **No issues.**

### `frontend/src/contexts/AgentRunContext.tsx`
- Changed `run_type: 'feature'` to `(event.run_type ?? 'feature') as RunRecord['run_type']`.
- Changed `target: event.key` to `event.target ?? event.key`.
- Fallbacks preserve backward safety for any edge case where the fields might be absent. **No issues.**

## Findings

No issues found. The change is minimal, targeted, and addresses the root cause directly. Cargo clippy and TypeScript type-check both pass cleanly.

## Verdict

**APPROVED** — ready to merge.
