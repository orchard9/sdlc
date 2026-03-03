# Spec: changelog-cli

## Summary

Add a `sdlc changelog` subcommand to the CLI that prints a human-readable, terminal-formatted digest of recent project activity derived from run records in `.sdlc/.runs/`. Serves as the developer's "what happened lately" view without opening the web UI.

## Problem

Agents and developers need a quick terminal view of recent project activity — which features advanced, which agent runs completed or failed, and when. Currently this requires opening the web UI or grepping through `.sdlc/.runs/*.json` manually.

## Behavior

### Command signature

```
sdlc changelog [--since <value>] [--limit <N>] [--json]
```

### Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--since` | string | `7d` | Filter events newer than this. Accepts: ISO date (`2026-03-01`), relative shorthand (`3d`, `7d`, `1w`), or the literal `last-merge` to find the most recent `feature_merged` event and use its timestamp |
| `--limit` | usize | `20` | Maximum events to display |
| `--json` | bool | false | Emit structured JSON instead of the pretty-print table |

### Data source

Reads `.sdlc/.runs/*.json` (excluding `*.events.json`) using the same `load_run_history` logic already in `sdlc-server`. Each `RunRecord` is treated as one event.

### Event classification and icons

Classify each `RunRecord` by `run_type` and `status` into one of the following display categories:

| Category | Icon | Condition |
|---|---|---|
| `run_failed` | ⚠️ | `status == "failed"` or `status == "stopped"` with `error != null` |
| `feature_merged` | 🚀 | `run_type == "merge"` or `key` contains `"merge"` |
| `approval` | ✅ | `run_type` contains `"approve"` or `key` contains `"approve"` |
| `phase_advanced` | 🔄 | `run_type == "transition"` or `key` contains `"transition"` |
| `agent_run` | ▶ | All other completed runs |
| `run_stopped` | ⏹ | `status == "stopped"` without error |

### Pretty-print format

Default (non-JSON) output is one line per event, newest first:

```
<icon> <label>                        <relative-time>
```

Example:

```
⚠️  Ponder: dev-driver-tool             2 min ago
▶  Harvest ponder/agent-observability  14 min ago
✅  Approve audit: feedback-edit         1 hr ago
🚀  Merge: telemetry-wallclock          3 hr ago
🔄  Phase advanced: feedback-edit        5 hr ago
```

- Label taken from `RunRecord.label`
- Relative time rendered as: `N sec ago`, `N min ago`, `N hr ago`, `N days ago`
- If no events match the filter, print: `No activity in the selected window.`

### JSON output format

```json
{
  "since": "2026-02-24T00:00:00Z",
  "limit": 20,
  "total": 5,
  "events": [
    {
      "id": "20260302-094858-uue",
      "category": "agent_run",
      "icon": "▶",
      "label": "Harvest ponder/agent-observability",
      "run_type": "knowledge_harvest",
      "status": "completed",
      "started_at": "2026-03-02T09:48:58Z",
      "cost_usd": 0.33655
    }
  ]
}
```

## Implementation Plan

1. **`crates/sdlc-cli/src/cmd/changelog.rs`** — new module:
   - Parse `--since`, `--limit`, `--json` flags
   - Load run history from `.sdlc/.runs/` using a standalone `load_run_history`-equivalent (avoid pulling sdlc-server as a dep — re-implement the 10-line scan locally in sdlc-cli, or extract a shared helper to sdlc-core)
   - Filter by `since` timestamp
   - Apply limit
   - Classify + pretty-print or emit JSON

2. **`crates/sdlc-cli/src/main.rs`** — register the `Changelog` variant in the `Commands` enum and dispatch to `cmd::changelog::run()`

3. **`crates/sdlc-cli/src/cmd/mod.rs`** — add `pub mod changelog;`

## Out of Scope

- Writing to the changelog (read-only)
- Watching for changes (no `--watch`)
- Any frontend/API integration (handled by `changelog-api` feature)
- Pager support

## Acceptance Criteria

- `sdlc changelog` runs without error and prints activity for the last 7 days
- `sdlc changelog --since 1d --limit 5` returns at most 5 events from the last 24 hours
- `sdlc changelog --since last-merge` finds the most recent merge event and shows everything after it
- `sdlc changelog --json` emits valid JSON matching the schema above
- When no runs exist in the window, prints `No activity in the selected window.`
- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy --all -- -D warnings` passes
