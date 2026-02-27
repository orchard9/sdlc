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
│   │       ├── config.rs       # Config struct (gates, platform, quality thresholds)
│   │       ├── classifier.rs   # sdlc next — rule engine
│   │       ├── artifact.rs     # Artifact approval/rejection
│   │       ├── comment.rs      # Comments with flag types
│   │       ├── directive.rs    # Builds full directive output (quality standard, approach, task, completion steps)
│   │       ├── milestone.rs    # Milestone containers
│   │       ├── score.rs        # Three-lens quality scores
│   │       ├── gate.rs         # Gate definitions and runner
│   │       ├── rules.rs        # Classifier rule evaluation
│   │       ├── paths.rs        # Path constants (.sdlc/, .ai/, etc.)
│   │       ├── io.rs           # Atomic file writes, directory ops
│   │       ├── search.rs       # In-memory full-text feature search using tantivy
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
│   │           ├── artifact.rs # sdlc artifact draft|approve|reject
│   │           ├── task.rs     # sdlc task *
│   │           ├── comment.rs  # sdlc comment *
│   │           ├── milestone.rs# sdlc milestone *
│   │           ├── next.rs     # sdlc next
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
│           │   └── config.rs   # project configuration (read-only)
│           ├── state.rs        # shared AppState
│           ├── embed.rs        # rust-embed for frontend/dist/
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
2. Unresolved blocker/question comment → `WaitForApproval` (human gate)
3. Feature is blocked → `UnblockDependency` (human gate)
4. Missing required artifact for current phase → `Create*` action (agent)
5. Artifact exists but not verified → `Approve*` action (agent verifies, then approves)
6. All artifacts approved → advance phase, emit next creation action
7. All phases complete → `Done`

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

**`is_heavy`** signals that the action is long-running (implementation, fix-review-issues, run-qa). Consumers may use this to warn users or set longer timeouts.

**`gates`** lists the verification checks associated with this action. Consumers may use this to decide what verification to run after acting on the directive.

---

## Gate System

Gates are metadata defined in `config.yaml` and included in the directive output. sdlc publishes them; the consumer decides whether and how to run them.

### Gate Types

| Type | Meaning |
|---|---|
| `shell` | A shell command the consumer may run to verify the action. Includes `command`, `max_retries`, and `timeout_seconds` hints. |
| `human` | An optional human review step configured by the team. In autonomous consumer mode, the agent evaluates the artifact and calls `sdlc artifact approve` directly. When `auto: false`, consumers that support interactive mode may pause and surface the artifact to a human reviewer. |
| `step_back` | Advisory questions the consumer may surface to a reviewer at phase transitions. |

Gates appear in the `gates` array of `sdlc next --json` output. They are not enforced by sdlc — they are signals to the consumer.

---

## Action Types

Defined in `sdlc-core/src/types.rs`. These are the complete set of actions the classifier can emit. Orchestrators must handle all of them.

| Action | Heavy | Who Acts | Default Timeout |
|---|---|---|---|
| `create_spec` | No | Agent | 10 min |
| `approve_spec` | No | Agent — verifies spec quality, then approves | 10 min |
| `create_design` | No | Agent | 10 min |
| `approve_design` | No | Agent — verifies design soundness, then approves | 10 min |
| `create_tasks` | No | Agent | 10 min |
| `approve_tasks` | No | Agent — verifies task breakdown, then approves | 10 min |
| `create_qa_plan` | No | Agent | 10 min |
| `approve_qa_plan` | No | Agent — verifies QA plan coverage, then approves | 10 min |
| `implement_task` | **Yes** | Agent | 45 min |
| `fix_review_issues` | **Yes** | Agent | 45 min |
| `create_review` | No | Agent | 10 min |
| `approve_review` | No | Agent — verifies review accuracy, then approves | 10 min |
| `create_audit` | No | Agent | 10 min |
| `approve_audit` | No | Agent — verifies audit accuracy, then approves | 10 min |
| `run_qa` | **Yes** | Agent | 45 min |
| `approve_merge` | No | Agent — verifies QA results, then approves merge | 10 min |
| `merge` | No | Agent | 10 min |
| `archive` | No | Agent | 10 min |
| `unblock_dependency` | No | **Human gate** — external blocker | — |
| `wait_for_approval` | No | **Human gate** — blocker comment or planning complete | — |
| `done` | No | — | — |

`Rust sdlc-core` owns this schema. Directive consumers receive these as strings from `sdlc next --json` output.

---

## Web UI Server (`sdlc-server`)

An axum HTTP server that exposes the same state machine operations as the CLI over REST, enabling remote access for web UIs and remote consumers.

### REST API

```
GET  /api/state                                    → project state summary
GET  /api/config                                   → project configuration (read-only)
GET  /api/features                                 → list all features
POST /api/features                                 → create a feature
GET  /api/features/:slug                           → single feature detail
GET  /api/features/:slug/next                      → next action classification
POST /api/features/:slug/transition                → transition feature phase
POST /api/features/:slug/tasks                     → add a task
POST /api/features/:slug/tasks/:id/start           → start a task
POST /api/features/:slug/tasks/:id/complete        → complete a task
POST /api/features/:slug/comments                  → add a comment
GET  /api/artifacts/:slug/:type                    → get artifact content and status
POST /api/artifacts/:slug/:type/approve            → approve an artifact
POST /api/artifacts/:slug/:type/reject             → reject an artifact
GET  /api/milestones                               → list all milestones
POST /api/milestones                               → create a milestone
GET  /api/milestones/:slug                         → milestone detail
GET  /api/milestones/:slug/review                  → milestone feature review
POST /api/milestones/:slug/features                → add feature to milestone
PUT  /api/milestones/:slug/features/order           → reorder milestone features
GET  /api/vision                                   → get vision document
PUT  /api/vision                                   → update vision document
POST /api/run/:slug                                → generate directive for feature
POST /api/init                                     → initialize project
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
