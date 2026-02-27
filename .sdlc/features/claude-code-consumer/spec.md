# Feature Spec: SDLC Agent SDK (TypeScript Consumer)

## Purpose

Build a TypeScript package (`packages/sdlc-agent`) that wraps `@anthropic-ai/claude-agent-sdk`
to create an autonomous agent loop that drives the SDLC state machine end-to-end — from
`draft` through `released` — with minimal human intervention.

The SDLC state machine deliberately produces no AI output; it emits structured directives.
This package is the canonical TypeScript consumer of those directives.

---

## User Stories

### As a developer with a new SDLC feature in `draft`
I want to run `sdlc-agent run <slug>` and have the agent:
- Write the spec, verify it, and approve it autonomously
- Continue through design, tasks, qa-plan, implementation, review, audit, and QA without stopping
- Stop only at true HITL gates (`wait_for_approval`, `unblock_dependency`) — human resolves the blocker, then re-invokes the runner

### As a developer running CI
I want `sdlc-agent run-heavy --feature <slug>` to execute only resource-intensive actions
(ImplementTask, RunQa) so I can schedule them on more powerful infrastructure.

### As a developer using the web UI
I want a "Run Agent" button on the FeatureDetail page that invokes the agent SDK and
streams progress back to the browser, so I can watch it work in real time.

### As a developer integrating this programmatically
I want to `import { runFeature } from "sdlc-agent"` and call it from my own orchestration
scripts with hooks for logging, tool interception, and custom gate handling.

---

## Acceptance Criteria

1. `sdlc-agent run <slug>` runs the feature directive loop until a human gate or `done`
2. All SDLC operations (write artifact, approve, reject, add task) go through typed MCP tools
3. Claude receives the full directive `message` as its prompt for each action
4. Shell gates are executed before any `sdlc_approve_artifact` call succeeds
5. Human gates pause execution and print a clear message; resuming re-reads the directive
6. Session context persists across directive iterations for a given feature (stored in `.sdlc/features/<slug>/.agent-session`)
7. The correct specialized agent (spec-writer, designer, implementer, etc.) is selected per ActionType
8. `--model` flag overrides the default model (claude-sonnet-4-6)
9. Opus-level agents (reviewer, auditor) default to `claude-opus-4-6`
10. `run-all` discovers features needing work via `sdlc query ready` and processes them
11. All output (directive, tool calls, results) can be streamed via `onMessage` callback
12. Package works with both CLI mode (`sdlc` subprocess) and REST mode (`--rest <url>`)

---

## Out of Scope

- Building a new model or AI capability — this consumes Claude via the official SDK
- Replacing the SDLC state machine itself — it remains the source of truth
- Replacing human judgment on true HITL gates (`wait_for_approval`, `unblock_dependency`) — these always surface to the developer
- Modifying files outside of `output_path` declared in the directive (enforced by tool)
- Support for non-Claude LLM backends in v1 (design allows it later via MCP tool swap)

---

## System Design Summary

```
sdlc-agent run <slug>
  └─ FeatureRunner loop
       ├─ sdlcClient.getDirective(slug)    → SdlcDirective
       ├─ agentForAction(action)           → AgentDefinition
       ├─ query({ prompt, mcpServers, tools, ... })   ← Claude Agent SDK
       │     Claude uses SDLC MCP tools:
       │       sdlc_get_directive          → reads state
       │       sdlc_write_artifact         → writes .sdlc/.../artifact.md
       │       sdlc_approve_artifact       → runs gates + advances phase
       │       sdlc_add_task               → logs implementation tasks
       └─ loops until action === "done" or human gate
```

### Specialized Agents (per ActionType)

| Action            | Agent          | Model  |
|-------------------|----------------|--------|
| create_spec       | spec-writer    | sonnet |
| create_design     | designer       | sonnet |
| create_tasks      | task-planner   | sonnet |
| create_qa_plan    | task-planner   | sonnet |
| implement_task    | implementer    | sonnet |
| fix_review_issues | implementer    | sonnet |
| create_review     | reviewer       | opus   |
| approve_review    | reviewer       | opus   |
| create_audit      | auditor        | opus   |
| approve_audit     | auditor        | opus   |
| run_qa            | qa-runner      | sonnet |
| approve_spec      | reviewer       | sonnet |
| approve_design    | reviewer       | sonnet |
| approve_tasks     | reviewer       | sonnet |
| approve_qa_plan   | reviewer       | sonnet |
| approve_merge     | qa-runner      | sonnet |

All `approve_*` actions are agentive — the agent reads the artifact, verifies quality, and calls
`sdlc artifact approve` or `sdlc artifact reject` without pausing for human input.

---

## Open Questions

1. **Parallelism**: Should `run-all` process multiple features in parallel by default,
   or serial to avoid interleaved Claude API calls? Start serial; add `--parallel <n>` later.

2. **Server integration**: Phase 5 adds SSE streaming from the server.
   The server would spawn `sdlc-agent` as a subprocess and forward its stdout.
   Is this the right integration, or should the server import sdlc-agent as a library?
   → Prefer library import for tighter integration and easier error handling.

3. **Monorepo tooling**: Use `bun` workspaces or keep standalone?
   → Standalone first; promote to workspace when a second TS package exists.

4. **Gate failures**: If a shell gate fails repeatedly, should the agent try to fix the
   code automatically or halt and surface the error?
   → Halt in v1. Agent can fix in v2 with a `fix_gate_failure` loop.
