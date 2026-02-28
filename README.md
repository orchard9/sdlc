# sdlc

A deterministic SDLC state machine for software projects. Manage features, artifacts, tasks, and milestones through a structured lifecycle. Emits structured JSON directives that AI agents and humans consume to decide what to do next.

## Quickstart

### Install

**macOS / Linux** — prebuilt binary, no prerequisites:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/orchard9/sdlc-releases/releases/latest/download/sdlc-installer.sh | sh
```

**Windows** — prebuilt binary, no prerequisites:

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/orchard9/sdlc-releases/releases/latest/download/sdlc-installer.ps1 | iex"
```

**Homebrew** (macOS / Linux):

```bash
brew install orchard9/tap/sdlc
```

**From source** (requires [Rust](https://rustup.rs) and [Node.js ≥ 18](https://nodejs.org)):

```bash
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```

The build script automatically compiles the frontend — no manual npm step needed.

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
- `.ai/` — project knowledge base (patterns, decisions, gotchas)
- `AGENTS.md` — SDLC section injected (or created)
- `.claude/commands/` — Claude Code slash command scaffolding
- `.gemini/commands/` — Gemini CLI native command TOML files
- `.opencode/command/` — OpenCode native command files
- `.agents/skills/` — Codex native skills

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

The lifecycle is the recommended path: approvals and artifacts keep work reviewable, while manual transitions remain available when teams need to move with intent.

### Typical daily workflow

```bash
# 1. See what needs attention
sdlc next

# 2. Get the directive for a specific feature
sdlc next --for auth-login --json

# 3. Act on the directive (write artifact, implement task, etc.)
#    Then check the new state
sdlc next --for auth-login

# 4. Approve artifacts to advance phases
sdlc artifact approve auth-login spec

# 5. See all features waiting for approval
sdlc query needs-approval

# 6. See all blocked features
sdlc query blocked
```

**With Claude Code:** use the slash commands installed by `sdlc init`:
- `/sdlc-ponder [slug]` — open the ideation workspace (explore ideas, recruit thought partners, capture artifacts)
- `/sdlc-ponder-commit <slug>` — crystallize a pondered idea into milestones and features
- `/sdlc-recruit <role>` — recruit an expert thought partner as a persistent agent
- `/sdlc-empathy <subject>` — deep user perspective interviews before making decisions
- `/sdlc-run <slug>` — autonomously drive a feature to the next human gate
- `/sdlc-next <slug>` — get the next directive and act on it
- `/sdlc-status` — project overview
- `/sdlc-plan` — distribute a plan into milestones, features, and tasks
- `/sdlc-pressure-test <milestone-slug>` — pressure-test a milestone against user perspectives
- `/sdlc-milestone-uat <milestone-slug>` — run the acceptance test for a milestone

**With Gemini CLI:** use `.gemini/commands/*.toml` (`sdlc-next.toml`, `sdlc-status.toml`, `sdlc-approve.toml`).

**With OpenCode:** use `.opencode/command/*.md` (`sdlc-next.md`, `sdlc-status.md`, `sdlc-approve.md`).

**With Codex:** use `.agents/skills/*/SKILL.md` (`sdlc-next`, `sdlc-status`, `sdlc-approve`).

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

Each phase requires specific artifacts. Artifacts are Markdown files (written by AI agents, humans, or both), then approved to advance the phase.

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
sdlc milestone review v2-launch    # status table across all features
```

### Roadmap (Ideation / Ponder)

A pre-milestone ideation workspace for exploring ideas before they enter the state machine. Ideas live at `.sdlc/roadmap/<slug>/` as a scrapbook of Markdown artifacts, a recruited team of thought partners, and a manifest tracking status.

```bash
# Open an idea (or start a new one)
sdlc ponder create preference-engine --title "Dynamic preference system"
sdlc ponder list                                           # all active ideas
sdlc ponder show preference-engine                        # manifest + team + scrapbook

# Capture thinking into the scrapbook
sdlc ponder capture preference-engine --content "..." --as problem.md
sdlc ponder capture preference-engine --file /tmp/notes.md --as exploration.md

# Manage thought partners
sdlc ponder team add preference-engine --name kai-tanaka --role "Preference systems architect" \
  --context "Built Spotify's preference engine" --agent .claude/agents/kai-tanaka.md

# Update status and park unused ideas
sdlc ponder update preference-engine --status converging
sdlc ponder archive old-idea                              # parks without deleting
```

**Ideation flow with Claude Code:**
```
/sdlc-ponder <slug>              # explore: interrogate, empathize, recruit, capture
/sdlc-ponder-commit <slug>       # commit: synthesize scrapbook → milestones + features
/sdlc-pressure-test <milestone>  # validate: pressure-test the new milestone
/sdlc-run <feature-slug>         # execute: drive features through the lifecycle
```

**Status lifecycle:** `exploring` → `converging` → `committed` (or `parked` if shelved)

### Gates (Consumer Hints)

Gates are verification hints published in the directive output. sdlc includes them in `sdlc next --json`; the consumer decides whether and how to act on them. Defined in `.sdlc/config.yaml`:

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
sdlc update                            # refresh agent scaffolding after upgrading sdlc

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
sdlc task list [<slug>]                # list tasks for a feature (or all features)
sdlc task get <slug> <task-id>
sdlc task edit <slug> <task-id> [--title] [--description] [--depends T1,T2]
sdlc task search <query> [--slug <slug>]

# Comments
sdlc comment create|list|resolve <slug>

# Milestones
sdlc milestone create <slug> --title "..." [--feature <slug>...]
sdlc milestone list
sdlc milestone info <slug>
sdlc milestone tasks <slug>            # list all tasks across milestone features
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
sdlc query search <query>              # full-text search across features
sdlc query search-tasks <query>        # full-text search across tasks

# Platform extension
sdlc platform <command> [args]         # project-specific scripts from config

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

Opens a React dashboard at a random OS-assigned port (printed on startup). Shows features grouped by milestone, phase progress, artifact approval flow, and the next command to run for each feature. State updates automatically via SSE — no refresh needed.

Pages:
- **Dashboard** — active features, milestones, and pondering ideas at a glance
- **Features** — all features with phase and status
- **Milestones** — milestone containers with feature progress
- **Roadmap** — ponder entries (ideation workspace); click any entry to see the scrapbook, team, and copy `/sdlc-ponder` commands
- **Archive** — released and archived features

---

## Building from Source

Requires [Rust](https://rustup.rs) and [Node.js ≥ 18](https://nodejs.org).

```bash
git clone https://github.com/orchard9/sdlc
cd sdlc
cargo build --release
cargo test --all
```

The frontend is built automatically by `crates/sdlc-server/build.rs` the first time Cargo compiles the server crate. After that, run `npm run build` in `frontend/` to update assets and they will be re-embedded on the next `cargo build`.

The workspace has three crates:
- `crates/sdlc-core` — state machine, classifier, types (no binary)
- `crates/sdlc-cli` — the `sdlc` binary
- `crates/sdlc-server` — axum HTTP server + SSE for the web UI

---

## Integration with AI Agents

`sdlc next --json` is the directive interface — it returns a structured classification that any consumer (AI agent, script, or human) uses to decide what to do next. sdlc has no opinion on who or what acts on the directive.

See [`docs/vision.md`](docs/vision.md) for the full directive model.
