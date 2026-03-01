# SDLC Tool System

**Status:** Authoritative spec
**Supersedes:** `sdlc-tools-suite.md`, `sdlc-tools-suite-technical.md`, `sdlc-tools-suite-plan.md`

---

## Overview

Tools are how agents and humans interact with the world outside the state machine. The SDLC state machine manages what to build and when. Tools are how you actually do it: run the build, query the database, check service health, pull logs, deploy, notify.

A tool is a TypeScript script (`tool.ts`) that speaks a three-mode JSON protocol. The SDLC server discovers, runs, and streams tools. The UI renders them. Agents call them via MCP. The result can feed back into the SDLC ecosystem — creating features, tasks, ponder entries, or investigations.

Tools are not throwaway scripts. They are maintained, versioned artifacts. They can evolve. And every interaction they produce is remembered.

---

## What Tools Enable

Ten diverse tools a real project needs — and what each requires:

| Tool | Purpose | Secrets | Input form | Streaming | Result actions |
|------|---------|---------|-----------|-----------|----------------|
| **build** | Run the build system | — | optional scope | yes | "Fix build errors" |
| **logs** | Pull recent logs from cloud provider | API key | service, time range | yes | "Investigate errors" |
| **deploy** | Push to staging or production | API token | environment, git ref | yes | "Roll back" (confirm) |
| **db-query** | Run a read-only SQL query | DATABASE_URL | SQL code editor | no | "Create task from result" |
| **api-health** | Check endpoint status and schema drift | — | URL, headers | no | "File a bug" if failing |
| **notify** | Post to Slack / Discord / GitHub | webhook URL | channel, message | no | — |
| **metrics** | Pull dashboards from Datadog / Grafana | API key | metric, date range | no | "Add to feedback" |
| **search-codebase** | Semantic search over the repo | — | query text | no | "Open AMA thread" |
| **feature-flags** | Read and toggle feature flags | SDK key | flag name, env | no | "Create removal task" |
| **browser-test** | Run Playwright/Puppeteer suite | — | test filter | yes | "Fix failing test" |

The patterns that emerge from this list define the architecture:
- **Secrets are the norm**, not the exception
- **Streaming is the common case** for anything with execution time
- **The result is never the end** — it's the beginning of the next action
- **Input forms are tool-specific** — a SQL editor and a text field are different things

---

## Protocol

Every tool is a TypeScript file at `.sdlc/tools/<name>/tool.ts`. It supports three invocation modes:

```
--meta    No stdin.  Stdout: ToolMeta JSON.
--run     Stdin: JSON input.  Stdout: ToolResult JSON, or NDJSON event stream if streaming.
--setup   No stdin.  Stdout: ToolResult JSON. Optional one-time initialization.
```

The runtime priority is bun > deno > node (via npx tsx). Tools may use any npm/bun packages.

The environment variable `SDLC_ROOT` is always set to the project root. Injected secrets arrive as environment variables (see §Secrets).

### Non-streaming `--run`

Single JSON object on stdout:

```json
{ "ok": true, "data": { ... }, "duration_ms": 420 }
{ "ok": false, "error": "connection refused", "duration_ms": 12 }
```

### Streaming `--run`

When `meta.streaming === true`, emit newline-delimited JSON events, one per line, ending with a `result` event:

```jsonl
{"type": "progress", "message": "Connecting to Render…", "percent": 0}
{"type": "log", "level": "stdout", "line": "Deploy triggered for service web-prod"}
{"type": "log", "level": "stdout", "line": "Build: Installing dependencies (1/4)"}
{"type": "log", "level": "stderr", "line": "Warning: deprecated dependency detected"}
{"type": "progress", "message": "Running health checks", "percent": 90}
{"type": "attachment", "kind": "screenshot", "name": "failure.png", "base64": "…"}
{"type": "result", "ok": true, "data": {"deploy_id": "dep_abc", "url": "https://…"}, "duration_ms": 34200}
```

Event types: `progress`, `log`, `attachment`, `result`. The server pipes these through SSE to the frontend in real time.

---

## ToolMeta Schema

```typescript
interface ToolMeta {
  // Identity
  name: string               // slug: lowercase, hyphens, digits
  display_name: string       // human label: "Deploy"
  description: string        // one sentence: what it does
  version: string            // semver: "1.2.0"

  // Setup
  requires_setup: boolean
  setup_done?: boolean       // injected by server after checking setup state
  setup_description?: string // shown in setup banner

  // Schemas (JSON Schema objects)
  input_schema: JSONSchema
  output_schema: JSONSchema

  // --- New fields (all optional, backward compatible) ---

  // Secrets this tool needs injected as env vars at run time
  secrets?: SecretRef[]

  // How to render the input form. Absent → JSON textarea fallback.
  form_layout?: FormField[]

  // If true, --run emits NDJSON event stream instead of single JSON blob
  streaming?: boolean

  // Follow-up actions available on the tool result
  result_actions?: ResultAction[]

  // Max execution time in seconds (default: 30; streaming tools: 300)
  timeout_seconds?: number

  // Categorization for UI grouping
  tags?: string[]            // "build" | "observability" | "data" | "comms" | "test" | "deploy"

  // Whether interactions are persisted (default: true)
  persist_interactions?: boolean

  // Whether this tool supports multi-turn threads (like AMA)
  threaded?: boolean
}
```

### SecretRef

```typescript
interface SecretRef {
  env_var: string        // "DATABASE_URL" — injected into subprocess env
  description: string    // "PostgreSQL connection string for the project DB"
  required: boolean      // if true and missing, run returns 422 with missing_secrets[]
}
```

### FormField

```typescript
type FormFieldType =
  | 'text'        // single-line input
  | 'textarea'    // multi-line text
  | 'code'        // syntax-highlighted editor
  | 'select'      // dropdown
  | 'checkbox'    // boolean toggle
  | 'date_range'  // start + end date picker
  | 'file'        // file path or upload

interface FormField {
  key: string              // matches input_schema property key
  type: FormFieldType
  label?: string
  placeholder?: string
  options?: string[]       // for select
  language?: string        // for code: 'sql' | 'json' | 'bash' | 'typescript'
  default?: unknown
  secret?: boolean         // render as password field; value never stored in interactions
}
```

### ResultAction

```typescript
interface ResultAction {
  label: string            // "Fix Issues"
  icon?: string            // lucide icon name: "wrench" | "bug" | "rocket" | "undo" | "plus"
  // JSONPath expression evaluated against the ToolResult — action shown only when true.
  // Examples: "$.ok == false"  |  "$.data.failed > 0"  |  "$.data.deploy_id != null"
  condition?: string
  // Agent prompt. Interpolation: {{result}} {{input}} {{tool}} {{project}}
  prompt_template: string
  // If present, show a confirmation dialog with this message before launching agent.
  // Interpolation: {{input.environment}}  etc.
  confirm?: string
}
```

**Example — deploy tool meta:**

```typescript
const meta: ToolMeta = {
  name: "deploy",
  display_name: "Deploy",
  description: "Push the current branch to staging or production via Render",
  version: "1.0.0",
  requires_setup: false,
  secrets: [
    { env_var: "RENDER_API_KEY", description: "Render API key", required: true },
    { env_var: "RENDER_SERVICE_ID", description: "Service ID to deploy", required: true },
  ],
  form_layout: [
    { key: "environment", type: "select", label: "Environment",
      options: ["staging", "production"] },
    { key: "ref", type: "text", label: "Git ref", placeholder: "main" },
  ],
  streaming: true,
  timeout_seconds: 600,
  tags: ["deploy"],
  result_actions: [
    {
      label: "Roll Back",
      icon: "undo",
      condition: "$.ok == true",
      confirm: "Roll back the most recent deploy to {{input.environment}}?",
      prompt_template: "Roll back the last deploy of the {{input.environment}} service. Deploy details: {{result.data | json}}. Use RENDER_API_KEY from the environment.",
    }
  ],
  input_schema: {
    type: "object",
    properties: {
      environment: { type: "string", enum: ["staging", "production"] },
      ref: { type: "string", default: "main" },
    },
    required: ["environment"],
  },
  output_schema: {
    type: "object",
    properties: {
      deploy_id: { type: "string" },
      url: { type: "string" },
      status: { type: "string" },
    },
  },
}
```

---

## Secrets

### Declaration and injection

A tool declares the secrets it needs in `meta.secrets`. The server:

1. Reads `--meta` before every `--run`
2. Checks which declared secrets exist in the SDLC secrets store
3. If any `required: true` secret is missing → returns `422 { missing_secrets: ["DATABASE_URL"] }` without spawning the subprocess
4. Decrypts available secrets and injects them as environment variables into the subprocess
5. Secrets never touch disk, never appear in logs, never leave the subprocess lifetime

### UI behavior

If a tool's required secrets are missing, the run panel shows an inline banner:

> **Secrets required:** `RENDER_API_KEY`, `RENDER_SERVICE_ID` — [Configure in Secrets →]

The Run button is not hidden (agents need to know what's missing). The 422 response carries `missing_secrets` so agent prompts can surface the gap.

---

## Persistence

### Where interactions live

```
.sdlc/
  tools/
    <name>/
      tool.ts
      config.yaml
      README.md
  tool-interactions/        # gitignored by default
    <name>/
      <id>.yaml             # interaction record (run or thread)
      <id>.log              # raw streaming log (for streaming tools)
```

`.sdlc/tool-interactions/` is listed in `.sdlc/.gitignore`. It is local operational history — not part of the project's audit trail in git. Users who want to commit it can remove the gitignore entry.

### Interaction record

Every tool invocation creates an interaction record:

```yaml
id: 20260228-143022-abc
tool_name: deploy
kind: run                  # "run" | "thread"
created_at: "2026-02-28T14:30:22Z"
completed_at: "2026-02-28T14:35:44Z"
input:
  environment: production
  ref: main
result:
  ok: true
  data:
    deploy_id: dep_xyz123
    url: https://myapp.onrender.com
  duration_ms: 322000
streaming_log: "20260228-143022-abc.log"  # present if streaming tool
tags: []
notes: null
```

For streaming tools, the full log is stored separately as `<id>.log` (raw NDJSON lines). The record stores only the final result.

### Retention

Default: keep the last 200 interactions per tool. Older records are pruned on each new run. Configurable in `.sdlc/config.yaml`:

```yaml
tools:
  interaction_retention: 200
```

### API

```
GET  /api/tools/:name/interactions          # list recent records (default 50)
GET  /api/tools/:name/interactions/:id      # get record + streaming log if present
DELETE /api/tools/:name/interactions/:id    # delete a record
```

---

## AMA: Threaded Conversations

AMA is a special case. It is not a single-shot tool — it is a multi-turn conversation with the codebase. Its interactions are **threads**, not runs.

### Thread model

```yaml
id: 20260228-100000-xyz
tool_name: ama
kind: thread
title: "How does authentication work?"    # auto-generated or user-set
created_at: "2026-02-28T10:00:00Z"
updated_at: "2026-02-28T10:45:00Z"
tags: []
committed_to: null                        # or "feature:my-slug" | "ponder:my-slug"
turns:
  - id: turn-001
    question: "How does the auth middleware work?"
    sources:
      - path: crates/sdlc-server/src/auth.rs
        lines: [1, 80]
        excerpt: "pub async fn auth_middleware…"
        score: 0.94
      - path: crates/sdlc-cli/src/cmd/tunnel.rs
        lines: [12, 45]
        excerpt: "fn generate_token…"
        score: 0.81
    synthesis: "The auth middleware checks for a bearer token or cookie…"
    run_id: "20260228-100012-abc"
    created_at: "2026-02-28T10:00:12Z"
  - id: turn-002
    question: "What happens when the token expires?"
    sources: [...]
    synthesis: "Token validation happens on every request…"
    run_id: "20260228-100234-def"
    created_at: "2026-02-28T10:02:34Z"
```

### Thread API

```
GET    /api/tools/ama/threads               # list threads (newest first)
POST   /api/tools/ama/threads               # create a new thread
GET    /api/tools/ama/threads/:id           # get thread with all turns
PATCH  /api/tools/ama/threads/:id           # update title / tags
DELETE /api/tools/ama/threads/:id           # delete thread
POST   /api/tools/ama/threads/:id/commit    # convert to feature / ponder / task
```

The existing `POST /api/tools/ama/answer` continues to work. When called with a `thread_id`, it appends the turn to the existing thread. When called without one, it creates a new thread and returns `thread_id` in the response.

### AMA UI changes

The AMA panel becomes a two-pane experience:
- **Left**: list of past threads, searchable, with titles and timestamps
- **Right**: the active thread — all turns displayed in sequence, new question input at the bottom

A thread can be given a title at any point (manually or auto-generated from the first question).

---

## Integration with the SDLC Ecosystem

Tools are not isolated. Their results feed back into the state machine and the ideation workspaces.

### From tool result → SDLC artifact

Every tool's result panel exposes an **"Add to…"** menu alongside result actions:

| Destination | When to use | What happens |
|-------------|-------------|--------------|
| **New feature** | Tool reveals work that needs to be done | Creates a feature in DRAFT phase; the result (or selected text) becomes the spec draft |
| **New task on current feature** | Tool reveals a concrete fix | Creates a `[tool-result]` task on the active feature, linked to the interaction |
| **New ponder entry** | Tool reveals a question or direction worth exploring | Creates a ponder entry; the result is captured as an artifact |
| **New investigation** | Tool reveals an error or anomaly to dig into | Creates a root-cause or evolve investigation entry |
| **Feedback** | Tool reveals user-visible behaviour | Adds a feedback note |

The agent runs that implement these transitions read the full interaction record — they have the original input, result, and any streaming log — and synthesize appropriate content for the target artifact.

### AMA → Feature (the key flow)

An AMA thread that goes deep on "why is auth slow" eventually reaches a natural conclusion: there's something to build. The **"Create feature from thread"** action:

1. Shows a modal: title (pre-filled from thread title), description (pre-filled from first question)
2. Spawns an agent that reads all turns + syntheses and writes a `spec.md` draft
3. Creates the feature in DRAFT phase with that spec attached
4. Sets `thread.committed_to = "feature:<slug>"`
5. The feature's spec cites the AMA thread in a comment: "Originated from AMA thread 'How does authentication work?'"

The thread is not deleted. It lives as source material.

### Build failure → Task

A failed build result's result action "Create task" does not just create a generic task. The agent:
- Parses the error output (streaming log)
- Identifies the failure type (compile error, test failure, lint)
- Writes a concise task title and description with the error message embedded
- Attaches the task to the current active feature (or lets the user choose)

### Deploy → Milestone advance

A successful production deploy can optionally trigger milestone UAT. The deploy tool's result action "Start UAT" calls `POST /api/milestone/:slug/uat`. The deploy interaction record is attached to the UAT run for traceability.

### Logs → Investigation

The logs tool's result action "Investigate errors" creates a root-cause investigation. The agent primes the investigation context with the filtered error lines from the streaming log.

---

## Result Action Engine

Result actions are generic — no new Rust endpoints per tool.

### Endpoint

```
POST /api/tools/:name/act
{
  "action_index": 0,
  "interaction_id": "20260228-143022-abc",  // optional — for context
  "result": { ... },
  "input": { ... }
}
```

### Server behavior

1. Calls `--meta` on the named tool
2. Finds `result_actions[action_index]`
3. If `condition` is set, evaluates it against `result` — returns 400 if condition is false
4. Interpolates `prompt_template` with `{{result}}`, `{{input}}`, `{{tool}}`, `{{project}}`
5. Calls `spawn_agent_run(key="tool-act:<name>:<action_index>", ...)`
6. Returns `{ run_key, run_id }` — caller subscribes to the run stream

### Interpolation variables

| Variable | Value |
|----------|-------|
| `{{result}}` | Full ToolResult JSON as a string |
| `{{result.data}}` | The `data` field of the result |
| `{{result.data.X}}` | Any nested path in data |
| `{{input}}` | The input JSON as a string |
| `{{input.X}}` | Any nested path in input |
| `{{tool}}` | Tool name |
| `{{project}}` | Project name from config |

### Condition evaluation

Conditions are simple JSONPath-style expressions evaluated against the full ToolResult:

```
"$.ok == false"           → true when run failed
"$.data.failed > 0"       → true when checks have failures
"$.data.deploy_id != null"→ true when deploy ID is present
```

---

## Built-in Tools

Two tools are installed by `sdlc init` / `sdlc update` and are always overwritten on re-init. They cannot be durably edited. To customize one, clone it.

### `ama` — Ask Me Anything

Builds a searchable vector index of the codebase. Answers questions with source citations. Supports multi-turn threads.

- `requires_setup: true` (builds the index)
- `threaded: true`
- `streaming: false` (search + synthesis is fast)
- Result actions: "Create feature from thread", "Open as ponder"

### `quality-check` — Dev Quality Check

Runs the project's shell gates defined in `.sdlc/config.yaml`.

- `requires_setup: false`
- `streaming: false`
- Result actions: "Fix Issues" (agent-driven fix), "Reconfigure" (detect stack and rewrite gates)

---

## Tool Lifecycle

### Create

**Plan-Act pattern** (via UI or `/sdlc-tool-build`):
1. `POST /api/tools/plan` — agent designs schemas and approach, streams plan text
2. User reviews and optionally adjusts
3. `POST /api/tools/build` — agent scaffolds `tool.ts`, `README.md`, tests, commits

### Clone

Built-in tools can be cloned to a user-owned name that survives re-init:

```
POST /api/tools/:name/clone  { "new_name": "my-ama" }
```

The cloned tool is a flat directory copy. Re-init never touches it.

### Evolve

User-created tools can be evolved via agent:

```
POST /api/tools/:name/evolve  { "change_request": "Add a --dry-run flag…" }
```

The agent reads `tool.ts`, applies the change, bumps the version, tests both modes, updates README, runs `sdlc tool sync`, and commits.

### Sync

`sdlc tool sync` regenerates `tools.md` — the canonical human + agent reference for all installed tools. Called automatically by build and evolve agents.

### Built-in detection

`sdlc_core::tool_runner::is_managed_tool(name)` returns true for `ama` and `quality-check`. The `built_in` field is injected into every `--meta` response by the server. The UI shows Clone (not Evolve) for built-in tools.

---

## Directory Structure

```
.sdlc/
  tools/
    _shared/               # shared TypeScript utilities imported by tools
      utils.ts             # common helpers (JSON output, error formatting)
      secrets.ts           # secret access helpers
    ama/
      tool.ts
      config.yaml          # index config (extensions, ignore patterns)
      README.md
    quality-check/
      tool.ts
      README.md
    <user-tool>/
      tool.ts
      README.md
  tool-interactions/       # gitignored
    ama/
      <id>.yaml            # thread records
    quality-check/
      <id>.yaml            # run records
    <user-tool>/
      <id>.yaml
      <id>.log             # streaming log (if applicable)
  .gitignore               # includes tool-interactions/
```

---

## Server Route Summary

```
# Discovery
GET    /api/tools                          # list all tools with meta + built_in
POST   /api/tools                          # scaffold new tool
GET    /api/tools/:name                    # single tool meta

# Invocation
POST   /api/tools/:name/run               # run tool (sync or streaming)
POST   /api/tools/:name/setup             # run setup mode
POST   /api/tools/:name/act               # trigger result action (agent run)

# Lifecycle
POST   /api/tools/:name/clone             # clone to new name
POST   /api/tools/:name/evolve            # evolve via agent
POST   /api/tools/plan                    # Plan phase (create workflow)
POST   /api/tools/build                   # Build phase (create workflow)

# Interaction history
GET    /api/tools/:name/interactions       # list recent interactions
GET    /api/tools/:name/interactions/:id  # get record + log
DELETE /api/tools/:name/interactions/:id  # delete record

# AMA threads
GET    /api/tools/ama/threads             # list threads
POST   /api/tools/ama/threads             # create thread
GET    /api/tools/ama/threads/:id         # get thread + turns
PATCH  /api/tools/ama/threads/:id         # update title / tags / committed_to
DELETE /api/tools/ama/threads/:id         # delete thread
POST   /api/tools/ama/threads/:id/commit  # convert to feature / ponder / task
```

---

## Frontend: ToolsPage

### Layout

Three-pane when a tool is selected:
1. **Left pane** — tool list (grouped by tag), new tool button
2. **Center pane** — tool run panel: form (rendered from `form_layout`), run button, secrets banner
3. **Right pane** — result panel: structured output or streaming log, result action buttons

On mobile: single pane with back navigation.

### Form rendering

If `form_layout` is present in meta, render declared fields:
- `text` → `<input type="text">`
- `textarea` → `<textarea>`
- `code` → syntax-highlighted editor (CodeMirror or Monaco lite)
- `select` → `<select>` with options
- `checkbox` → toggle
- `date_range` → dual date input
- `secret` → `<input type="password">`, value excluded from persisted interactions

If absent → JSON textarea (current behavior, full backward compat).

### Streaming log viewer

For streaming tools, replace the result panel with a live log viewer:
- Lines colored by level: `stdout` (default), `stderr` (amber), progress (muted)
- `attachment` events render inline previews (screenshots, etc.)
- Auto-scroll with a "pause" toggle
- Full log downloadable as `.log`

### Secrets banner

If `--meta` returns `missing_secrets`, show above the form:

> **Secrets required** — `RENDER_API_KEY`, `RENDER_SERVICE_ID` must be configured before this tool can run. [Configure →]

### AMA panel

Thread list in the left pane (replaces single-question UI). Right pane shows the full thread. New question appended at the bottom. Thread title editable inline. "Create feature" / "Send to ponder" buttons in the thread header.

### History panel

Per-tool history tab: timeline of recent interactions, newest first. Each row shows time, input summary, result status, duration. Click to expand full result. Delete button. "Open" button for threads.

---

## Implementation Order

The following are independent and can be parallelized where noted.

**Phase 1 — Foundation** (blocking everything):
1. Interaction persistence layer (`sdlc-core`: `tool_interaction.rs` — load/save, retention)
2. Enhanced `ToolMeta` deserialization (all new optional fields, backward compat)
3. Secret injection in `run_tool` handler

**Phase 2 — Protocol** (after Phase 1):
4. Streaming `--run` mode: server reads NDJSON, pipes through SSE
5. Result action engine: `POST /api/tools/:name/act`, prompt interpolation, agent spawn
6. Interaction recording: write record on every `--run` completion

**Phase 3 — AMA threads** (after Phase 1, parallel with Phase 2):
7. AMA thread persistence (thread + turn records)
8. Thread API (`/api/tools/ama/threads/*`)
9. AMA → Feature/Ponder commit flow

**Phase 4 — Frontend** (after Phase 2 + 3):
10. Form renderer (schema-driven fields)
11. Streaming log viewer
12. Result action buttons (condition evaluation client-side)
13. AMA thread panel (two-pane, thread list + active thread)
14. Tool history tab

**Phase 5 — Tool expansion**:
15. Build, deploy, logs tools (requires secrets injection from Phase 1)
16. db-query tool (requires code form field from Phase 4)
17. Metrics, notify, feature-flags tools

---

## Design Principles

**Tools are the project's interface with the world.** They are not developer conveniences — they are how the team's agents interact with real infrastructure. A deploy tool that requires manually setting env vars is not a deploy tool; it's a suggestion.

**Every result is the beginning, not the end.** The most valuable moment in a tool interaction is when the result reveals what to do next. Result actions close that loop. An architecture that makes closing the loop require a new server endpoint per tool is an architecture that will always be behind.

**Persistence is memory.** An AMA thread that gets lost on page refresh is a conversation that never happened. Interactions are the project's operational history. They should be as queryable and durable as the feature state.

**The built-in vs. user distinction matters.** Built-in tools are infrastructure — always current, always correct. User tools are owned — customizable, evolvable, durable. The system must make the distinction clear and the transition (clone) frictionless.

**The form is the contract.** A tool that accepts a JSON blob forces the user to know the schema. A tool that renders a form makes the schema discoverable. Form layout is not cosmetic — it is the tool's user interface, and tool authors should control it.
