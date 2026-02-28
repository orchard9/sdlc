# CLAUDE.md

## Project

`sdlc` is a Rust CLI + library that implements a deterministic state machine for feature lifecycle management. It tracks features through structured phases, emits directives for AI consumers, and records approvals. It has no LLM calls — it is the state layer that agents operate against.

## Stack

- **Language**: Rust (stable, see `rust-toolchain.toml`)
- **Workspace crates**: `sdlc-core` (library), `sdlc-cli` (binary), `sdlc-server` (HTTP server)
- **Frontend**: React + Vite in `frontend/` — embedded into `sdlc-server` at compile time
- **State storage**: YAML files in `.sdlc/` — no database, no network
  - `.sdlc/features/<slug>/` — per-feature artifact Markdown files
  - `.sdlc/roadmap/<slug>/` — ponder (ideation) entries: manifest, team, scrapbook artifacts

## Build & Test

```bash
# Build (requires pre-built frontend or Node.js ≥ 18 in PATH)
cargo build --all

# Test — use SDLC_NO_NPM to skip the npm build step and avoid hangs
# when frontend/dist is absent. The flag tells build.rs to use a stub UI.
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings

# Build the real frontend (needed for sdlc-server UI, not for tests)
cd frontend && npm ci && npm run build
```

## Key Files

| File | Purpose |
|---|---|
| `crates/sdlc-core/src/types.rs` | Phase, ArtifactType, ActionType, TaskStatus enums |
| `crates/sdlc-core/src/rules.rs` | Priority-ordered classifier rules — the full state machine |
| `crates/sdlc-core/src/classifier.rs` | Rule engine that evaluates rules against feature state |
| `crates/sdlc-core/src/feature.rs` | Feature struct, artifact management, phase transitions |
| `crates/sdlc-core/src/gate.rs` | Gate definitions (shell, human, step_back) |
| `crates/sdlc-core/src/ponder.rs` | PonderEntry, PonderStatus, team/artifact CRUD — roadmap ideation layer |
| `crates/sdlc-cli/src/cmd/next.rs` | `sdlc next` — classifies and formats the directive |
| `crates/sdlc-cli/src/cmd/ponder.rs` | `sdlc ponder *` — CLI for the ideation workspace |
| `crates/sdlc-server/src/routes/roadmap.rs` | REST routes for ponder entries (`/api/roadmap`) |
| `crates/sdlc-server/src/auth.rs` | Tunnel auth middleware — token/cookie gate, local bypass |
| `crates/sdlc-cli/src/cmd/tunnel.rs` | Tunnel process lifecycle (cloudflared), QR printing, token generation |
| `.sdlc/state.yaml` | Project-level state summary (includes `active_ponders`) |
| `.sdlc/config.yaml` | Gates, platform commands, quality thresholds |
| `.sdlc/features/<slug>/` | Per-feature artifact Markdown files |
| `.sdlc/roadmap/<slug>/` | Ponder entry: `manifest.yaml`, `team.yaml`, scrapbook Markdown files |

## The State Machine Flow

```
DRAFT
  no spec        → create_spec      (agent writes spec.md)
  spec=Draft     → approve_spec     (agent verifies, then approves or rejects)
  spec=Rejected  → create_spec      (agent rewrites)
  spec=Approved  → transition to specified

SPECIFIED
  no design                    → create_design
  design=Draft                 → approve_design
  design=Approved, no tasks    → create_tasks
  tasks=Draft                  → approve_tasks
  tasks+design=Approved, no qa → create_qa_plan
  qa_plan=Draft                → approve_qa_plan
  all planning approved        → transition to planned

PLANNED → transition to ready
READY   → transition to implementation

IMPLEMENTATION
  pending tasks  → implement_task
  all done       → create_review → transition to review

REVIEW
  review=Draft     → approve_review
  review=Rejected  → fix_review_issues
  review=Approved  → transition to audit

AUDIT
  no audit       → create_audit
  audit=Draft    → approve_audit
  audit=Approved → transition to qa

QA
  no results          → run_qa
  results=Draft       → approve_merge
  results=Failed      → fix_review_issues
  results=Approved    → transition to merge

MERGE → merge (released)
```

All actions — including all `approve_*` verification steps and `approve_merge` — are executed agentively.

## Ethos

**Workspaces are the funnel into the work.** Ponder, root-cause, evolve, and guideline workspaces exist so ideas and problems are explored before they become features. See `workspaces.md`.

**Always forward.** Issues are captured as tasks and addressed in subsequent cycles — features and milestone state are never reverted or reset. When a problem is found, create a task and keep moving unless the blocker makes further progress meaningless.

**Autonomous by default.** Agents execute all actions without pausing for human input. Approval steps (`approve_*`) are agentive — the verifying agent calls `sdlc artifact approve` directly. No confirmation loops. This applies equally to living documents — `VISION.md`, `ARCHITECTURE.md`, and all `.sdlc/` artifacts are written directly by agents without staging for review. Git is the undo button.

**Fire and iterate, not gate and wait.** Never add a review step where an agent can just act. If an action might produce imperfect output, that's fine — wrong is fixable, blocked is not. The cost of a bad write is a second run. The cost of a confirmation loop is friction that compounds across every feature, every milestone, every day.

**Trust the state machine.** The classifier output is always the authoritative source of what to do next. Agents don't guess at phases or decide which artifact to write — they read `sdlc next --for <slug> --json` and execute exactly what it says.

**Milestones carry their own definition of done.** Every milestone has a `vision` (why it matters, what a user can do when it ships) and an `acceptance_test.md` (the runnable scenario that proves it's done). Agents use `sdlc milestone info <slug> --json` to read both.

**Everything in git.** All state — features, milestones, artifacts, UAT results — lives in `.sdlc/` as plain files and is committed to the project repository. There is no external database or service. Git is the audit trail, the history, and the backup.

**User perspectives are first-class.** The state machine ensures we build things right. User perspectives ensure we build the right things. `/sdlc-pressure-test <milestone>` runs empathy interviews against a milestone's scope and autonomously edits vision, features, acceptance criteria, and creates `[user-gap]` tasks. Use `/recruit` to assemble the right team for any project function — it always includes user perspectives.

## How Agents Use sdlc

**Explore an idea before committing to milestones:**
```bash
/sdlc-ponder [slug or new idea]
```

**Crystallize a pondered idea into milestones and features:**
```bash
/sdlc-ponder-commit <slug>
```

**One step at a time:**
```bash
/sdlc-next <slug>
```

**Full autonomous run to completion:**
```bash
/sdlc-run <slug>
```

**Pressure-test direction against user needs:**
```bash
/sdlc-pressure-test <milestone-slug>
```

The agent reads `sdlc next --for <slug> --json`, executes the action, and loops until `done`.

**The contract agents must honor:**
- Phases advance from artifact state, not direct transition calls. Agents call `sdlc artifact draft` then `sdlc artifact approve` — the machine derives the phase.
- `sdlc feature transition` is for setup and recovery only, never for normal execution.
- `sdlc next --for <slug> --json` is the oracle — always authoritative.

See `AGENTS.md` for the full consumer-facing agent instruction set (mental model, CLI reference, invariants, recovery protocol). That file is what gets installed in projects that consume sdlc.

## Agentive Template System

`sdlc init` and `sdlc update` install slash commands to user home directories for four AI coding CLIs. Commands are embedded as Rust `const &str` in `crates/sdlc-cli/src/cmd/init.rs` and written by `install_user_scaffolding()`.

| Platform | Location | Format |
|---|---|---|
| Claude Code | `~/.claude/commands/sdlc-*.md` | Markdown with frontmatter (`description`, `argument-hint`, `allowed-tools`) |
| Gemini CLI | `~/.gemini/commands/sdlc-*.toml` | TOML (`description` + `prompt`) — concise playbook variant |
| OpenCode | `~/.opencode/command/sdlc-*.md` | Markdown with frontmatter — concise playbook variant |
| Agents (generic) | `~/.agents/skills/sdlc-*/SKILL.md` | SKILL.md (Agent Skills open standard) — minimal variant |

**Current commands (Claude Code slash commands):**

| Command | Purpose |
|---|---|
| `/sdlc-ponder [slug]` | Open ideation workspace — explore ideas, capture artifacts, recruit thought partners |
| `/sdlc-ponder-commit <slug>` | Crystallize pondered idea into milestones/features via `/sdlc-plan` |
| `/sdlc-recruit <role>` | Recruit an expert thought partner as a persistent agent |
| `/sdlc-empathy <subject>` | Deep user perspective interviews before making decisions |
| `/sdlc-next <slug>` | Execute one directive step |
| `/sdlc-run <slug>` | Autonomous run to completion |
| `/sdlc-status` | Project overview |
| `/sdlc-plan` | Distribute a plan into milestones, features, tasks |
| `/sdlc-prepare <milestone>` | Pre-flight milestone — align features with vision, fix gaps, write wave plan |
| `/sdlc-run-wave <milestone>` | Execute Wave 1 features in parallel, advance to next wave |
| `/sdlc-pressure-test <milestone>` | Pressure-test milestone against user perspectives |
| `/sdlc-milestone-uat <milestone>` | Run acceptance test for a milestone |
| `/sdlc-enterprise-readiness` | Production readiness analysis |
| `/sdlc-setup-quality-gates` | Set up pre-commit hooks |
| `/sdlc-quality-fix` | Fix failing quality-check results — triages by failure count and applies fix-forward / fix-all / remediate |

**Adding a command:** Add a `const SDLC_*_COMMAND: &str` (Claude format), `const SDLC_*_PLAYBOOK: &str` (Gemini/OpenCode), and `const SDLC_*_SKILL: &str` (Agents). Register in all four `write_user_*` functions. Add filenames to `migrate_legacy_project_scaffolding()`.

**Changing CLI commands:** If you add, rename, or change the arguments of any `sdlc` subcommand, update the command reference table in `GUIDANCE_MD_CONTENT` (§6 "Using sdlc") in `init.rs`. That table is the single source of truth agents read before acting — stale entries cause agents to call nonexistent commands or skip new ones.

**Legacy migration:** `migrate_legacy_project_scaffolding()` removes old project-level `.claude/commands/sdlc-*.md` files (and equivalents for Gemini, OpenCode, `.agents/`, `.codex/`) since commands are now user-level.

**AGENTS.md:** Managed via `<!-- sdlc:start -->` / `<!-- sdlc:end -->` markers — safe for in-place updates without clobbering project-specific content.

See `docs/ai-cli-compatibility.md` for the full cross-tool compatibility reference.

## Adding a Rule

Edit `crates/sdlc-core/src/rules.rs`. Rules are evaluated in order — highest priority first. Use the `rule!` macro. Add tests.

If adding a new `ActionType`, update `ActionType::all()` in `types.rs` — tests will fail until updated (this is intentional).

## Coding Conventions

- No `unwrap()` in library code — use `?` and `SdlcError`
- All file writes go through `crates/sdlc-core/src/io.rs` (atomic writes)
- JSON output via `print_json()`, table output via `print_table()` in `sdlc-cli/src/output.rs`
- Integration tests use `tempfile::TempDir` for isolated `.sdlc/` directories
- No refresh buttons in the UI — state updates automatically via SSE (`/api/events`); add SSE to any hook that fetches project/feature state
- Utility copy buttons (clipboard) are always present on command code blocks so agents can copy `/sdlc-run` and `/sdlc-next` commands in one click
- All `/sdlc-*` commands end output with `**Next:** <command>` — one concrete next command, always present
- `/sdlc-*` commands must orchestrate real work (multiple steps, decisions, synthesis) — a command that wraps a single CLI call is not a command, it's noise; delete it and fold the call into the preceding command's `**Next:**` output
