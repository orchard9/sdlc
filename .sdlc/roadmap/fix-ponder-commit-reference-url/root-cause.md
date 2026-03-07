## Root Cause Analysis

**Bug:** When a ponder-commit run starts, the activity feed (FAB agent panel) shows a link to `/features/ponder-commit:frontend-port-8881` instead of `/ponder/frontend-port-8881`.

**Root cause:** `frontend/src/contexts/AgentRunContext.tsx:66-73`

When a `run_started` SSE event arrives, the frontend creates a temporary `RunRecord` with:
- `run_type: "feature"` — **hardcoded** (should be the actual run type)
- `target: event.key` — uses the **full key** (`ponder-commit:frontend-port-8881`) instead of extracting the target

The `RunStarted` SSE event (`crates/sdlc-server/src/state.rs:215-219`) only carries `id`, `key`, and `label` — it does NOT include `run_type` or `target`.

The comment says "will be corrected on next fetch" — but the broken link is visible and clickable before the fetch completes (and for failed runs, may never be corrected).

**The fix (3 files):**

1. `crates/sdlc-server/src/state.rs` — Add `run_type` and `target` to `SseMessage::RunStarted`
2. `crates/sdlc-server/src/routes/events.rs` — Include `run_type` and `target` in the JSON
3. `crates/sdlc-server/src/routes/runs.rs` — Pass `run_type` and `target` when emitting `RunStarted`
4. `frontend/src/contexts/AgentRunContext.tsx` — Use `event.run_type` and `event.target` instead of hardcoded values
5. `frontend/src/lib/types.ts` — Add `run_type` and `target` to `RunSseEvent` type (if needed)

Decided: The SSE event should carry the same data the RunRecord has. No need for client-side key parsing.