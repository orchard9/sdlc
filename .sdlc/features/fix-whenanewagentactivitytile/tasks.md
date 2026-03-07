# Tasks: Fix Agent Activity Tile URL

## Task 1: Add run_type and target to SseMessage::RunStarted

Add `run_type: String` and `target: String` fields to the `RunStarted` variant in `crates/sdlc-server/src/state.rs`. Update the emit site in `crates/sdlc-server/src/routes/runs.rs` to pass `record.run_type.clone()` and `record.target.clone()`. Update the serialization in `crates/sdlc-server/src/routes/events.rs` to include both fields in the JSON payload.

## Task 2: Update frontend to consume run_type and target from SSE

Add optional `run_type` and `target` fields to `RunSseEvent` in `frontend/src/lib/types.ts`. Update the `run_started` handler in `frontend/src/contexts/AgentRunContext.tsx` to use `event.run_type` (falling back to `'feature'`) and `event.target` (falling back to `event.key`) when constructing the `RunRecord`.

## Task 3: Build and verify

Run `cargo clippy --all` and `cargo build --all` to ensure Rust changes compile cleanly. Verify frontend builds with `cd frontend && npm run build`.
