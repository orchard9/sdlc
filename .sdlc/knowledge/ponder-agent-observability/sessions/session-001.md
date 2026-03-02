# Knowledge Harvest: ponder/agent-observability

## Status
- **Action**: Updated (entry already existed)
- **Knowledge slug**: `ponder-agent-observability`
- **Source**: ponder workspace `agent-observability`

## What Was Harvested

The ponder workspace captures early user feedback (from Discord) about agent observability needs in the SDLC UI. Key content includes:

- **Agent activity breakdown**: Users want to see what the agent is doing at a granular level — network, CPU, LLM wait, sub-agent wait — not just "running/not running"
- **Quota visibility**: Dollar cost needs translation to meaningful units (% of daily limit, estimated remaining calls)
- **Time-series view**: Activity Monitor-style graph over time, not just a point-in-time snapshot
- **Always-on monitoring**: Background service, not a panel that polls on render
- **Concurrency signal**: Identify where concurrency could be added by showing idle agents

## Notable Content

The workspace is in early "raw signal" stage — waiting on a telemetry spike (`mcp-run-telemetry`) before moving to design. The session orientation notes the next step is to follow spike findings and build the UI on top of whatever telemetry layer is recommended.

Open questions remain around: Claude API telemetry capabilities, storage strategy (in-memory vs persistent), MVP scope, and RunRecord enrichment.

## Harvest Notes

Entry was previously created; this run updated the content to reflect current workspace state.
