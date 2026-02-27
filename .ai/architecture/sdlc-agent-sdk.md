---
category: architecture
title: SDLC Agent SDK — TypeScript consumer using Claude Agent SDK
learned: 2026-02-26
source: spec
confidence: high
---

# SDLC Agent SDK

A TypeScript package (`packages/sdlc-agent`) that wraps `@anthropic-ai/claude-agent-sdk`
to create an autonomous agent loop that drives the SDLC state machine end-to-end.

## Problem

The SDLC state machine emits structured directives via `sdlc next --for <slug> --json`
but provides no built-in consumer. Each AI platform (Claude Code, Gemini CLI, etc.)
needs its own consumer. We need a canonical TypeScript consumer that:

1. Reads directives programmatically
2. Delegates to Claude (via the Claude Agent SDK) to act on them
3. Writes artifacts to the correct paths
4. Signals completion back to the state machine
5. Handles gates before advancing phases
6. Persists session context across a multi-phase feature run

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  sdlc-agent CLI / API                │
│         sdlc-agent run <slug>  |  runFeature()       │
└───────────────────┬─────────────────────────────────┘
                    │
           ┌────────▼────────┐
           │  FeatureRunner   │   Core loop: directive → act → advance
           └────────┬────────┘
                    │
        ┌───────────┼───────────┐
        │           │           │
   ┌────▼───┐  ┌────▼────┐  ┌──▼────────┐
   │  SDLC  │  │ Claude  │  │   Gate    │
   │  Tools │  │  Agent  │  │  Runner   │
   │ (MCP)  │  │  SDK    │  │           │
   └────┬───┘  └────┬────┘  └──────────┘
        │           │
   sdlc CLI /    query()
   REST API      + agents
```

### Component Responsibilities

**FeatureRunner** — the main loop
- Calls `getDirective(slug)` to read the next action
- Checks `action === "done"` to exit
- Selects agent definition based on `ActionType`
- Calls Claude via `query()` with SDLC tools + appropriate agent
- Handles tool calls that signal completion (`sdlc_approve_artifact`)
- Runs shell gates before advancing
- Loops until done or error

**SDLC Tools (MCP)** — Claude's interface to the state machine
- Registered via `createSdkMcpServer()` as an in-process MCP server
- All writes go through `sdlc_write_artifact` (enforces output_path)
- All advances go through `sdlc_approve_artifact` (runs gates first)

**Gate Runner** — verification before phase advancement
- Reads `gates[]` from directive
- Runs `type: shell` gates as subprocesses
- Pauses for `type: human` gates (interactive prompt or webhook)
- Reports pass/fail back to the agent

**Specialized Agents** — different Claude configs per ActionType
- Each agent has a focused system prompt and restricted tool set
- Prevents e.g. an implementer from accidentally approving its own review

---

## Package Layout

```
packages/sdlc-agent/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts              # Public API: runFeature, runProject, SdlcAgent
│   ├── cli.ts                # CLI entrypoint: sdlc-agent run|run-all
│   ├── runner.ts             # FeatureRunner — core directive loop
│   ├── tools/
│   │   ├── index.ts          # createSdkMcpServer() bundle
│   │   ├── get-directive.ts  # sdlc_get_directive tool
│   │   ├── write-artifact.ts # sdlc_write_artifact tool
│   │   ├── approve.ts        # sdlc_approve_artifact tool
│   │   ├── reject.ts         # sdlc_reject_artifact tool
│   │   ├── add-task.ts       # sdlc_add_task tool
│   │   ├── complete-task.ts  # sdlc_complete_task tool
│   │   └── add-comment.ts    # sdlc_add_comment tool
│   ├── agents/
│   │   ├── index.ts          # agentForAction(action): AgentDefinition
│   │   ├── spec-writer.ts    # CreateSpec
│   │   ├── designer.ts       # CreateDesign
│   │   ├── task-planner.ts   # CreateTasks + CreateQaPlan
│   │   ├── implementer.ts    # ImplementTask (heavy)
│   │   ├── reviewer.ts       # CreateReview + ApproveReview
│   │   ├── qa-runner.ts      # RunQa (heavy)
│   │   └── auditor.ts        # CreateAudit
│   ├── gates.ts              # Gate runner (shell + human)
│   ├── sdlc-client.ts        # Thin wrapper over sdlc CLI / REST API
│   ├── session.ts            # Session persistence across directive iterations
│   └── types.ts              # SdlcDirective, AgentOptions, RunResult types
└── tests/
    ├── runner.test.ts
    └── tools.test.ts
```

---

## Core Types

```typescript
// From sdlc next --json output
type SdlcDirective = {
  feature: string;
  title: string;
  current_phase: string;
  action: ActionType;
  message: string;
  output_path: string | null;
  is_heavy: boolean;
  timeout_minutes: number;
  gates: GateDefinition[];
};

type ActionType =
  | "create_spec" | "approve_spec"
  | "create_design" | "approve_design"
  | "create_tasks" | "create_qa_plan"
  | "implement_task" | "fix_review_issues"
  | "create_review" | "approve_review"
  | "create_audit" | "run_qa"
  | "approve_merge" | "merge"
  | "archive" | "unblock_dependency"
  | "wait_for_approval" | "done";

type GateDefinition = {
  name: string;
  type: "shell" | "human" | "step_back";
  command?: string;          // for shell gates
  auto: boolean;
  max_retries?: number;
};

type AgentOptions = {
  cwd?: string;               // project root (default: process.cwd())
  apiMode?: "cli" | "rest";   // default: cli
  restBaseUrl?: string;       // for apiMode: rest
  model?: string;             // default: claude-sonnet-4-6
  maxTurns?: number;          // default: 30
  persistSessions?: boolean;  // default: true
  hooks?: AgentHooks;
  onDirective?: (d: SdlcDirective) => void;
  onMessage?: (m: SDKMessage) => void;
};

type RunResult = {
  feature: string;
  finalPhase: string;
  actionsCompleted: number;
  stopped: boolean;
  error?: Error;
};
```

---

## SDLC MCP Tools

Claude uses these tools instead of raw shell calls to interact with the state machine.
This keeps all SDLC operations observable, retry-safe, and auditable.

### `sdlc_get_directive`
Returns the current directive for a feature slug.
```typescript
{
  slug: z.string().describe("Feature slug")
}
→ SdlcDirective (JSON)
```

### `sdlc_write_artifact`
Writes content to the artifact path declared in the current directive.
Validates that the path matches `directive.output_path` (guards against path traversal).
```typescript
{
  slug: z.string(),
  artifact_type: z.enum(["spec","design","tasks","qa_plan","review","audit","qa_results"]),
  content: z.string().describe("Full markdown content to write")
}
→ { written: true, path: string }
```

### `sdlc_approve_artifact`
Runs gates then calls `sdlc artifact approve <slug> <type>`.
Returns gate results so Claude can decide whether to retry.
```typescript
{
  slug: z.string(),
  artifact_type: z.string(),
  notes?: z.string().describe("Approval notes / commit message context")
}
→ { approved: boolean, gates: GateResult[] }
```

### `sdlc_reject_artifact`
Rejects an artifact with a reason.
```typescript
{
  slug: z.string(),
  artifact_type: z.string(),
  reason: z.string()
}
→ { rejected: boolean }
```

### `sdlc_add_task`
Adds a task to the feature.
```typescript
{
  slug: z.string(),
  description: z.string(),
  priority?: z.enum(["high","medium","low"])
}
→ { task_id: string }
```

### `sdlc_complete_task`
Marks a task complete.
```typescript
{
  slug: z.string(),
  task_id: z.string(),
  notes?: z.string()
}
→ { completed: boolean }
```

### `sdlc_add_comment`
Adds a comment or question to the feature (shows up in `sdlc query needs-approval`).
```typescript
{
  slug: z.string(),
  body: z.string(),
  flag_type?: z.enum(["blocker","question","note"])
}
→ { comment_id: number }
```

---

## Agent Definitions

Each `ActionType` maps to an `AgentDefinition` (sub-agent config in Claude Agent SDK terms).
The main runner selects the right agent and passes the directive's `message` as the prompt.

### `spec-writer` (CreateSpec)
```typescript
{
  description: "Writes feature specification documents",
  prompt: `You are a senior product engineer writing feature specifications.
  Standard: Steve Jobs bar — right solution over expedient, no known debt shipped.
  Approach: Understand the full context structurally before writing. Plan the spec before drafting.
  You will receive a feature title and context. Write a complete, detailed spec in Markdown.
  The spec must cover: purpose, user stories, acceptance criteria, out-of-scope, open questions.
  Use sdlc_write_artifact to save the spec, then sdlc_approve_artifact when satisfied.`,
  tools: ["Read", "Glob", "sdlc_get_directive", "sdlc_write_artifact", "sdlc_approve_artifact"],
  model: "sonnet"
}
```

### `designer` (CreateDesign)
```typescript
{
  description: "Creates technical design documents",
  prompt: `You are a senior software architect creating technical design documents.
  Standard: Steve Jobs bar — right solution over expedient, no known debt shipped.
  Approach: Read and understand the full codebase structure before designing. Structural decisions first, detail second.
  Read the existing spec, then produce a design covering: architecture, data model, API contracts,
  component breakdown, dependencies, risks. Use sdlc_write_artifact to save.`,
  tools: ["Read", "Glob", "Grep", "sdlc_get_directive", "sdlc_write_artifact", "sdlc_approve_artifact"],
  model: "sonnet"
}
```

### `task-planner` (CreateTasks / CreateQaPlan)
```typescript
{
  description: "Decomposes features into implementation tasks and QA plans",
  prompt: `You are a tech lead breaking down work into granular tasks.
  Standard: Steve Jobs bar — right solution over expedient, no known debt shipped.
  Approach: Understand the full design structurally before decomposing. Tasks should enable the right implementation, not just any implementation.
  Read spec and design, then create a task breakdown (sdlc_add_task for each) and
  write a QA plan covering unit tests, integration tests, edge cases.`,
  tools: ["Read", "sdlc_get_directive", "sdlc_write_artifact", "sdlc_add_task",
          "sdlc_approve_artifact"],
  model: "sonnet"
}
```

### `implementer` (ImplementTask) — heavy
```typescript
{
  description: "Implements a specific task: writes code, tests, documentation",
  prompt: `You are a senior engineer implementing a specific task.
  Standard: Steve Jobs bar — right solution over expedient, no known debt shipped.
  Approach: Read and understand the full codebase structure before writing a line. Plan the implementation structurally, then proceed to detail. If the right solution requires refactoring, do it.
  Read the task description, existing code context, and write high-quality code.
  Write tests alongside the implementation. Use Bash to run tests before marking complete.
  Use sdlc_complete_task when the task passes all tests.`,
  tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep",
          "sdlc_get_directive", "sdlc_complete_task", "sdlc_add_comment"],
  model: "sonnet"
}
```

### `reviewer` (CreateReview)
```typescript
{
  description: "Reviews implementation for correctness, completeness, and quality",
  prompt: `You are a senior engineer conducting a code review.
  Standard: Steve Jobs bar — right solution over expedient, no known debt shipped.
  Approach: Read the full implementation structurally before evaluating any individual file. Hold the review to the standard, not to "it works."
  Review all changed files, check against spec and tasks, look for bugs, security issues,
  missing tests, and tech debt. Write a structured review document.`,
  tools: ["Read", "Glob", "Grep", "Bash",
          "sdlc_get_directive", "sdlc_write_artifact", "sdlc_approve_artifact"],
  model: "opus"
}
```

### `auditor` (CreateAudit)
```typescript
{
  description: "Performs security and compliance audit",
  prompt: `You are a security engineer performing a production-readiness audit.
  Standard: Steve Jobs bar — right solution over expedient, no known debt shipped.
  Approach: Understand the full system architecture before auditing any component. Structural vulnerabilities first, then detail.
  Check for: OWASP vulnerabilities, exposed secrets, improper error handling, missing auth,
  data validation gaps, and compliance issues. Write a detailed audit report.`,
  tools: ["Read", "Glob", "Grep", "sdlc_get_directive", "sdlc_write_artifact",
          "sdlc_approve_artifact"],
  model: "opus"
}
```

### `qa-runner` (RunQa) — heavy
```typescript
{
  description: "Executes the QA plan and documents results",
  prompt: `You are a QA engineer running the QA plan against the implementation.
  Standard: Steve Jobs bar — right solution over expedient, no known debt shipped.
  Approach: Read the full QA plan and understand all scenarios structurally before executing any. A failing scenario is a blocker, not a note.
  Execute each test scenario from the QA plan. Use Bash to run automated tests.
  Document pass/fail results in the qa-results artifact.`,
  tools: ["Read", "Bash", "sdlc_get_directive", "sdlc_write_artifact", "sdlc_approve_artifact"],
  model: "sonnet"
}
```

---

## Core Loop (FeatureRunner)

```typescript
async function runFeature(slug: string, opts: AgentOptions): Promise<RunResult> {
  let actionsCompleted = 0;
  let sessionId: string | undefined;

  while (true) {
    // 1. Read current directive
    const directive = await sdlcClient.getDirective(slug);
    opts.onDirective?.(directive);

    // 2. Check terminal states
    if (directive.action === "done") break;

    // 3. Check true HITL gates — stop and surface to human
    if (directive.action === "wait_for_approval" || directive.action === "unblock_dependency") {
      console.log(`Human gate: ${directive.action} — ${directive.message}`);
      break;
    }

    // All other actions (including all approve_* verification steps) run agentively
    // 4. Select agent definition
    const agentDef = agentForAction(directive.action);

    // 5. Build MCP tools server
    const mcpServer = createSdlcMcpServer(sdlcClient, directive);

    // 6. Invoke Claude via Agent SDK
    for await (const message of query({
      prompt: buildPrompt(directive),
      options: {
        cwd: opts.cwd,
        resume: sessionId,    // carry context across iterations
        model: agentDef.model ?? opts.model ?? "claude-sonnet-4-6",
        maxTurns: opts.maxTurns ?? 30,
        mcpServers: { sdlc: mcpServer },
        allowedTools: agentDef.tools,
        permissionMode: "acceptEdits",
        systemPrompt: agentDef.prompt,
        // Enforce timeout from directive
        abortController: createTimeoutController(directive.timeout_minutes),
      }
    })) {
      opts.onMessage?.(message);

      // Capture session ID for resumption
      if (message.type === "system" && message.subtype === "init") {
        sessionId = message.session_id;
      }
    }

    actionsCompleted++;
  }

  return { feature: slug, finalPhase: "?", actionsCompleted, stopped: false };
}
```

---

## Session Persistence Strategy

Each **feature** gets a persistent session that spans all directive iterations.
This gives Claude context about: what the spec said when implementing, what review issues
were found when fixing them, etc.

- Session ID stored in `.sdlc/features/<slug>/.agent-session`
- Passed as `resume: sessionId` on each iteration
- Session invalidated when feature is archived or released

This prevents Claude from "forgetting" the spec when it later implements tasks.

---

## CLI Interface

```bash
# Run a single feature to completion (or next human gate)
sdlc-agent run <slug> [--model <model>] [--max-turns <n>] [--rest <url>]

# Run all features that are in a ready/active state
sdlc-agent run-all [--parallel <n>] [--only-light]

# Run only heavy actions (for scheduled execution on CI/more powerful hardware)
sdlc-agent run-heavy [--feature <slug>]

# Show what would be acted on (dry run)
sdlc-agent plan [<slug>]
```

---

## Integration with REST API

When `apiMode: "rest"` is specified, the SDLC client uses the server's REST API instead
of spawning subprocess calls. This is the right mode for the web UI's "Run Agent" button.

| CLI command                         | REST equivalent                        |
|-------------------------------------|----------------------------------------|
| `sdlc next --for <slug> --json`     | `GET /api/features/{slug}/next`        |
| `sdlc artifact approve <slug> <t>`  | `POST /api/artifacts/{slug}/{t}/approve` |
| `sdlc artifact reject <slug> <t>`   | `POST /api/artifacts/{slug}/{t}/reject` |
| `sdlc task complete <slug> <id>`    | `POST /api/features/{slug}/tasks/{id}/complete` |

---

## Integration Points in Existing Codebase

### 1. New package directory
```
packages/sdlc-agent/    ← new TypeScript package
```

Add to root `package.json` workspaces if using npm/bun workspaces, or keep standalone.

### 2. Web UI — "Run Agent" button
`frontend/src/pages/FeatureDetail.tsx` already has a "Generate Directive" button
calling `POST /api/run/{slug}`. Extend this:
- `POST /api/run/{slug}` triggers the agent SDK in a background task
- Returns a `run_id` for SSE status streaming
- Frontend polls `GET /api/runs/{run_id}/status` for updates

### 3. Server — new `/api/runs` routes
`crates/sdlc-server/src/routes/runs.rs` currently returns a static directive.
Extend it to spawn the TypeScript agent process and stream output back.

### 4. CLI shortcut
Add `sdlc agent run <slug>` as an alias in the Rust CLI that delegates to `sdlc-agent`.

---

## Implementation Plan

### Phase 1 — Core loop + tools (1–2 days)
- [ ] Scaffold `packages/sdlc-agent/` with TypeScript + bun
- [ ] Implement `sdlc-client.ts` (CLI subprocess wrapper)
- [ ] Implement all 7 MCP tools
- [ ] Implement `FeatureRunner` basic loop (no session persistence yet)
- [ ] Manual test: run `create_spec` action end-to-end

### Phase 2 — Agent definitions (1 day)
- [ ] Implement all 7 agent definitions
- [ ] Wire `agentForAction()` dispatch
- [ ] Test `spec-writer` → `designer` → `task-planner` flow

### Phase 3 — Sessions + gates (1 day)
- [ ] Session persistence (`.agent-session` file per feature)
- [ ] Gate runner for `type: shell` gates
- [ ] Interactive pause for `type: human` gates

### Phase 4 — CLI + integration (1 day)
- [ ] Build `sdlc-agent` CLI (commander or yargs)
- [ ] `run-all` with configurable parallelism
- [ ] Add to `packages/sdlc-agent/bin` in package.json

### Phase 5 — Server integration (2 days)
- [ ] Extend `POST /api/run/{slug}` to spawn agent
- [ ] SSE streaming of agent messages to frontend
- [ ] Frontend run status UI
