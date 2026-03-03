# Ponder

A deterministic feature lifecycle engine for software projects. Manage features, artifacts, tasks, and milestones through a structured lifecycle. Emits structured JSON directives that AI agents and humans consume to decide what to do next.

- **[QUICKSTART.md](QUICKSTART.md)** — zero to working in 5 minutes
- **[DEVELOPER.md](DEVELOPER.md)** — contributor setup, dev loop, build targets

---

## Install

**macOS / Linux:**

```bash
curl -sSfL https://raw.githubusercontent.com/orchard9/sdlc/main/install.sh | sh
```

**Windows:**

```powershell
irm https://raw.githubusercontent.com/orchard9/sdlc/main/install.ps1 | iex
```

Installs `ponder` to `~/.local/bin` with a `sdlc` alias — both commands work interchangeably.

```bash
ponder --version   # or: sdlc --version
```

Then `cd your-project && sdlc init` — see [QUICKSTART.md](QUICKSTART.md) for the full walkthrough.

---

## Core Concepts

### Features

A feature is a unit of work tracked through the lifecycle. Each feature has a slug, a title, a current phase, and artifacts at each phase.

```bash
sdlc feature create <slug> --title "..." [--description "..."]
sdlc feature list [--phase <phase>]
sdlc feature show <slug>
```

### Artifacts

Artifacts are Markdown files written by AI agents or humans, then approved to advance the phase.

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
sdlc task get <slug> T1
sdlc task search "JWT"
```

### Comments

Async collaboration on features, tasks, and artifacts. `Blocker` and `Question` comments halt the classifier.

```bash
sdlc comment create <slug> --message "Need clarification on token expiry" --flag Blocker
sdlc comment list <slug>
sdlc comment resolve <slug> C1
```

### Milestones

Named containers for feature sets with a shared goal.

```bash
sdlc milestone create v2-launch --title "Version 2.0 Launch"
sdlc milestone add-feature v2-launch auth-login
sdlc milestone info v2-launch
```

### Roadmap (Ponder / Ideation)

A pre-milestone workspace for exploring ideas before they enter the state machine. Ideas live at `.sdlc/roadmap/<slug>/` as a scrapbook of Markdown artifacts, a recruited team, and a manifest.

```bash
sdlc ponder create preference-engine --title "Dynamic preference system"
sdlc ponder list
sdlc ponder show preference-engine
sdlc ponder capture preference-engine --content "..." --as problem.md
sdlc ponder update preference-engine --status converging
sdlc ponder archive old-idea
```

**Status lifecycle:** `exploring` → `converging` → `committed` (or `parked`)

### Gates (Consumer Hints)

Gates are verification hints emitted in `sdlc next --json`. Defined in `.sdlc/config.yaml`:

```yaml
gates:
  implement_task:
    - type: shell
      command: "cargo test"
      name: "test"
      auto: true
      max_retries: 3
```

---

## CLI Reference

```bash
# Ideation (Ponder / Roadmap)
sdlc ponder create <slug> --title "..."
sdlc ponder list [--status <status>]
sdlc ponder show <slug>
sdlc ponder capture <slug> --content "<text>" --as <filename>
sdlc ponder capture <slug> --file <path> [--as <filename>]
sdlc ponder team add <slug> --name <name> --role <role> --context <ctx> --agent <path>
sdlc ponder team list <slug>
sdlc ponder update <slug> [--status <status>] [--title "..."] [--tag <tag>]
sdlc ponder archive <slug>
sdlc ponder artifacts <slug>

# Lifecycle
sdlc init [--platform <name>]          # initialize .sdlc/ in current project
sdlc state                             # show project state
sdlc next [--for <slug>]               # classify next action (directive interface)
sdlc focus                             # single highest-priority action (milestone order)
sdlc update                            # refresh agent scaffolding after upgrading

# Features
sdlc feature create <slug> --title "..."
sdlc feature list [--phase <phase>]
sdlc feature show <slug>
sdlc feature update <slug> [--title "..."] [--description "..."]
sdlc feature transition <slug> <phase> # force a phase (setup/recovery only)
sdlc archive <slug>                    # shorthand for sdlc feature archive
sdlc merge <slug>                      # finalize merge phase and mark feature released

# Artifacts
sdlc artifact approve <slug> <type>
sdlc artifact reject <slug> <type>

# Tasks
sdlc task add <slug> <title>
sdlc task start <slug> <task-id>
sdlc task complete <slug> <task-id>
sdlc task block <slug> <task-id> <reason>
sdlc task list [<slug>]
sdlc task get <slug> <task-id>
sdlc task edit <slug> <task-id> [--title] [--description] [--depends T1,T2]
sdlc task search <query> [--slug <slug>]

# Comments
sdlc comment create|list|resolve <slug>

# Milestones
sdlc milestone create <slug> --title "..." [--feature <slug>...]
sdlc milestone list
sdlc milestone info <slug>
sdlc milestone tasks <slug>
sdlc milestone add-feature <slug> <feature> [--position N]
sdlc milestone remove-feature <slug> <feature>
sdlc milestone reorder <slug> <feature>...
sdlc milestone skip <slug>
sdlc milestone release <slug>

# Quality scoring
sdlc score set <slug> <lens> <score>
sdlc score show <slug>
sdlc score history <slug>

# Project-level
sdlc project status|stats|blockers
sdlc query blocked
sdlc query ready [--phase <phase>]
sdlc query needs-approval
sdlc query search <query>
sdlc query search-tasks <query>

# Platform extension
sdlc platform <command> [args]

# Configuration
sdlc config validate

# Autonomous agent (drives features with Claude)
sdlc agent run <slug> [--max-turns N] [--model <id>]

# Web UI
sdlc ui [--port <port>] [--no-open]
sdlc ui list
sdlc ui kill [<name>]
sdlc ui open [<name>]
```

All commands support `--json` for machine-readable output.

---

## Configuration

`.sdlc/config.yaml` controls gates, platform commands, and quality thresholds. Created automatically by `sdlc init`.

See [`docs/architecture.md`](docs/architecture.md) for the full schema.

---

## Web UI

```bash
sdlc ui
```

Opens a React dashboard at a random OS-assigned port. Shows features grouped by milestone, phase progress, artifact approval flow, and the next command for each feature. State updates automatically via SSE — no refresh needed.

Pages: **Dashboard**, **Features**, **Milestones**, **Roadmap**, **Archive**

---

## Integration with AI Agents

`sdlc next --json` is the directive interface — it returns a structured classification that any consumer (AI agent, script, or human) uses to decide what to do next. Ponder has no opinion on who or what acts on the directive.

See [`docs/vision.md`](docs/vision.md) for the full directive model.
See [`AGENTS.md`](AGENTS.md) for the full agent instruction set.
