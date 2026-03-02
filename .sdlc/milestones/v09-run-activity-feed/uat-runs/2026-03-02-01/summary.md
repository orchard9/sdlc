# UAT Run — Run Activity Feed — complete readable timeline for every agent run
**Date:** 2026-03-02T02:56:44Z
**Verdict:** PassWithTasks
**Tests:** 6/6
**Tasks created:** run-activity-ui#T1

## Results
Suite: Run Activity Feed — v09-run-activity-feed UAT
Duration: 3508ms
Passed: 6 | Failed: 0 | Skipped: 0

## Mode
Mode B — generated spec from checklist (no prior spec existed). Exercised UI via Playwright MCP
against synthetic events sidecar (.sdlc/.runs/20260302-020808-lrr.events.json) that matches the
server's actual event format from message_to_event().

## Verified
- Init card renders with model, tool count, MCP server list
- Tool call cards render with tool names
- Tool call input is expandable via show/hide toggle
- Assistant text blocks render between tool calls
- Result card shows cost ($0.0342) and turn count (4 turns)
- Telemetry API returns correct structure: { run_id, prompt, events }

## Failures
| Test | Classification | Resolution |
|---|---|---|
| Error badge never shows | Code bug | Task run-activity-ui#T1 created |

## Notes
- Prompt display: RunInitCard correctly shows prompt when non-null. Existing runs have null
  prompt in the in-memory run_history cache (loaded at server startup before prompt field existed).
  New runs created via spawn_agent_run will have prompt stored correctly.
- Subagent cards: not tested — no subagent-spawning runs available. Rendering code exists
  (SubagentCard component referenced in pairEvents but PairedEvent type doesn't include subagent
  variant — future work).
