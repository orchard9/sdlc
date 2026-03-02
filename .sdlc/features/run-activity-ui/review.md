# Review: run-activity-ui

## Summary

Implemented the Run Activity Feed — a structured event timeline component that fetches `GET /api/runs/:id/telemetry` and renders the complete event sequence in the run detail view.

## Changes

### Backend

- `crates/sdlc-server/src/state.rs` — added `prompt: Option<String>` field to `RunRecord` (serde skip_serializing_if = None for backward compatibility)
- `crates/sdlc-server/src/routes/runs.rs` — `spawn_agent_run` now stores a truncated prompt preview (first 2000 chars) in the RunRecord; added `get_run_telemetry` handler that returns `{ run_id, prompt, events }` from the stored sidecar
- `crates/sdlc-server/src/lib.rs` — registered `GET /api/runs/{id}/telemetry` route

### Frontend

- `frontend/src/lib/types.ts` — added `prompt?: string | null` to `RunRecord`; added `RawRunEvent`, `RunTelemetry`, and all `Paired*` event types
- `frontend/src/api/client.ts` — added `getRunTelemetry(id)` method
- `frontend/src/hooks/useRunTelemetry.ts` — polling hook: fetches telemetry on mount, polls every 2s while running, stops on completion
- `frontend/src/components/runs/pairEvents.ts` — converts flat event array to structured `PairedEvent[]`; groups tool activity (tool_call → tool_progress → tool_summary) per assistant message
- `frontend/src/components/runs/RunInitCard.tsx` — init event card: model badge, tool count, MCP servers, prompt text
- `frontend/src/components/runs/ToolCallCard.tsx` — tool exchange card with collapsible JSON input and optional summary/timing; blue left border
- `frontend/src/components/runs/AssistantTextBlock.tsx` — assistant prose block
- `frontend/src/components/runs/RunResultCard.tsx` — run result footer with green/red border, cost, turn count
- `frontend/src/components/runs/RunActivityFeed.tsx` — top-level component: fetches telemetry, processes events, renders list; shows loading/error/empty states
- `frontend/src/components/layout/RunCard.tsx` — updated to use `RunActivityFeed` for completed/failed/stopped runs (rich structured view) while keeping the live SSE `AgentLog` for active runs

## Verification

- `SDLC_NO_NPM=1 cargo build --all` — passes (no warnings, no errors)
- `cd frontend && npm run build` — passes (TypeScript strict + vite bundle)
- Visual design follows existing dark UI conventions (border-blue-500 for tools, border-green-500/border-red-500 for result)
- Backward compatible: `prompt` field is optional in both Rust and TypeScript; old RunRecords without it display correctly
