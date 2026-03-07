---
session: 1
timestamp: 2026-03-07T00:00:00Z
orientation:
  current: "Root cause identified — SSE RunStarted event lacks run_type and target fields, frontend hardcodes 'feature' and uses full key as target"
  next: "Commit to feature and implement the 3-file fix"
  commit: "Met — bug is fully diagnosed with a clear, minimal fix"
---

**Xist · Owner**
fix ponder commit reference url

when committing a ponder session, it shows up in the activity feed with a reference url like http://localhost:7777/features/ponder-commit:frontend-port-8881

However then we get this error: Failed to load feature

invalid slug 'ponder-commit:frontend-port-8881': must be lowercase alphanumeric with hyphens

---

## Investigation

Traced the data flow from ponder-commit endpoint through SSE to frontend rendering:

1. **`crates/sdlc-server/src/routes/runs.rs:1557`** — `commit_ponder` creates run key `ponder-commit:{slug}` and calls `spawn_agent_run` with `run_type = "ponder"`.

2. **`crates/sdlc-server/src/routes/runs.rs:464`** — `spawn_agent_run` correctly extracts `target` from key: `key.split(':').next_back()` → `"frontend-port-8881"`.

3. **`crates/sdlc-server/src/routes/runs.rs:739`** — Emits `SseMessage::RunStarted { id, key, label }` — no `run_type` or `target`.

4. **`frontend/src/contexts/AgentRunContext.tsx:66-73`** — On `run_started` SSE event, creates temp RunRecord with **hardcoded** `run_type: 'feature'` and `target: event.key` (the full key including colon).

5. **`frontend/src/lib/routing.ts:7-8`** — `runTargetRoute('feature', 'ponder-commit:frontend-port-8881')` → `/features/ponder-commit:frontend-port-8881`.

6. **`frontend/src/components/layout/RunCard.tsx:139`** — Renders this as a clickable link.

7. **`frontend/src/pages/FeatureDetail.tsx`** — Tries to load feature with slug `ponder-commit:frontend-port-8881`, which fails validation.

## Root Cause

The `RunStarted` SSE event doesn't include `run_type` or `target`. The frontend guesses `'feature'` and uses the full key (with colon) as the target. Both are wrong for ponder-commit runs (and potentially other non-feature run types).

## Fix

Add `run_type` and `target` to `SseMessage::RunStarted` and propagate through SSE JSON to the frontend. Four touch points:

| File | Change |
|------|--------|
| `crates/sdlc-server/src/state.rs` | Add `run_type: String` and `target: String` to `RunStarted` variant |
| `crates/sdlc-server/src/routes/runs.rs` | Pass `record.run_type` and `record.target` when emitting |
| `crates/sdlc-server/src/routes/events.rs` | Include `run_type` and `target` in SSE JSON |
| `frontend/src/contexts/AgentRunContext.tsx` | Use `event.run_type ?? 'feature'` and `event.target ?? event.key` (fallback for old servers) |

Status updated to **converging** — ready to commit.
