# QA Plan: Fix Agent Activity Tile URL

## Test 1: Cargo build + clippy

Run `SDLC_NO_NPM=1 cargo build --all` and `cargo clippy --all -- -D warnings`. Verify no errors or warnings from the changed files.

## Test 2: Frontend build

Run `cd frontend && npm run build`. Verify no TypeScript errors.

## Test 3: SSE payload includes run_type and target

Inspect `events.rs` to confirm the `run_started` JSON payload now includes `run_type` and `target` fields alongside the existing `id`, `key`, `label`.

## Test 4: Frontend uses SSE-provided values

Inspect `AgentRunContext.tsx` to confirm the `run_started` handler uses `event.run_type` and `event.target` from the SSE event rather than hardcoding `'feature'` and `event.key`.

## Test 5: Backward compatibility

Confirm `RunSseEvent.run_type` and `RunSseEvent.target` are optional (`?`) in `types.ts` so older events without these fields don't break.
