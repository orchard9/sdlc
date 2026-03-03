# Dev Driver — Plan

## Vision

A developer can run `sdlc ui --run-actions`, create one scheduled action in the UI, and
from that point forward their project advances autonomously — features get implemented,
waves get started, quality gets checked — without manual triggering after every step.

The `dev-driver` tool is the brain: it reads project state, applies a priority waterfall,
picks the single most important development action, dispatches it asynchronously, and exits.
Scheduled every 4 hours via the orchestrator, it turns sdlc into a self-advancing system.

## Milestone

**v21-dev-driver** — Dev Driver: autonomous development advancement

The milestone delivers three things: the tool, the flag, and the distribution.

---

## Features

### 1. dev-driver-tool

**Title:** dev-driver: stock tool that finds and does the next development action

**What it does:**
- Implements `.sdlc/tools/dev-driver/tool.ts`
- Input: `{}` (no parameters)
- Priority waterfall (first match wins, one action per invocation):
  1. Flight lock check: if `.sdlc/.dev-driver.lock` exists and < 2h old → exit `{ action: "waiting" }`
  2. Quality check: runs `quality-check` tool — if `failed > 0` → exit `{ action: "quality_failing", failed_checks }`
  3. Features with active directives: `sdlc state` → find features in IMPLEMENTATION/REVIEW/AUDIT/QA
     - Pick first alphabetically → async spawn `claude --print "/sdlc-next <slug>"`
     - Write lock file, exit `{ action: "feature_advanced", slug, phase }`
  4. Wave ready: `sdlc project prepare` → find milestone with all features PLANNED/READY
     - Async spawn `claude --print "/sdlc-run-wave <milestone>"`
     - Write lock file, exit `{ action: "wave_started", milestone }`
  5. No work: exit `{ action: "idle", reason: "no actionable work found" }`
- Lock format: `{ started_at: ISO, action: string, pid: number }`, 2h TTL
- Output schema: `ToolResult<{ action, slug?, phase?, milestone?, failed_checks?, reason? }>`

**Tasks:**
- Scaffold `dev-driver/tool.ts` following quality-check pattern
- Implement flight lock read/write/check logic
- Implement priority waterfall (5 levels)
- Implement async spawn for Claude dispatch (detached: true, stdio: ignore)
- Implement output schema and ToolResult shape
- Write README.md for the tool

### 2. dev-driver-run-actions-flag

**Title:** sdlc ui --run-actions: make action execution opt-in (invert no-orchestrate default)

**What it does:**
- Removes `--no-orchestrate` flag from `sdlc ui`
- Adds `--run-actions` flag (default: false — orchestrator does NOT run unless flag is set)
- Current behavior: orchestrator runs by default, `--no-orchestrate` skips it
- New behavior: orchestrator off by default, `--run-actions` enables it
- Implementation: flip the boolean in `crates/sdlc-cli/src/main.rs` + update server startup
- Update any docs or DEVELOPER.md references to `--no-orchestrate`

**Rationale:** Actions execute real Claude agent work. That shouldn't happen unless explicitly
opted in — running `sdlc ui` in CI, for debugging, or as a background service should not
trigger autonomous code changes.

**Tasks:**
- Remove `no_orchestrate` field, add `run_actions` field in Cli struct
- Flip boolean logic: spawn orchestrator only when `run_actions == true`
- Update DEVELOPER.md and any README references
- Update CLAUDE.md if mentioned

### 3. dev-driver-init-scaffold

**Title:** dev-driver scaffolded by sdlc init and sdlc update

**What it does:**
- Adds `dev-driver` to the stock tools written by `sdlc init` and `sdlc update`
- Alongside `quality-check` and `ama`, `dev-driver` is scaffolded into `.sdlc/tools/dev-driver/`
- Documents the standard default action pattern in `.sdlc/tools/tools.md`
- Stock action recipe:
  ```
  Label:      dev-driver
  Tool:       dev-driver
  Input:      {}
  Recurrence: 14400s (4 hours)
  ```

**Tasks:**
- Add `dev-driver/tool.ts` and `dev-driver/README.md` as constants in `init.rs`
- Add to the `write_tool_scaffolding()` function (or equivalent) in `init.rs`
- Update `tools.md` to document dev-driver and the default action recipe
- Add to `sdlc update` path so existing projects receive it
