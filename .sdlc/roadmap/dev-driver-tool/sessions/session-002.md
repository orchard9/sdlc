---
session: 2
timestamp: 2026-03-02T01:00:00Z
orientation:
  current: "All design decisions resolved — ready to commit"
  next: "sdlc-ponder-commit dev-driver-tool to generate milestones and features"
  commit: "All 6 open questions answered — commit is unblocked"
---

## Session 2 — Resolved decisions

Jordan answered all 6 open questions in one pass. No remaining tensions.

---

### ⚑ Resolved: Execution model

**Async spawn.** The tool dispatches Claude Code with `spawn(..., { detached: true })` and exits immediately.
The lock file persists until the agent process finishes. Lock TTL (2h) is the safety net.

No blocking. No timeout risk on long wave runs.

---

### ⚑ Resolved: No dry_run input

Dry run doesn't belong as a tool input parameter. If you want to preview what the tool would
do, configure that at the action level (separate action with `dry_run: true` in input), or
use `sdlc orchestrate` CLI directly.

The tool takes `{}` as input — no parameters.

---

### ⚑ Resolved: Feature tie-breaking

**First wins.** When multiple features have active directives at the same priority level,
pick the first alphabetically by slug. Simple, deterministic, no hidden heuristics.

---

### ⚑ Resolved: Flight lock scope

**If anything development-related is running, don't run.**

The lock check is: does `.sdlc/.dev-driver.lock` exist and is it < 2h old?
In practice, this covers concurrent dev-driver runs. The lock is written on dispatch,
cleared when the agent process finishes (or after 2h TTL).

This is broader than "just dev-driver" — if a dev-driver run is in flight (the agent is
running /sdlc-run or /sdlc-run-wave), nothing else dispatches until it completes.

---

### ⚑ Resolved: Ships with sdlc init

`dev-driver` is a stock tool, scaffolded by `sdlc init` alongside `quality-check` and `ama`.
Every new project gets it automatically. Existing projects pick it up on `sdlc update`.

---

### ⚑ Resolved: Actions opt-in via --run-actions flag

**This is the most significant architectural decision.**

Current behavior: `sdlc ui` runs the orchestrator daemon by default. `--no-orchestrate` skips it.

New behavior:
- `sdlc ui` → server only, no orchestrator (actions don't run)
- `sdlc ui --run-actions` → server + orchestrator tick loop (actions run)

This inverts the current default and renames the flag. `--no-orchestrate` is removed.

**Rationale:** Actions execute real work (running Claude agents, advancing features). That
shouldn't happen unless the developer explicitly opts in. The current "on by default" behavior
is too aggressive for a server that might be started in CI, as a background service, or during
debugging sessions.

**Implementation:** The server's `AppState` starts the orchestrator tick loop as a tokio task
when `--run-actions` is passed. Otherwise no tick loop is spawned.

---

### What gets built

**1. `dev-driver` stock tool** (`.sdlc/tools/dev-driver/tool.ts`)
- Input: `{}`
- Priority waterfall: lock → quality → features → waves → idle
- Async spawn for dispatch
- Flight lock: `.sdlc/.dev-driver.lock`

**2. `--run-actions` flag on `sdlc ui`**
- Remove `--no-orchestrate`
- Add `--run-actions` (off by default)
- Behavior: when set, spawns orchestrator tick loop as tokio task

**3. Stock tool scaffolding in `sdlc init` / `sdlc update`**
- `dev-driver/tool.ts` added to stock tool set
- Stock action template documented (label: dev-driver, tool: dev-driver, recurrence: 4h)

---

### Default action the user creates (once, manually)

```
Label:     dev-driver
Tool:      dev-driver
Input:     {}
Recurrence: every 4 hours (14400s)
```

This is not auto-created — it's documented as the standard setup pattern. User creates it
via the Actions UI or CLI after starting with `--run-actions`.
