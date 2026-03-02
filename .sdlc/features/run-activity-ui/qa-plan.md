# QA Plan: run-activity-ui

## Scope

Verify the Run Activity Feed renders correctly for completed runs, live runs, and empty states.

## Test cases

1. Expand a completed run in the Agent Activity panel — RunActivityFeed renders with init card, tool exchange cards, assistant text, and run result card.
2. Expand a running run — live AgentLog renders with streaming events.
3. Expand a run with no events — shows "No activity recorded yet" message.
4. Tool input JSON is hidden by default; clicking "show input" expands it.
5. Run result card shows green border for success and red border for failure.
6. RunRecord.prompt is displayed in the init card when present.
7. Frontend TypeScript builds without errors (`npm run build`).
8. Rust backend builds without errors (`SDLC_NO_NPM=1 cargo build --all`).
9. `/api/runs/:id/telemetry` endpoint returns `{ run_id, prompt, events }`.

## Verification

All items verified during implementation. Rust and frontend builds pass.
