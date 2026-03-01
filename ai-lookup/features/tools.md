# Tools Feature Trace

Custom AI agent tools — TypeScript scripts following a `--meta / --run / --setup` protocol, invoked by the server and tracked with interaction history. Tools can be created via CLI scaffold, cloned from built-ins, or built end-to-end by an agent using the Plan-Act pattern.

---

## Entry Points Table

| Component | File | Purpose |
|-----------|------|---------|
| **UI Page** | `frontend/src/pages/ToolsPage.tsx` | Main tool management UI |
| **Create Modal** | `ToolsPage.tsx:13-300+` | 4-step Plan→Build creation flow |
| **AMA Answer Panel** | `frontend/src/components/tools/AmaAnswerPanel.tsx` | Streams agent output via SSE |
| **Tool Result Actions** | `frontend/src/components/tools/ToolResultActions.tsx` | Renders post-run action buttons |
| **API Client** | `frontend/src/api/client.ts:196-309` | All tool API endpoints |
| **Server Routes** | `crates/sdlc-server/src/routes/tools.rs` | HTTP handlers for `/api/tools/*` |
| **Plan-Act Endpoints** | `crates/sdlc-server/src/routes/runs.rs:1825-2100+` | `plan_tool`, `build_tool`, `evolve_tool`, `act_tool` |
| **Route Registration** | `crates/sdlc-server/src/lib.rs:305-354` | All `/api/tools/*` route bindings |
| **SSE Events** | `crates/sdlc-server/src/state.rs:200-205` | `ToolPlanCompleted`, `ToolBuildCompleted` |
| **CLI Commands** | `crates/sdlc-cli/src/cmd/tool.rs` | `sdlc tool list/run/sync/info/scaffold` |
| **Core Runner** | `crates/sdlc-core/src/tool_runner.rs` | Runtime detection + subprocess invocation |
| **Interaction Persistence** | `crates/sdlc-core/src/tool_interaction.rs` | Run history YAML storage |
| **Path Helpers** | `crates/sdlc-core/src/paths.rs` | `.sdlc/tools/`, `.sdlc/tool-interactions/` constants |

---

## File Storage Layout

```
.sdlc/
├── tools/
│   ├── ama/                      # Built-in AMA tool (managed, overwritten by sdlc init)
│   │   ├── tool.ts
│   │   ├── config.yaml
│   │   └── README.md
│   ├── quality-check/            # Built-in quality checker (managed)
│   │   ├── tool.ts
│   │   └── config.yaml
│   ├── _shared/                  # Shared TS utilities (types, log, runtime, config)
│   ├── {custom-tool}/            # User-created tools
│   │   ├── tool.ts
│   │   ├── config.yaml
│   │   └── README.md
│   └── tools.md                  # Generated manifest (sdlc tool sync)
└── tool-interactions/
    ├── ama/threads/{id}.yaml     # AMA thread metadata
    ├── ama/{interaction-id}.yaml
    └── {tool-name}/{id}.yaml     # Max 200 per tool (enforced after each run)
```

---

## API Endpoints

```
GET    /api/tools                               → list_tools
POST   /api/tools                               → create_tool (scaffold)
GET    /api/tools/{name}                        → get_tool_meta
POST   /api/tools/{name}/clone                  → clone_tool
POST   /api/tools/{name}/run                    → run_tool
POST   /api/tools/{name}/setup                  → setup_tool
POST   /api/tools/{name}/evolve                 → evolve_tool (runs.rs)
POST   /api/tools/{name}/act                    → act_tool (runs.rs)
GET    /api/tools/{name}/interactions           → list_tool_interactions
GET    /api/tools/{name}/interactions/{id}      → get_tool_interaction
DELETE /api/tools/{name}/interactions/{id}      → delete_tool_interaction
POST   /api/tools/plan                          → plan_tool (runs.rs) — Phase 1 of Plan-Act
POST   /api/tools/build                         → build_tool (runs.rs) — Phase 2 of Plan-Act
POST   /api/tools/ama/threads                   → create AMA thread
POST   /api/tools/ama/answer                    → answer_ama (runs.rs)
POST   /api/tools/quality-check/reconfigure     → reconfigure quality gates
POST   /api/tools/quality-check/fix             → fix quality issues
```

---

## Tool Protocol

Tools are TypeScript scripts invoked as subprocesses. Three modes:

| Mode | Stdin | Stdout | Non-zero exit |
|------|-------|--------|---------------|
| `--meta` | none | JSON `ToolMeta` | Error |
| `--run` | JSON input | JSON `ToolResult` (`ok: true/false`) | Valid (means `ok: false`) |
| `--setup` | none | JSON `ToolResult` | Error |

Runtime detection priority: **bun > deno > node (npx tsx)**

`SDLC_ROOT` is always injected as env var. Secrets declared in `ToolMeta.secrets` are resolved from env vars and injected.

---

## ToolMeta Fields (tool_runner.rs)

```rust
pub struct ToolMeta {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub requires_setup: bool,
    pub setup_done: Option<bool>,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub secrets: Option<Vec<SecretRef>>,       // env vars to inject
    pub form_layout: Option<Vec<FormField>>,   // dynamic form generation (not yet used in UI)
    pub streaming: Option<bool>,               // not yet implemented
    pub result_actions: Option<Vec<ResultAction>>,
    pub timeout_seconds: Option<u64>,
    pub tags: Option<Vec<String>>,
    pub threaded: Option<bool>,
    pub persist_interactions: Option<bool>,
}
```

---

## Execution Flow: run_tool

```
POST /api/tools/{name}/run
  → validate slug (tools.rs:validate_tool_name — [a-zA-Z0-9-_]+ only)
  → run --meta → get ToolMeta → resolve_secrets (check required env vars)
  → create ToolInteractionRecord { status: "running" }, save YAML (best-effort)
  → tool_runner::run_tool(script, "--run", stdin_json, root, secrets)
    → detect_runtime(): bun > deno > node
    → spawn subprocess, write stdin, collect stdout, inherit stderr (real-time logs)
  → parse JSON output (ok:false is valid — not an error)
  → update interaction: { status: "completed", result, completed_at }
  → enforce_interaction_retention(max=200) — delete oldest first
  → return JSON to client
```

---

## Plan-Act Tool Creation Flow

```
POST /api/tools/plan { name, description, requirements? }
  → validate slug
  → build prompt: design input_schema, output_schema, approach, dependencies
  → spawn_agent_run("tool-plan:{name}", max_turns=15)
  → SSE stream → AmaAnswerPanel in frontend
  → emit SseMessage::ToolPlanCompleted { name }

Frontend: 'planning' → 'adjusting'
  → plan text shown in editable textarea
  → user may adjust, then clicks "Continue to Build"

POST /api/tools/build { name, plan }
  → validate slug, plan not empty
  → build prompt: use /sdlc-tool-build skill, scaffold → implement → test → sdlc tool sync → commit
  → spawn_agent_run("tool-build:{name}", max_turns=25)
  → SSE stream → AmaAnswerPanel
  → emit SseMessage::ToolBuildCompleted { name }

Frontend: 'building' → close modal, refresh tools list
```

---

## CreateToolModal State Machine (ToolsPage.tsx)

```
type CreateStep = 'form' | 'planning' | 'adjusting' | 'building'

form       → POST /api/tools/plan → move to 'planning'
planning   → stream via AmaAnswerPanel, wait for ToolPlanCompleted → 'adjusting'
adjusting  → textarea with editable plan → POST /api/tools/build → 'building'
building   → stream via AmaAnswerPanel, wait for ToolBuildCompleted → close modal
```

---

## Interaction History (tool_interaction.rs)

- Location: `.sdlc/tool-interactions/{tool-name}/{id}.yaml`
- ID format: timestamp-based (lexicographic sort = chronological)
- Max 200 per tool — `enforce_interaction_retention()` runs after every `run_tool` call
- Fields: `id`, `tool_name`, `created_at`, `completed_at`, `input`, `result`, `status`, `tags`, `notes`, `streaming_log`

---

## Quality Assessment

### The Good
- `tools.rs` has 15+ unit tests (list, get, run, setup, create, validate, clone)
- `tool_runner.rs` tests: runtime detection, scaffold correctness
- `tool_interaction.rs` tests: full CRUD + retention enforcement
- Best-effort interaction saves — failures don't abort tool runs
- Stderr inherited in real-time for tool debugging

### The Bad (gaps)
- Zero tests for `plan_tool`, `build_tool`, `evolve_tool`, `act_tool` in `runs.rs`
- No tests for `resolve_secrets()` (env var injection + missing secret error)
- No frontend e2e tests for CreateToolModal 4-step flow

### Incomplete / Partial Features
| Feature | Evidence |
|---------|---------|
| `streaming` mode | `ToolMeta.streaming` field exists but no implementation |
| `form_layout` UI | `FormField` struct defined but no frontend consumer found |
| `ResultAction` conditions | JSONPath evaluation referenced but no library confirmed |

---

## Managed (Built-in) Tools

Defined in `tool_runner.rs`:
```rust
const MANAGED_TOOLS: &[&str] = &["ama", "quality-check"];
```
These are always overwritten by `sdlc init` — do not manually edit `.sdlc/tools/ama/` or `.sdlc/tools/quality-check/`.

---

## Uncertainties

1. AMA threads (`tool-interactions/ama/threads/`) use a separate data model (`ama_threads.rs`) — relationship to base tool interaction model needs verification
2. `_shared/` directory contents (types.ts, log.ts, runtime.ts, config.ts) — what APIs are available to tool authors?
3. How `result_actions` conditions are evaluated on the frontend — no JSONPath library import confirmed
4. `evolveTool` endpoint exists but integration with the creation flow unclear
