# sdlc Architecture

Technical reference for contributors and integrators.

---

## Stack

| Layer | Technology | Notes |
|---|---|---|
| Language | Rust (stable) | Determinism, performance, no runtime |
| State storage | YAML files in `.sdlc/` | No database — git is the audit trail |
| CLI | `sdlc-cli` crate | Binary distributed as single executable |
| HTTP server | `sdlc-server` crate (Axum) | Embedded frontend, SSE for live state |
| Frontend | React + Vite | Compiled into `sdlc-server` binary at build time |
| Agent interface | `sdlc next --json` | Directive protocol for all AI consumers |

## Workspace Layout

```
sdlc/
├── crates/
│   ├── sdlc-core/        — State machine, classifier, types, I/O primitives
│   ├── sdlc-cli/         — `sdlc` binary, all subcommands
│   └── sdlc-server/      — HTTP server + SSE + embedded React UI
├── frontend/             — React + Vite source (compiled into sdlc-server)
├── .sdlc/                — Live project state (YAML + Markdown artifacts)
│   ├── features/<slug>/  — Per-feature artifacts (spec.md, design.md, etc.)
│   ├── milestones/       — Milestone manifests
│   ├── roadmap/<slug>/   — Ponder ideation workspace
│   └── config.yaml       — Gates, quality thresholds, platform commands
└── docs/                 — Architecture, vision, design docs
```

## Key Components

**sdlc-core** — The state machine. Contains `rules.rs` (priority-ordered classifier
rules), `classifier.rs` (rule engine), `feature.rs` (artifact management, phase
transitions), `types.rs` (Phase, ArtifactType, ActionType enums). No binary — pure
library. Never makes decisions; stores and classifies state only.

**sdlc-cli** — The `sdlc` binary. All subcommands live in `src/cmd/`. Key entry
point: `cmd/next.rs` which calls the classifier and formats directives. Also manages
`sdlc init` / `sdlc update` — installs agent scaffolding (slash commands, skills)
across Claude Code, Gemini CLI, OpenCode, and Codex.

**sdlc-server** — Axum HTTP server. Routes under `src/routes/`. Embeds the compiled
frontend via `build.rs`. Streams state updates to the browser via SSE (`/api/events`).
All agent-invoked operations use `spawn_agent_run` in `routes/runs.rs` — never raw
`tokio::spawn`.

**Classifier** — Given feature state (which artifacts exist, their approval status,
open comments), `classify()` returns the next `ActionType`. Rules are priority-ordered
in `rules.rs`. Pure function — deterministic, no I/O, no side effects.

**Directive Interface** — `sdlc next --for <slug> --json` is the contract that all
AI consumers depend on. JSON output includes: `action`, `message`, `output_path`,
`is_heavy`, `gates`. This interface must remain stable; breaking it breaks every
agent and orchestrator that consumes it.

## Data Flow

### Single service (interactive / manual)

```
AI Agent / CLI User
       │
       ▼
sdlc next --for <slug> --json
       │
       ▼
 Classifier (sdlc-core)
  reads .sdlc/features/<slug>/
  evaluates rules in priority order
  returns ActionType + directive
       │
       ▼
Agent acts: writes artifact, runs task, calls approve
       │
       ▼
sdlc artifact approve <slug> <type>
  writes .sdlc/features/<slug>/<artifact>.md
  updates YAML state
       │
       ▼
Next classify() call returns next action
```

### Next-tick orchestrator (enterprise / autonomous)

```
Orchestrator (heartbeat)
       │
       ├── sdlc next --for service-001 --json  → directive → dispatch agent
       ├── sdlc next --for service-002 --json  → directive → dispatch agent
       ├── sdlc next --for service-003 --json  → done      → skip
       │   ...
       └── sdlc next --for service-N   --json  → directive → dispatch agent

Each agent acts independently:
  reads directive → writes artifact → calls approve → exits

Orchestrator ticks again:
  same services advance, done services stay done, new directives dispatched
```

Each `sdlc next` call is stateless and independent — no coordination required
between services. The orchestrator needs no knowledge of the state machine; it
only needs to dispatch on the directive it receives.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| No database | YAML in git | Audit trail is native; zero ops; works offline; scales per-repo |
| No LLM in sdlc | Directive-only | State machine never drifts; agent logic is swappable |
| Rust for core | Determinism + perf | Classifier must behave identically for 1 or 10,000 services |
| Frontend embedded | Compiled into binary | Zero deployment complexity; single executable ships everything |
| SSE not polling | Server-sent events | UI stays live without refresh; works behind firewalls |
| Rules in Rust | Priority-ordered list | Explicit, testable, auditable — no hidden heuristics |
| Stateless classify() | Pure function | Safe to call in parallel for N services simultaneously |

## What to Read First

1. `crates/sdlc-core/src/rules.rs` — The complete state machine in one file
2. `crates/sdlc-core/src/types.rs` — Every enum that flows through the system
3. `crates/sdlc-cli/src/cmd/next.rs` — The directive interface implementation
4. `docs/plan-act-pattern.md` — Two-phase agent workflow (plan → act)
