# sdlc Vision

Enterprises are deploying hundreds of self-operating AI services. The failure mode
isn't bad AI — it's governance chaos. sdlc is the deterministic governance layer
that makes autonomous services auditable, compliant, and controllable at scale.

> **Every service knows exactly where it is, what comes next, and what it has
> already done — without asking a human.**

---

## The Problem

Enterprise software organizations are entering a new era: AI agents that write,
deploy, and operate software autonomously. A Fortune 500 running 1,000 services
can't have a human in every loop. But they also can't have 1,000 services doing
whatever they want — compliance teams, security reviews, and release audits exist
for reasons that don't disappear because AI is involved.

The existing tools fail in both directions. Jira and Linear assume humans read
and act on state. GitHub Actions assumes code, not lifecycle. Custom scripts
don't generalize. None of them produce the structured, machine-readable directives
that AI agents can act on deterministically across thousands of services simultaneously.

The result: enterprise AI programs stall not because the AI is bad, but because
there's no governance layer it can actually talk to.

## The Answer

sdlc is a deterministic state machine that any AI agent — or orchestrator — can
query and act on. It tracks features through a structured lifecycle, emits precise
directives (`sdlc next --for <slug> --json`), records every approval and transition,
and stores everything in git — making the audit trail native to the tool.

The execution model is the **next-tick orchestrator**: a scheduler calls `sdlc next`
for every service on a heartbeat, receives a directive, dispatches an agent to act
on it, and loops. At enterprise scale, this means thousands of services advancing
autonomously in parallel — each one governed by the same deterministic rules, each
one auditable in git.

The key insight: the state machine has no opinions about who or what executes
the work. AI agents, orchestrators, scripts, and humans all speak the same directive
interface. Enterprise compliance gets a clean audit log. The orchestrator gets
unambiguous next steps. The team gets a shared source of truth that scales to
thousands of services.

## Core Design Principles

### 1. Directives, Not Dashboards

The primary interface is machine-readable JSON (`sdlc next --json`), not a UI.
Humans get the UI as a view — agents and orchestrators get the directive as their
source of truth. This ensures AI consumers are never second-guessing state.

### 2. Git Is the Audit Trail

All state lives in `.sdlc/` as plain files, committed to git. There is no external
database to go down, no service to lose data, no vendor to audit. The compliance
story is: `git log`.

### 3. Determinism at Scale

The classifier is pure Rust — given the same inputs, it always produces the same
output. 1,000 services behave identically to 1. This is the property that makes
enterprise governance possible: no special cases, no drift, no surprises.

### 4. The State Machine Is Dumb by Design

sdlc stores state and emits directives. It does not make decisions. All heuristics,
interpretations, and judgment live in the agents and orchestrators that consume it.
This separation means the governance layer never needs to change when agent behavior
improves.

### 5. Always Forward

Issues are captured as tasks, never as blockers that halt the machine. A service
that hits a problem creates a task and keeps moving. Blocked is always worse than
imperfect.

## What This Is Not

- **Not a project management tool for humans.** Jira already exists.
- **Not an agent framework.** sdlc doesn't run agents — it governs what they do.
- **Not a deployment system.** It tracks lifecycle state, not infrastructure.

## Success Criteria

- A Fortune 500 engineering platform team can onboard 100 self-operating services
  onto sdlc in a week, because the directive interface requires zero human
  configuration per service.
- A compliance officer can produce a full lifecycle audit trail for any service by
  running `git log .sdlc/features/<slug>/`, because all state lives in plain files.
- An AI agent or orchestrator running `sdlc next --for <slug> --json` always knows
  exactly what to do next with no ambiguity, because the classifier is deterministic
  and complete.
- A next-tick orchestrator can drive 1,000 services in parallel without coordination
  overhead, because each `sdlc next` call is stateless and independent.
- Jordan can ship 100 services in 3 months, because the state machine handles
  governance and he handles product.
