# Dev Driver — Design Sketch

## What it is

A stock sdlc tool (`dev-driver`) that reads project state, applies a priority waterfall,
picks ONE action to take, dispatches it, and exits. Paired with a default scheduled action
that runs every 4 hours, making development self-advancing.

## Tool contract

**Input:** `{ dry_run?: boolean }`  
**Output:** `ToolResult<DevDriverResult>`

```typescript
interface DevDriverResult {
  action: 'waiting' | 'quality_failing' | 'feature_advanced' | 'wave_started' | 'idle'
  // Waiting
  lock_age_mins?: number
  // Quality failing
  failed_checks?: string[]
  // Feature advanced
  slug?: string
  directive_type?: string
  // Wave started
  milestone?: string
  // All cases
  reason?: string
  dry_run?: boolean
  dry_run_plan?: string   // what it would have done
}
```

## Flight lock

Path: `.sdlc/.dev-driver.lock`  
Format: JSON `{ pid: number, started_at: ISO, action: string }`  
TTL: 2 hours. Lock older than 2h is cleared and run proceeds.

## Execution model

The tool dispatches agent work asynchronously:

```typescript
// Spawn Claude Code and exit immediately
spawn('claude', ['--print', `/sdlc-run ${slug}`], {
  detached: true,
  stdio: 'ignore',
  cwd: root,
})
process.unref()  // don't wait for child
```

The lock is NOT removed by the tool — it's removed by the spawned agent
process as its last action (convention, not enforced). After 2h TTL it
auto-clears regardless.

**Alternative (dry_run mode):** Tool reads all state, computes the priority
waterfall, returns `dry_run_plan` describing what it would do — no dispatch.

## Default action

```yaml
label: dev-driver
tool_name: dev-driver
tool_input: {}
recurrence_secs: 14400   # 4 hours
```

Created via:
```bash
sdlc orchestrate add "dev-driver" --tool dev-driver --every 4h
```

Or via the Actions UI with:
- Tool: dev-driver
- Recurrence: 4h
- Input: {}

## Feature priority within Level 3

When multiple features have active directives, order by:
1. Phase (IMPLEMENTATION > REVIEW > AUDIT > QA > SPECIFIED)
2. Within same phase: most recently active feature (mtime of most recent artifact)
3. Tie-break: alphabetical slug

## Stock tool — ships with sdlc init

Like `quality-check` and `ama`, `dev-driver` is a stock tool scaffolded by
`sdlc tool scaffold dev-driver` and documented in `.sdlc/tools/tools.md`.

It reads `.sdlc/config.yaml` for any project-specific overrides:
```yaml
dev_driver:
  skip_quality_check: false
  max_parallel_features: 1
  idle_threshold_days: 7   # after N days idle, add to report
```

## v1 exclusions

- Feature suggestion (product work, not automation)
- Codebase audit sweep (too expensive per tick)
- Multi-feature parallel dispatch (too risky without better state tracking)
- Cross-project fleet management
