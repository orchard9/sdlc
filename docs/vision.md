# sdlc Vision

Deterministic project management for autonomous software development.

> **Operating philosophy:** Every design decision in sdlc traces back to one principle — the state machine and the consumer are separate concerns. The binary knows nothing about agents. The consumer knows nothing about state. This separation makes both testable, replaceable, and trustworthy.

---

## The Problem

AI coding agents are powerful but untethered. They implement features without specs, skip review, ignore test failures, and repeat the same architectural mistakes across features. There's no shared memory, no quality enforcement, no lifecycle discipline.

The result: agents that write a lot of code but ship very little.

## The Answer

A deterministic state machine that tracks every feature through a structured lifecycle. Every phase requires approved artifacts. Every action emits a structured directive that any consumer — an AI agent, a script, a human — can act on.

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```

`sdlc next --json` tells the consumer exactly what to do next. The consumer decides how to do it. sdlc records what was approved and advances the phase. sdlc is a ledger — not a foreman.

Follow the lifecycle defaults by default. Use explicit overrides when needed. Keep reviews intentional.

---

## Core Design Principles

### 1. The Binary Knows Nothing About Agents

`sdlc` is a state machine. It reads YAML, writes YAML, and evaluates rules. It has no LLM calls, no HTTP clients, no agent SDKs. This makes it:
- **Fast** — state operations take milliseconds
- **Reliable** — no network failures, no rate limits
- **Testable** — 200+ pure unit and integration tests
- **Portable** — works with any consumer (Claude, Gemini, GPT, human)

### 2. Forward-Progress Lifecycle

Features are designed to move forward through phases. The classifier gives a default next action based on artifacts and approvals so teams have a clear, reviewable path. If a review fails, you typically fix the review issues and re-submit rather than restarting the feature.

This mirrors how real software ships: you don't un-ship a feature. You fix it.

### 3. Artifacts as Contracts

Artifacts define the contract between phases. They are Markdown files written by an agent, then verified by another agent pass before the phase advances. All artifact verification passes (`approve_*` actions) are executed agentively. The agent that writes an artifact is different from the agent that verifies it — separation creates accountability, not human checkpoints.

- `spec.md` — what we're building and why (required for `specified`)
- `design.md` — how we're building it (required for `planned`)
- `tasks.md` — decomposed implementation units (required for `planned`)
- `review.md` — code review findings with quality scores (required for `review`)

The `approve_*` actions in the state machine are agent-verification steps, not human prompts. The agent that writes the artifact is different from the agent that verifies it — separation creates accountability.

Artifacts live in `.sdlc/features/<slug>/` and are committed to git. The history of a feature's development is the history of its artifacts.

### 4. The Classifier Is the Directive Interface

`sdlc next --for <slug> --json` is the single interface between state and consumers:

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

Every consumer — whether it's a Python script, Claude Code skill, web UI, or human reading the terminal — reads this output to decide what to do next. The classifier evaluates priority-ordered rules and emits the highest-priority action.

### 5. Verification Gates Are Consumer Hints

Quality gates are published in the directive output as metadata. The consumer decides whether and how to run them. An agent that writes code that doesn't compile should not be advanced. An agent that writes a spec that hasn't passed verification should not be advanced.

The gate system is what makes agents accountable. Without it, agents optimize for producing output. With it, they optimize for producing *correct* output.

### 6. User Perspectives Are First-Class

The state machine ensures we build things *right*. User perspectives ensure we build the *right things*. Both are required.

Milestone visions are written in the user's voice: "A [specific person] can [specific action], which matters because [specific value]." Acceptance tests include user-perspective checks alongside technical checks. The `/sdlc-pressure-test` command runs empathy interviews against a milestone's scope and autonomously sharpens vision, feature descriptions, acceptance criteria, and creates `[user-gap]` tasks for anything the planned work doesn't address.

User perspective is not a phase — it's a lens applied at planning time (shaping what to build) and at QA time (verifying we built what users need). The `product_fit` quality score captures this quantitatively. The pressure test captures it qualitatively.

### 7. State Is Committed to Git

`.sdlc/state.yaml`, `.sdlc/config.yaml`, and all feature artifacts live in the project repo. This means:
- Feature history is version-controlled
- Teams share state without a database
- State is auditable and rollback-able
- No SaaS dependency, no network requirement

---

## The Directive Model

`sdlc` is the state layer. The consumer layer sits above it:

```
┌─────────────────────────────────────────────────────┐
│  Consumer (Claude Code / web UI / script / human)   │
│  Reads directive, acts on it, submits artifacts     │
└──────────────────────┬──────────────────────────────┘
                       │ sdlc next --json
                       ▼
┌─────────────────────────────────────────────────────┐
│  sdlc Binary (Rust)                                 │
│  Pure state machine — no LLM awareness              │
│  .sdlc/state.yaml + .sdlc/features/{slug}/          │
└─────────────────────────────────────────────────────┘
```

The consumer pattern:

```
1. sdlc next --for <slug> --json   → read the directive
2. action == done?                 → exit
3. action == HITL gate?            → surface to human, exit
   (wait_for_approval, unblock_dependency)
4. execute the action agentively   → write artifact, implement, or verify+approve
5. repeat
```

**HITL is about gating the loop, not approving individual steps.**

### Quality Standard and Approach

Every directive carries two explicit standards that govern how consumers execute work:

**Quality bar:** The Steve Jobs standard — the right solution over the expedient one. No known debt is shipped. Agents are expected to take the harder path when it produces better outcomes, not the faster path that produces working-but-wrong results.

**Approach orientation:** Structural before detail. Agents scale their effort to the complexity of the action — understanding the full system before touching it, planning before implementing, refactoring structure before patching detail. Simple changes may proceed directly; anything non-trivial requires a planning pass first.

These are encoded in every directive as `**Standard:**` and `**Approach:**` lines in the directive header, visible before the task description. The human decides when to run the next step by invoking `sdlc-next` or `sdlc-run`. Inside a step, agents execute without interruption — including verification passes that result in `sdlc artifact approve` calls.

sdlc doesn't know or care what consumes its directives — Claude, Gemini, a shell script, or a human reading the terminal. The interface is always the same structured JSON.

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

**Not an AI dispatcher.** sdlc does not call agents, spawn subprocesses, or route work to AI backends. It emits structured directives (`sdlc next --json`) that consumers act on. The consumer decides what to run and how.

**Not an AI runtime.** It has no agent SDK, no prompt engineering, no LLM calls. It provides the state that consumers operate against.

**Not a CI/CD system.** Gates are consumer hints, not enforcement mechanisms. sdlc doesn't replace GitHub Actions or Buildkite.

**Not opinionated about consumers.** Any consumer that can read JSON and write Markdown files can work with sdlc. Claude, Gemini, GPT, a shell script, or a human — the interface is the same.

---

## Why Rust?

- **No dependencies in production** — single binary, no runtime, no interpreter
- **Fast** — state operations are instant; no perceived latency in the directive loop
- **Reliable** — the type system makes invalid state transitions compile errors
- **Portable** — one binary works on macOS, Linux, and Windows
- **Embeddable** — `sdlc-core` is a library; `sdlc-server` reuses it directly without subprocess overhead
