---
session: 1
timestamp: 2026-03-02T00:00:00Z
orientation:
  current: "Tool design shaped — priority waterfall, flight lock, async dispatch, default 4h action"
  next: "Decide: async spawn vs block execution model, then commit to building"
  commit: "Resolve execution model question + confirm dry_run is default on first run"
---

## Session 1 — Dev Driver: stock tool + default action

**Participants:** Jordan (user), Priya Nair (Distributed Systems), Tobias Krenn (Skeptic), Felix Wagner (Developer Tooling)

---

### Brief

Jordan wants two things:
1. A stock sdlc tool that "finds and does the next thing that should be done" in a development context
2. A default action that drives development forward automatically

The tool might: run a wave, implement a task, suggest features, audit the codebase, or wait.

---

### Initial interrogation

The real problem: work only happens when a human fires a command. The dev-driver closes the gap between "project has work to do" and "work gets done." It's an autonomous scheduler that sits on top of `sdlc next` and the existing skill commands.

---

**Priya Nair · Distributed Systems**

Immediately flagged concurrency. The tool runs on a tick schedule. If it kicks off `/sdlc-run-wave` which runs for 40+ minutes, the next tick could dispatch a second overlapping agent. File writes collide. State corrupts.

> "The tool needs a flight lock — write `.sdlc/.dev-driver.lock` when you start. Check it before doing anything. If the lock is < 2h old, exit with 'waiting'. Classic cron lock pattern."

⚑ Decided: flight lock is mandatory. Path: `.sdlc/.dev-driver.lock`

---

**Tobias Krenn · Skeptic**

Challenged scope. "Run a wave, implement a task, suggest features, audit the codebase — that's five tools in a trenchcoat. What's the single thing 90% of use cases need?"

His answer: just advance the highest-priority feature with a pending directive. That's it. Feature suggestion and codebase auditing are different categories of work — pull them out of v1.

Also: "Who watches this thing? If it makes bad decisions at 3am, how does the developer know? You need observability before capability. Log what it decided AND why."

⚑ Decided: v1 is a single-mode advance tool only. Suggestion/auditing are future expansions.

---

**Felix Wagner · Developer Tooling**

Sketched the priority waterfall:
1. Failing quality checks → report, exit
2. Flight lock exists → wait, exit
3. Features with active directives → pick highest priority, dispatch
4. Wave ready to start → dispatch wave
5. Nothing → idle

Each level has a clear exit condition. One action per invocation. Recurrence handles re-entry.

⚑ Decided: priority waterfall as the governing logic. One action per invocation.

---

### Core design sketch

**Tool name:** `dev-driver`
**Input:** `{ dry_run?: boolean }`
**Output:** `{ action: 'waiting' | 'quality_failing' | 'feature_advanced' | 'wave_started' | 'idle', ... }`

**Default action:**
```yaml
label: dev-driver
tool_name: dev-driver
tool_input: {}
recurrence_secs: 14400   # 4 hours
```

---

### Open execution model question

**Priya's challenge:** If the tool does `execSync('claude --print "/sdlc-run feature-x"')` it blocks for up to 60min on waves. It will time out.

Two options:
- **Option A: Async spawn** — fire Claude, immediately exit. Lock persists until Claude finishes (removes it as last action). Fast exit, fire-and-forget.
- **Option B: Short-horizon only** — tool only reads state and outputs a structured decision; doesn't actually dispatch. Pushes the problem upstream.

? Open: async spawn (fire-and-forget + lock) vs. direct execution (block + timeout risk)?

Lean: Option A (async spawn) gives real value delivery. Option B is a half-measure.

---

### Artifacts captured

- `priority-waterfall.md` — 5-level decision tree with exclusions
- `design-sketch.md` — tool contract, flight lock spec, execution model, default action config

---

### What's clear

- The tool is a priority scheduler on top of `sdlc next` + existing skill commands
- One action per invocation — the recurrence handles advancement over time
- Flight lock is the key safety mechanism
- Ships as a stock tool via `sdlc tool scaffold dev-driver` (like quality-check, ama)
- Default action: every 4 hours, `{}` input, label `dev-driver`
- Dry-run mode shows what it would do without dispatching — recommended as default on first use

### What remains unresolved

1. Async spawn vs block (execution model)
2. Should dry_run be the default until explicitly opted out?
3. Feature tie-breaking: what's "highest priority" when 3 features are in IMPLEMENTATION?
4. Lock format: just timestamp, or include what's running?
5. Does this ship as part of `sdlc init` scaffolding, or as a separate scaffold command?
