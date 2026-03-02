# Architecture: Telemetry Layer

## What Was Built (Spike → Features)

### Spike: mcp-run-telemetry
**Verdict:** ADOPT — redb + stream event extension
**Doc:** `.sdlc/spikes/mcp-run-telemetry/findings.md`

**Key finding:** The Claude Agent SDK stream already emits all needed data (ToolResult, SubagentStarted/Progress/Completed, thinking blocks). It was being discarded. No new dependencies needed — redb was already in the workspace.

### Feature: run-events-api (SHIPPED)
- `TelemetryStore` in `crates/sdlc-server/src/telemetry.rs` — redb-backed, composite key `(run_id, seq)`
- Captures: tool calls, tool results, subagent lifecycle, thinking blocks, run init, run result
- Endpoints: `GET /api/runs/:id/telemetry`, `GET /api/runs/:id/telemetry/summary`
- Storage: `.sdlc/telemetry.redb` (gitignored, persists across restarts)

### Feature: run-activity-ui (SHIPPED)
- Chronological event timeline in the agent panel
- ToolCallCard (with matched tool_result), SubagentCard, AssistantTextBlock, init/result cards
- Live updates via re-fetch on SSE RunFinished event

## Foundation Solid — What's Still Ahead

The original vision (see brief.md) had more:

1. **Quota visibility** — translate $ cost to % of daily API limit, estimated remaining calls. NOT YET built.
2. **Time-series graph** — activity graphed over time per run, not just a feed. NOT YET built.
3. **Concurrency signal** — which agents are idle while others wait. NOT YET built.
4. **Wait-type breakdown** — LLM wait vs sub-agent wait vs tool wait. NOT YET captured in events.

## Architecture Constraints

- No external service (local embedded DB only) ✅
- Persists across restarts ✅
- Binary self-contained (pure Rust) ✅
- DB growth: ~1.5 MB per 13 events — need retention for long-running installs