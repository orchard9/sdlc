# sdlc Vision

Deterministic SDLC orchestration for autonomous software development.

> **Operating philosophy:** Every design decision in sdlc traces back to one principle — the state machine and the AI are separate concerns. The binary knows nothing about agents. The orchestrator knows nothing about state. This separation makes both testable, replaceable, and trustworthy.

---

## The Problem

AI coding agents are powerful but untethered. They implement features without specs, skip review, ignore test failures, and repeat the same architectural mistakes across features. There's no shared memory, no quality enforcement, no lifecycle discipline.

The result: agents that write a lot of code but ship very little.

## The Answer

A deterministic state machine that agents operate through — not around. Every feature follows the same lifecycle. Every phase requires approved artifacts. Every action emits structured output that orchestrators consume. The agent is a worker. The state machine is the foreman.

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```

The classifier (`sdlc next --json`) tells any orchestrator exactly what to do next. The orchestrator dispatches the right agent. The agent writes the artifact. The gate checks it. The state advances.

No shortcuts. No skipped phases. No rubber-stamped reviews.

---

## Core Design Principles

### 1. The Binary Knows Nothing About Agents

`sdlc` is a state machine. It reads YAML, writes YAML, and evaluates rules. It has no LLM calls, no HTTP clients, no agent SDKs. This makes it:
- **Fast** — state operations take milliseconds
- **Reliable** — no network failures, no rate limits
- **Testable** — 200+ pure unit and integration tests
- **Portable** — works with any agent backend (Claude, Gemini, GPT, human)

### 2. Forward-Only Lifecycle

Features move forward through phases. The classifier blocks backward movement by requiring artifacts to be present and approved before advancing. If a review fails, you don't go back to draft — you fix the review issues and re-submit.

This mirrors how real software ships: you don't un-ship a feature. You fix it.

### 3. Artifacts as Contracts

Every phase transition requires a specific Markdown artifact, written by an agent or human, approved by a human or gate. Artifacts are the contract between phases:
- `spec.md` — what we're building and why (required for `specified`)
- `design.md` — how we're building it (required for `planned`)
- `tasks.md` — decomposed implementation units (required for `planned`)
- `review.md` — code review findings with quality scores (required for `review`)

Artifacts live in `.sdlc/features/<slug>/` and are committed to git. The history of a feature's development is the history of its artifacts.

### 4. The Classifier Is the Orchestrator Interface

`sdlc next --for <slug> --json` is the single interface between state and orchestration:

```json
{
  "feature": "auth-login",
  "action": "create_spec",
  "message": "No spec exists. Write the feature specification...",
  "output_path": ".sdlc/features/auth-login/spec.md",
  "is_heavy": false,
  "current_phase": "draft"
}
```

Every orchestrator — whether it's a Python script, Claude Code skill, web UI, or human reading the terminal — consumes this output to decide what to do next. The classifier evaluates priority-ordered rules and emits the highest-priority action.

### 5. Verification Gates Are Not Optional

Quality gates are mechanical. They run shell commands and block phase transitions until they pass. An agent that writes code that doesn't compile does not advance. An agent that writes a spec that a human hasn't approved does not advance.

The gate system is what makes agents accountable. Without it, agents optimize for producing output. With it, they optimize for producing *correct* output.

### 6. State Is Committed to Git

`.sdlc/state.yaml`, `.sdlc/config.yaml`, and all feature artifacts live in the project repo. This means:
- Feature history is version-controlled
- Teams share state without a database
- State is auditable and rollback-able
- No SaaS dependency, no network requirement

---

## The Orchestration Model

`sdlc` is the state layer. The orchestration layer sits above it:

```
┌─────────────────────────────────────────────────────┐
│  Orchestrator (sdlc_driver / web UI / Claude skill) │
│  Nested verification loops with decision gates      │
│  Modes: auto | guided | advise                      │
└──────────────────────┬──────────────────────────────┘
                       │ sdlc next --json
                       ▼
┌─────────────────────────────────────────────────────┐
│  sdlc Binary (Rust)                                 │
│  Pure state machine — no LLM awareness              │
│  .sdlc/state.yaml + .sdlc/features/{slug}/          │
└──────────────────────┬──────────────────────────────┘
                       │ config-driven dispatch
                       ▼
┌─────────────────────────────────────────────────────┐
│  Agent Backends                                     │
│  Claude Agent SDK │ xadk (Gemini) │ human           │
└─────────────────────────────────────────────────────┘
```

The orchestrator loop:

```
1. sdlc next --for <slug> --json        → get next action
2. action == Done?                      → exit
3. action is human gate?                → pause, print instructions
4. dispatch agent with enriched context
5. run verification gates
6. gate fail + retries left?            → re-dispatch with error context
7. gate fail + no retries?              → pause for human
8. all gates pass?                      → loop
```

This separation means you can swap the orchestrator without touching the state machine, and swap the agent backend without touching either.

---

## Orchestration Modes

| Mode | Behavior |
|---|---|
| `advise` | Print what would happen next, exit. Human drives everything. |
| `guided` | Show dry-run preview before each step, wait for human confirmation. |
| `auto` | Run loop until human gate or Done. Mechanical checks auto-verified. |

Human approval gates (`approve_spec`, `approve_design`, `approve_review`, `approve_merge`) always block regardless of mode.

---

## The Feature Lifecycle in Detail

### Phases and Required Artifacts

| Phase | Required to Enter | What Gets Written |
|---|---|---|
| `draft` | — | Feature manifest created |
| `specified` | `spec.md` approved | Feature specification |
| `planned` | `spec.md` + `design.md` + `tasks.md` + `qa-plan.md` approved | Architecture and task breakdown |
| `ready` | All planning artifacts approved | Ready for implementation |
| `implementation` | — | Code written per tasks |
| `review` | `review.md` approved | Code review with scores |
| `audit` | `audit.md` approved | Three-lens audit report |
| `qa` | `qa-results.md` approved | Acceptance test results |
| `merge` | Merge approval gate passed | — |
| `released` | — | Feature complete |

### Quality Scoring

Reviews and audits include three-lens quality scores:
- **Product fit** — does it solve the user's problem?
- **Research grounding** — is it based on verified information?
- **Implementation** — is the code correct and production-ready?

Scores are numeric (0–100). Phase transitions can be blocked below configurable thresholds.

```bash
sdlc score set auth-login implementation 82
sdlc score show auth-login
```

---

## What sdlc Is Not

**Not a project management tool.** It doesn't replace Jira, Linear, or GitHub Issues. It's a build engine for a single project, not a portfolio tracker.

**Not an AI runtime.** It has no agent SDK, no prompt engineering, no LLM calls. It provides the state that agents operate against.

**Not a CI/CD system.** Verification gates run checks, but sdlc doesn't replace GitHub Actions or Buildkite. Gates are for in-loop quality enforcement; CI is for post-push validation.

**Not opinionated about agents.** Any agent that can read JSON and write Markdown files can operate against sdlc. Claude, Gemini, GPT, or a shell script — the interface is the same.

---

## Why Rust?

- **No dependencies in production** — single binary, no runtime, no interpreter
- **Fast** — state operations are instant; no perceived latency in the orchestration loop
- **Reliable** — the type system makes invalid state transitions compile errors
- **Portable** — one binary works on macOS, Linux, and Windows
- **Embeddable** — `sdlc-core` is a library; `sdlc-server` reuses it directly without subprocess overhead
