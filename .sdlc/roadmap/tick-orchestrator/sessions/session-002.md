---
session: 2
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Design locked — Action = trigger + tool. recurrence field in struct, re-scheduling in Phase 2. Phase 1 scope tight."
  next: "Commit and build: sdlc-core orchestrator module + redb ActionDb + sdlc orchestrate CLI + integration test with quality-check"
  commit: "Integration test passes: two scheduled quality-check actions fire within a 500ms tick window"
---

## Session 2

Jordan opened with a key insight: the "action" concept needs to be dialed in well,
and the answer is to unify it with the existing tool system. Actions = timing + a tool.

This resolves the biggest open question from Session 1 ("what does an action execute?")
in the cleanest possible way.

### The Insight

Tools already have:
- A well-defined stdin/stdout JSON protocol (`--run` mode)
- `input_schema` (via `--meta`)
- `result_actions` for follow-up behavior
- `version` for auditability

An action is just: *when to run* (orchestrator's job) + *what to run* (tool's job).
The tick loop calls `run_tool()` — the exact same function the UI uses.

⚑ Decided: **Action = { trigger: ActionTrigger, tool_name: String, tool_input: serde_json::Value }**
Rationale: reuses the tool system, forces tools to be well-designed (clean input schemas,
meaningful output, idempotency), and gives the orchestrator a clean execution contract.

### Tool Improvements as a Side Effect

For tools to work as orchestrator payloads they must:
- Have stable, clean input schemas
- Return meaningful `ok`/`error` output (not just side effects)
- Be idempotent (orchestrator may retry on failure)

This pressure improves the tool ecosystem Jordan is building across 100 services.
Tools and orchestrator co-evolve — each one makes the other better.

⚑ Decided: **The `sdlc-next` tool** — `sdlc next --for <slug>` + agent dispatch = a
tool called `sdlc-next`. The orchestrator doesn't need to know anything about sdlc
semantics; it just runs a tool. This keeps the orchestrator generic.

### Recurrence Field

**Priya Nair · Distributed Systems**
Add `recurrence: Option<Duration>` to the Action struct in Phase 1. Struct fields
are cheap. DB migrations are not. Store the field; skip the re-scheduling logic
in v1.

⚑ Decided: recurrence field in struct now, re-scheduling logic in Phase 2.

### Status Result Storage

**Priya Nair · Distributed Systems**
`Completed { result: serde_json::Value }` on ActionStatus — store the tool's
full JSON result in the DB. For 1,000 actions/tick, Jordan needs per-action
observability. This is the audit trail at the action level.

⚑ Decided: include result payload in Completed status variant.

### Webhook → Tool Mapping (Phase 2)

Webhook registration: `{ path, tool_name, input_template }`. Raw payload stored
on ingress. On tick: match → render template → run_tool(). Phase 1 is untouched.

### Phase 1 Scope (Locked)

1. `crates/sdlc-core/src/orchestrator/action.rs` — Action, ActionTrigger, ActionStatus
2. `crates/sdlc-core/src/orchestrator/db.rs` — ActionDb (redb), composite key, range_due(), startup_recovery()
3. `crates/sdlc-cli/src/cmd/orchestrate.rs` — tick loop + `sdlc orchestrate add`
4. Integration test: two quality-check actions 100ms apart, tick rate 500ms, verify both fire

**Dana Cho · Product Skeptic**
Use quality-check in the integration test — it already exists, has no side effects,
proves the loop works. sdlc-next is the real payload but adds agent complexity.
Validate the orchestrator first.

### Open Questions (Resolved in This Session)

- ✓ What does an action execute? → A tool.
- ✓ How does recurrence work? → Field in struct, logic in Phase 2.
- ? Should webhook raw payload become tool_input directly, or via template? → Template (Phase 2 design, not Phase 1).
- ? How does the orchestrator discover available tools? → `run_tool(script, "--meta", ...)` — already exists. Phase 3: management API lists available tools + their schemas.
