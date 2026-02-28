# AGENTS.md

Agent instructions for sdlc.

<!-- sdlc:start -->

## SDLC

> **Required reading:** `.sdlc/guidance.md` — engineering principles that govern all implementation decisions on this project. <!-- sdlc:guidance -->

This project uses `sdlc` as its SDLC state machine. `sdlc` manages feature lifecycle, artifacts, tasks, and milestones. It emits structured directives via `sdlc next --json` that any consumer (Claude Code, custom scripts, or humans) acts on to decide what to do next.

Consumer scaffolding is installed globally under `~/.claude/commands/`, `~/.gemini/commands/`, `~/.opencode/command/`, and `~/.agents/skills/` — available across all projects. Use `/sdlc-specialize` in Claude Code to generate a project-specific AI team (agents + skills) tailored to this project's tech stack and roles.

### Key Commands

- `sdlc feature create <slug> --title "..."` — create a new feature
- `sdlc next --for <slug> --json` — get the next action directive (JSON)
- `sdlc next` — show all active features and their next actions
- `sdlc artifact approve <slug> <type>` — approve an artifact to advance the phase
- `sdlc state` — show project state
- `sdlc feature list` — list all features and their phases
- `sdlc task list [<slug>]` — list tasks for a feature (or all tasks)

### Lifecycle

draft → specified → planned → ready → implementation → review → audit → qa → merge → released

Treat this lifecycle as the default pathway. You can use explicit manual transitions when needed, but approvals/artifacts are the recommended way to keep quality and traceability.

### Artifact Types

`spec` `design` `tasks` `qa_plan` `review` `audit` `qa_results`

### CRITICAL: Never edit .sdlc/ YAML directly

All state changes go through `sdlc` CLI commands. See §6 of `.sdlc/guidance.md` for the full command reference. Direct YAML edits corrupt state.

### Directive Interface

Use `sdlc next --for <slug> --json` to get the next directive. The JSON output tells the consumer what to do next (action, message, output_path, is_heavy, gates).

### Consumer Commands

- `/sdlc-next <slug>` — execute one step, then stop (human controls cadence)
- `/sdlc-run <slug>` — run autonomously until a HITL gate or completion
- `/sdlc-status [<slug>]` — show current state
- `/sdlc-plan` — distribute a plan into milestones, features, and tasks
- `/sdlc-milestone-uat <milestone-slug>` — run the acceptance test for a milestone
- `/sdlc-pressure-test <milestone-slug>` — pressure-test a milestone against user perspectives
- `/sdlc-enterprise-readiness [--stage <stage>]` — analyze production readiness
- `/sdlc-setup-quality-gates` — set up pre-commit hooks and quality gates
- `/sdlc-cookbook <milestone-slug>` — create developer-scenario cookbook recipes
- `/sdlc-cookbook-run <milestone-slug>` — execute cookbook recipes and record results
- `/sdlc-ponder [slug]` — open the ideation workspace for exploring and committing ideas
- `/sdlc-ponder-commit <slug>` — crystallize a pondered idea into milestones and features
- `/sdlc-recruit <role>` — recruit an expert thought partner as a persistent agent
- `/sdlc-empathy <subject>` — deep user perspective interviews before decisions

Project: sdlc

<!-- sdlc:end -->

## Mental Model

The sdlc state machine derives the current phase from artifact state. **You never advance phases manually — you advance artifacts. The machine computes everything else.**

Three things happen in order for any artifact:
1. You write the file to `.sdlc/features/<slug>/<type>.md`
2. You call `sdlc artifact draft <slug> <type>` — this tells the machine the artifact exists
3. You call `sdlc artifact approve <slug> <type>` — this advances the phase

Skipping step 2 means the machine has no record of the artifact. `sdlc next` will still return `create_<type>` regardless of what you wrote to disk.

**`sdlc next --for <slug> --json` is always right.** When you're unsure of the current state, call it. The machine's view is authoritative — your mental model of "I just approved X" is not.

---

## Invariants

These must never be violated:

- **Draft before approve** — never call `sdlc artifact approve` without first calling `sdlc artifact draft` in the same action sequence
- **Directives drive action** — the `action` field from `sdlc next --for <slug> --json` tells you exactly what the machine needs next; execute it, then call `sdlc next` again
- **Phases are read-only** — `sdlc feature transition` exists for setup and debugging; never call it during a feature run; artifact approvals drive transitions automatically

---

## CLI Reference

Exact subcommands. Anything not listed here does not exist — do not guess.

| Namespace | Subcommands |
|---|---|
| `sdlc artifact` | `draft` · `approve` · `reject` |
| `sdlc feature` | `create` · `list` · `show` · `transition` · `archive` |
| `sdlc task` | `add` · `list` · `start` · `complete` |
| `sdlc comment` | `create` · `list` · `resolve` |
| `sdlc score` | `set` |
| `sdlc next` | _(no subcommands — use `--for <slug>` and `--json` flags)_ |
| `sdlc ponder` | `create` · `list` · `show` · `capture` · `team add` · `team list` · `update` · `archive` · `artifacts` |

---

## Recovery Protocol

When a command fails or you're unsure of your position:

1. Call `sdlc next --for <slug> --json` — read the current `action`
2. Execute exactly that action, nothing else
3. Call `sdlc next --for <slug> --json` again to confirm state advanced

**Common errors and causes:**

| Error | Cause |
|---|---|
| `cannot transition` | An artifact hasn't been drafted or approved yet — check `sdlc next` to find which one |
| `unrecognized subcommand` | You used a command that doesn't exist — check the CLI Reference above |
| `sdlc next` keeps returning same `action` after you acted | You wrote the file but forgot to call `sdlc artifact draft` |

---

## Never Do

- Do NOT call `sdlc feature transition` during a feature run
- Do NOT call `sdlc artifact approve` without first calling `sdlc artifact draft`
- Do NOT write an artifact file without calling `sdlc artifact draft` afterward — the machine cannot see files you wrote
- Do NOT skip `sdlc next` between actions — always re-read the directive after each action

---

## Key Commands

```bash
sdlc feature create <slug> --title "..."   # create a new feature
sdlc next --for <slug> --json              # get next action directive
sdlc next                                  # all features and their next actions
sdlc artifact draft <slug> <type>          # mark artifact written (required before approve)
sdlc artifact approve <slug> <type>        # approve after verification passes
sdlc artifact reject <slug> <type>         # reject when verification fails
sdlc task add <slug> --title "..." --id T1 # register a task
sdlc task list <slug>                      # list tasks and their status
sdlc task start <slug> <task-id>           # mark task in progress
sdlc task complete <slug> <task-id>        # mark task done
sdlc comment create <slug> "<text>"        # add a comment or blocker
sdlc score set <slug> <dimension> <n>      # record quality score (0–100)
sdlc state                                 # show project state
sdlc feature list                          # list all features and phases
```

---

## Lifecycle

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```

---

## How Agents Drive the Lifecycle

The state machine emits one of three kinds of actions:

**Agent actions** — executed autonomously without human confirmation:

| Action | What the agent does |
|---|---|
| `create_spec` | Reads project vision and feature description, writes spec.md, calls `draft` |
| `approve_spec` | Reads spec.md, verifies quality, calls `approve` or `reject` |
| `create_design` | Reads spec, writes design.md, calls `draft` |
| `approve_design` | Reads design.md against spec, verifies soundness, calls `approve` or `reject` |
| `create_tasks` | Reads design, decomposes into implementation tasks, calls `draft` |
| `approve_tasks` | Reads tasks.md against design, verifies completeness, calls `approve` or `reject` |
| `create_qa_plan` | Reads spec + tasks, writes acceptance test scenarios, calls `draft` |
| `approve_qa_plan` | Reads qa-plan.md against spec, verifies coverage, calls `approve` or `reject` |
| `implement_task` | Reads tasks + design, implements next pending task, calls `task complete` |
| `fix_review_issues` | Reads review findings, fixes issues in code, re-submits with `draft` |
| `create_review` | Reads all changed code, writes evidence-based review.md with scores, calls `draft` |
| `approve_review` | Spot-checks review findings against code, calls `approve` or `reject` |
| `create_audit` | Runs three-lens audit (product fit, research grounding, implementation), calls `draft` |
| `approve_audit` | Verifies audit findings are accurate, calls `approve` or `reject` |
| `run_qa` | Executes qa-plan scenarios, writes qa-results.md, calls `draft` |
| `approve_merge` | Verifies QA results are acceptable, calls `approve` on qa_results |
| `merge` | Transitions feature to released |

**True HITL gates** — agent stops and surfaces to human:

| Gate | Condition |
|---|---|
| `wait_for_approval` | Blocker/question comment exists — human resolves before proceeding |
| `unblock_dependency` | Feature has an external blocker only a human can resolve |

**HITL is about gating the loop, not approving individual steps.** The human decides when to invoke `sdlc-next` (one step) or `sdlc-run` (full autonomous run to next gate). Agents execute all non-gate actions without interruption, including all verification passes.

---

## Consumer Commands

**Ideation (pre-feature):**
- `/sdlc-ponder [slug]` — open the ideation workspace; explore ideas, recruit thought partners, capture scrapbook artifacts
- `/sdlc-ponder-commit <slug>` — crystallize a pondered idea into milestones and features
- `/sdlc-recruit <role>` — recruit an expert thought partner as a persistent agent (usable independently or within ponder)
- `/sdlc-empathy <subject>` — deep user perspective interviews (usable independently or within ponder)

**Execution:**
- `/sdlc-next <slug>` — execute one step, then stop (human controls cadence)
- `/sdlc-run <slug>` — run autonomously until a HITL gate or completion
- `/sdlc-status [<slug>]` — show current state
- `/sdlc-plan` — distribute a plan into milestones, features, and tasks
- `/sdlc-milestone-uat <milestone-slug>` — run the acceptance test for a milestone
- `/sdlc-pressure-test <milestone-slug>` — pressure-test a milestone against user perspectives
- `/sdlc-enterprise-readiness [--stage <stage>] [--into <milestone>]` — analyze production readiness and distribute findings
- `/sdlc-setup-quality-gates` — set up pre-commit hooks and quality gates

---

## Artifact Types

`spec` `design` `tasks` `qa_plan` `review` `audit` `qa_results`

---

## Directive Interface

`sdlc next --for <slug> --json` returns:

```json
{
  "feature": "<slug>",
  "title": "...",
  "action": "<action_type>",
  "message": "...",
  "output_path": ".sdlc/features/<slug>/<file>.md",
  "is_heavy": false,
  "gates": []
}
```

Use `action` to route behavior. Use `message` as the prompt context for the action. Use `output_path` as the canonical file path to write.
