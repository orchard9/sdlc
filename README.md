# sdlc

A deterministic SDLC state machine for autonomous software projects. Manage features, artifacts, tasks, and milestones through a structured lifecycle — driven by AI agents or humans.

## Quickstart

### Install

```bash
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```

Or from source:

```bash
git clone https://github.com/orchard9/sdlc
cd sdlc
cargo install --path crates/sdlc-cli
```

Verify:

```bash
sdlc --version
```

### Initialize a project

```bash
cd your-project
sdlc init
```

This creates:
- `.sdlc/config.yaml` — gates, platform commands, quality thresholds
- `.sdlc/state.yaml` — feature lifecycle state
- `.ai/` — project knowledge base (harvested by agents)
- `AGENTS.md` — SDLC section injected (or created)
- `.claude/commands/` — `/sdlc-next`, `/sdlc-status`, `/sdlc-approve`

### Create a feature

```bash
sdlc feature create auth-login --title "User authentication with OAuth"
sdlc next --for auth-login
```

Output:
```json
{
  "feature": "auth-login",
  "action": "create_spec",
  "message": "No spec exists. Write the feature specification for 'auth-login'.",
  "output_path": ".sdlc/features/auth-login/spec.md"
}
```

### Drive a feature through the lifecycle

```bash
# See what's next across all active features
sdlc next

# Approve an artifact to advance the phase
sdlc artifact approve auth-login spec

# Check overall project state
sdlc state

# Launch the web UI
sdlc ui
```

### Feature lifecycle

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```

Each phase transition requires the relevant artifacts to be present and approved.

---

## Core Concepts

### Features

A feature is a unit of work tracked through the lifecycle. Each feature has:
- A slug (e.g., `auth-login`)
- A title and optional description
- A current phase
- Artifacts at each phase

```bash
sdlc feature create <slug> --title "..." [--description "..."]
sdlc feature list
sdlc feature show <slug>
sdlc feature list --phase implementation
```

### Artifacts

Each phase requires specific artifacts. Artifacts are Markdown files written by agents or humans, then approved to advance the phase.

| Phase | Artifact |
|---|---|
| `specified` | `spec.md` |
| `planned` | `spec.md`, `design.md`, `tasks.md`, `qa-plan.md` |
| `review` | `review.md` |
| `audit` | `audit.md` |
| `qa` | `qa-results.md` |

```bash
sdlc artifact approve <slug> <type>    # advance phase
sdlc artifact reject <slug> <type>     # mark for revision
```

### Tasks

Tasks live inside a feature and track granular implementation work.

```bash
sdlc task add <slug> --title "Implement JWT middleware"
sdlc task start <slug> T1
sdlc task complete <slug> T1
sdlc task get <slug> T1        # full detail
sdlc task search "JWT"         # search across all features
```

### Comments

Async collaboration on features, tasks, and artifacts. `Blocker`-flagged comments halt the classifier.

```bash
sdlc comment create <slug> --message "Need clarification on token expiry" --flag Blocker
sdlc comment list <slug>
sdlc comment resolve <slug> C1
```

### Milestones

Named containers for feature sets with a shared goal.

```bash
sdlc milestone create v2-launch --title "Version 2.0 Launch"
sdlc milestone review v2-launch    # status table across all features
```

### Verification Gates

Every action has configurable verification gates — shell commands that must pass before the phase advances. Defined in `.sdlc/config.yaml`:

```yaml
gates:
  implement_task:
    - type: shell
      command: "cargo test"
      name: "test"
      auto: true
      max_retries: 3
  create_spec:
    - type: human
      name: "review"
      auto: false
```

---

## CLI Reference

```bash
# Lifecycle
sdlc init [--platform <name>]          # initialize .sdlc/ in current project
sdlc state                             # show project state
sdlc next [--for <slug>] [--json]      # classify next action
sdlc run <slug> [--dry-run]            # classify + dispatch to configured backend

# Features
sdlc feature create <slug> --title "..."
sdlc feature list [--phase <phase>]
sdlc feature show <slug>
sdlc archive <slug>

# Artifacts
sdlc artifact approve <slug> <type>
sdlc artifact reject <slug> <type>

# Tasks
sdlc task add|start|complete|block <slug>
sdlc task get|edit|search <slug>

# Comments
sdlc comment create|list|resolve <slug>

# Milestones
sdlc milestone create|list|info|review <slug>

# Quality scoring
sdlc score set <slug> <lens> <score>
sdlc score show <slug>
sdlc score history <slug>

# Project-level
sdlc project status|stats|blockers
sdlc query blocked|ready|needs-approval

# Platform extension
sdlc platform <command> [args]         # project-specific scripts from config

# Configuration
sdlc config validate

# Web UI
sdlc ui [--port 3141] [--no-open]
```

All commands support `--json` for machine-readable output.

---

## Configuration

`.sdlc/config.yaml` controls gates, platform commands, agent routing, and quality thresholds. Created automatically by `sdlc init`.

See [`docs/architecture.md`](docs/architecture.md) for the full schema.

---

## Web UI

```bash
sdlc ui
```

Opens a React dashboard on `http://localhost:3141`. Shows features grouped by attention type (needs approval / in progress / blocked / completed), live agent output via SSE, artifact approval flow, and gate status.

---

## Building from Source

```bash
git clone https://github.com/orchard9/sdlc
cd sdlc
cargo build --release
cargo test --all
```

The workspace has three crates:
- `crates/sdlc-core` — state machine, classifier, types (no binary)
- `crates/sdlc-cli` — the `sdlc` binary
- `crates/sdlc-server` — axum HTTP server + SSE for the web UI

To build the frontend separately:

```bash
cd frontend
npm install
npm run build    # outputs to frontend/dist/ (embedded into sdlc-server at compile time)
```

---

## Integration with AI Agents

`sdlc` is designed as the state layer for AI-driven development pipelines. The `sdlc next --json` command is the orchestrator interface — it returns a structured action classification that agents consume to decide what to do next.

See [`docs/vision.md`](docs/vision.md) for the full orchestration model.
