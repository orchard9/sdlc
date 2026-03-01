# Spec: orchestrator-tick-cli

## Overview

Add `sdlc orchestrate` — a CLI command that starts a tick-rate daemon and
provides `add` / `list` subcommands to manage actions. The daemon is the
execution layer on top of `ActionDb` from `orchestrator-action-model`.

## Commands

### `sdlc orchestrate [--tick-rate <secs>] [--db <path>]`

Starts the tick loop. Blocks until CTRL-C. On startup: `startup_recovery(2 * tick_rate)`.

```
tick_start = Instant::now()
due = db.range_due(now)
for action in due:
    db.set_status(action.id, Running)
    script = tool_script(root, &action.tool_name)
    result = run_tool(script, "--run", Some(&tool_input_json), root, None)
    status = match result:
        Ok(stdout)  → Completed { result: parse_json(stdout) }
        Err(e)      → Failed { reason: e.to_string() }
    db.set_status(action.id, status)
    if action.recurrence.is_some() && Completed:
        db.insert(action.rescheduled(interval))
elapsed = tick_start.elapsed()
sleep(max(0, tick_rate - elapsed))
loop
```

Defaults: `--tick-rate 60`, `--db .sdlc/orchestrator.db`.

### `sdlc orchestrate add <label> --tool <name> --input <json> [--at <spec>] [--every <secs>]`

Inserts a `Pending` scheduled action.

- `--at` accepts: `now`, `now+Ns` (s=secs, m=mins, h=hours), or RFC3339 datetime
- `--at` defaults to `now` (fires on next tick)
- `--every <secs>` sets `recurrence: Some(Duration::from_secs(secs))`

### `sdlc orchestrate list [--status <filter>]`

Reads all actions from the DB, prints a table:
`ID | LABEL | TOOL | STATUS | UPDATED_AT`

## File locations

- `crates/sdlc-cli/src/cmd/orchestrate.rs` — new file
- `crates/sdlc-core/src/paths.rs` — add `orchestrator_db_path(root) -> PathBuf`
- `crates/sdlc-cli/src/cmd/mod.rs` — add `pub mod orchestrate`
- `crates/sdlc-cli/src/main.rs` — add `Orchestrate` variant + dispatch
- `.gitignore` — add `.sdlc/orchestrator.db`

## Rescheduling after Completed

```rust
fn rescheduled(action: &Action, interval: Duration) -> Action {
    Action::new_scheduled(
        &action.label,
        &action.tool_name,
        action.tool_input.clone(),
        Utc::now() + chrono::Duration::from_std(interval).unwrap_or_default(),
        action.recurrence,
    )
}
```

## Error handling

- Tool not found → `Failed { reason: "tool script not found: ..." }` (don't crash the daemon)
- Tool execution error → `Failed { reason: e.to_string() }`
- DB error → propagate to `main` (daemon exits with error)
- On CTRL-C → clean exit (the action currently Running will be recovered on next startup)
