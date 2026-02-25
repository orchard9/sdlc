# sdlc Architecture

Technical reference for contributors and integrators.

---

## Workspace Layout

```
sdlc/
├── Cargo.toml                  # workspace root
├── Cargo.lock
├── crates/
│   ├── sdlc-core/              # state machine library (no binary)
│   │   └── src/
│   │       ├── lib.rs          # public module exports
│   │       ├── types.rs        # Phase, ArtifactType, ActionType, TaskStatus
│   │       ├── feature.rs      # Feature struct, CRUD, artifact management
│   │       ├── task.rs         # Task struct, lifecycle transitions
│   │       ├── state.rs        # State struct (project-level summary)
│   │       ├── config.rs       # Config struct (gates, platform, agent routing)
│   │       ├── classifier.rs   # sdlc next — rule engine
│   │       ├── artifact.rs     # Artifact approval/rejection
│   │       ├── comment.rs      # Comments with flag types
│   │       ├── milestone.rs    # Milestone containers
│   │       ├── score.rs        # Three-lens quality scores
│   │       ├── gate.rs         # Gate definitions and runner
│   │       ├── rules.rs        # Classifier rule evaluation
│   │       ├── paths.rs        # Path constants (.sdlc/, .ai/, etc.)
│   │       ├── io.rs           # Atomic file writes, directory ops
│   │       └── error.rs        # SdlcError enum
│   │
│   ├── sdlc-cli/               # the `sdlc` binary
│   │   └── src/
│   │       ├── main.rs         # clap CLI, command dispatch
│   │       ├── root.rs         # project root resolution
│   │       ├── output.rs       # JSON/text output helpers
│   │       └── cmd/            # one file per top-level command
│   │           ├── init.rs     # sdlc init
│   │           ├── feature.rs  # sdlc feature *
│   │           ├── artifact.rs # sdlc artifact approve|reject
│   │           ├── task.rs     # sdlc task *
│   │           ├── comment.rs  # sdlc comment *
│   │           ├── milestone.rs# sdlc milestone *
│   │           ├── next.rs     # sdlc next
│   │           ├── run.rs      # sdlc run
│   │           ├── state.rs    # sdlc state
│   │           ├── score.rs    # sdlc score *
│   │           ├── gate.rs     # sdlc gate *
│   │           ├── config.rs   # sdlc config validate
│   │           ├── platform.rs # sdlc platform *
│   │           ├── project.rs  # sdlc project status|stats|blockers
│   │           └── query.rs    # sdlc query blocked|ready|needs-approval
│   │
│   └── sdlc-server/            # web UI HTTP server
│       └── src/
│           ├── lib.rs          # serve() entry point
│           ├── routes/         # axum route handlers
│           ├── state.rs        # shared AppState
│           ├── embed.rs        # rust-embed for frontend/dist/
│           ├── subprocess.rs   # SSE streaming for sdlc run
│           └── error.rs        # server error types
│
└── frontend/                   # React + Vite dashboard
    ├── src/
    │   ├── pages/              # FeaturesPage, MilestonesPage, SetupWizard, etc.
    │   ├── components/         # UI components (shadcn/ui base)
    │   └── lib/                # API client, SSE helpers
    └── dist/                   # built output, embedded into sdlc-server at compile time
```

---

## Data Layer

All state is stored as YAML files in `.sdlc/` at the project root. No database. No network.

### `.sdlc/state.yaml`

Project-level summary. Written by `sdlc init`, updated by feature/task/milestone mutations.

```yaml
project: my-project
created_at: "2026-01-15T10:00:00Z"
features:
  auth-login:
    slug: auth-login
    title: "User authentication with OAuth"
    description: "OAuth with Google and GitHub. JWT tokens, refresh flow."
    phase: implementation
    artifacts:
      spec: approved
      design: approved
      tasks: approved
      qa_plan: approved
    tasks:
      T1: { title: "JWT middleware", status: completed }
      T2: { title: "OAuth callback handler", status: in_progress }
    comments: {}
    phase_history:
      - { phase: draft, entered_at: "2026-01-15T10:00:00Z" }
      - { phase: specified, entered_at: "2026-01-16T09:30:00Z" }
```

### `.sdlc/config.yaml`

Gates, platform commands, quality thresholds. Created by `sdlc init` with sensible defaults.

```yaml
project: my-project
quality:
  thresholds:
    product_fit: 70
    research_grounding: 70
    implementation: 70
gates:
  implement_task:
    - type: shell
      command: "cargo test"
      name: test
      auto: true
      max_retries: 3
  create_spec:
    - type: human
      name: review
      auto: false
platform:
  commands:
    deploy:
      description: "Deploy to environment"
      script: ".sdlc/platform/deploy.sh"
      args:
        - name: environment
          required: true
          choices: [staging, production]
```

### `.sdlc/features/<slug>/`

Per-feature artifacts. Each is a Markdown file written by an agent or human.

```
.sdlc/features/auth-login/
├── spec.md
├── design.md
├── tasks.md
├── qa-plan.md
├── review.md
├── audit.md
└── qa-results.md
```

---

## The Classifier

`sdlc next --for <slug> --json` is the core intelligence of the binary. It evaluates priority-ordered rules against current feature state and emits the highest-priority action.

**Rule evaluation order** (highest priority first):

1. Feature is `released` → `Done`
2. Unresolved blocker comment → `WaitForApproval`
3. Missing required artifact for current phase → `Create*` action
4. Artifact exists but not approved → `Approve*` action (human gate)
5. All artifacts approved → advance phase, emit next creation action
6. All phases complete → `Done`

Output schema:

```json
{
  "feature": "auth-login",
  "title": "User authentication with OAuth",
  "description": "OAuth with Google and GitHub...",
  "current_phase": "draft",
  "action": "create_spec",
  "message": "No spec exists. Write the feature specification for 'auth-login'.",
  "output_path": ".sdlc/features/auth-login/spec.md",
  "is_heavy": false,
  "timeout_minutes": 10,
  "gates": [
    { "name": "review", "type": "human", "auto": false }
  ]
}
```

**`is_heavy`** signals that the action is long-running (implementation, fix-review-issues, run-qa). Orchestrators use this to warn users or set longer timeouts.

**`gates`** lists the verification gates that will run after this action completes. The orchestrator knows what checks to expect before it dispatches.

---

## Gate System

Gates are defined in `config.yaml` and evaluated after each action.

### Gate Types

| Type | Behavior |
|---|---|
| `shell` | Runs a command. Passes if exit code 0. Retries on failure up to `max_retries`. |
| `human` | Always pauses. Cannot be auto-approved. Orchestrator must wait for `sdlc artifact approve`. |
| `step_back` | Dispatches a lightweight agent with adversarial questions at phase transitions. |

### Gate Runner

```
run_gate(gate, context):
  if gate.type == shell:
    for attempt in 1..=max_retries:
      result = exec(gate.command)
      if result.exit_code == 0: return Pass
      if attempt < max_retries: re-dispatch with error context
    return Fail(exhausted)

  if gate.type == human:
    return WaitForHuman(gate.name)
```

Exit codes from `sdlc run`:
- `0` — all gates passed
- `1` — agent error
- `2` — gate failure (mechanical)
- `3` — human gate (wait for approval)

---

## Action Types

Defined in `sdlc-core/src/types.rs`. These are the complete set of actions the classifier can emit. Orchestrators must handle all of them.

| Action | Heavy | Default Timeout |
|---|---|---|
| `create_spec` | No | 10 min |
| `approve_spec` | No | — (human) |
| `create_design` | No | 10 min |
| `approve_design` | No | — (human) |
| `create_tasks` | No | 10 min |
| `create_qa_plan` | No | 10 min |
| `implement_task` | **Yes** | 45 min |
| `fix_review_issues` | **Yes** | 45 min |
| `create_review` | No | 10 min |
| `approve_review` | No | — (human) |
| `create_audit` | No | 10 min |
| `run_qa` | **Yes** | 45 min |
| `approve_merge` | No | — (human) |
| `merge` | No | 10 min |
| `archive` | No | 10 min |
| `unblock_dependency` | No | 10 min |
| `wait_for_approval` | No | — (human) |
| `done` | No | — |

`Rust sdlc-core` owns this schema. Orchestrators consume it as strings from `sdlc next --json` output.

---

## `sdlc run` Dispatch

`sdlc run <slug>` is the integration point between the state machine and agent backends.

```
1. sdlc next --for <slug> --json         → classify
2. load .sdlc/config.yaml               → find agent route for action
3. build context string:
   - feature title + description
   - VISION.md (if exists in project root)
   - classification output (action, message, output_path)
4. dispatch to configured backend
5. evaluate gates on result
6. exit with appropriate code (0/1/2/3)
```

Agent routing config in `.sdlc/config.yaml`:

```yaml
agents:
  default:
    type: claude_agent_sdk
    model: claude-opus-4-6
  actions:
    create_spec:
      type: xadk
      agent_id: sdlc_spec
    create_design:
      type: claude_agent_sdk
      model: claude-opus-4-6

human_gates:
  - approve_spec
  - approve_design
  - approve_review
  - approve_merge
```

Supported backend types: `claude_agent_sdk`, `xadk`, `human`.

---

## Web UI Server (`sdlc-server`)

An axum HTTP server that exposes the same state machine operations as the CLI over REST + SSE.

### REST API

```
GET  /api/state                       → project state
GET  /api/features                    → all features
GET  /api/features/:slug              → single feature
POST /api/features/:slug/artifacts/:type/approve
POST /api/features/:slug/artifacts/:type/reject
GET  /api/features/:slug/next         → next action classification
POST /api/features/create             → create feature
GET  /api/milestones                  → all milestones
```

### SSE Streaming

`sdlc run` output is streamed via SSE:

```
GET /api/run/:slug/stream
→ Content-Type: text/event-stream
→ data: { "type": "stdout", "line": "dispatching to claude..." }
→ data: { "type": "gate", "name": "compile", "status": "pass" }
→ data: { "type": "done", "exit_code": 0 }
```

### Frontend Embedding

The React frontend is compiled to `frontend/dist/` and embedded into the binary at compile time via `rust-embed`. Running `sdlc ui` serves the SPA from memory with no external file dependency.

Development: set `SDLC_FRONTEND_DIR=./frontend/dist` to serve from disk without recompiling.

---

## Project Root Resolution

The binary resolves the project root at startup:

1. `--root <path>` flag (explicit)
2. `$SDLC_ROOT` environment variable
3. Walk up from `$PWD` looking for `.sdlc/` directory
4. Walk up from `$PWD` looking for `.git/` directory
5. Fall back to `$PWD`

This means `sdlc` commands work from any subdirectory of a project, the same way `git` does.

---

## File I/O Guarantees

All state writes in `sdlc-core/src/io.rs` are atomic:

- `atomic_write(path, bytes)` — writes to a `.tmp` file, then `rename()` to destination. Prevents partial writes from corrupting state.
- `write_if_missing(path, bytes)` — no-op if file exists. Safe to call from init multiple times.
- `ensure_dir(path)` — `create_dir_all`, idempotent.
- `append_text(path, text)` — appends to existing file (used for AGENTS.md SDLC section).

All state mutations are idempotent. Running `sdlc init` twice is safe.

---

## Testing

```bash
cargo test --all       # unit + integration tests for all crates
```

Test structure:
- `sdlc-core` — unit tests embedded in source files (types, classifier rules, state transitions)
- `sdlc-cli` — integration tests in `tests/integration.rs` using `tempfile` for isolated `.sdlc/` directories
- `sdlc-server` — unit tests for route handlers and SSE serialization

All integration tests run against real `.sdlc/` directory structures in temp directories — no mocking of the file system.

---

## Adding a New Command

1. Add a variant to the `Commands` enum in `main.rs`
2. Create `crates/sdlc-cli/src/cmd/<name>.rs`
3. Add the business logic in `sdlc-core` if it requires state access
4. Wire the dispatch in `main.rs`
5. Add integration tests in `tests/integration.rs`

If the new command introduces a new action type, add it to `ActionType` in `sdlc-core/src/types.rs` and update `ActionType::all()`. All tests that assert on `all().len()` will fail until updated — this is intentional, to catch incomplete additions.
