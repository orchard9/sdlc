# Design: changelog-cli

## Overview

`sdlc changelog` is a read-only CLI command that scans `.sdlc/.runs/*.json` and renders a chronological digest of recent project activity. It is purely additive — no new data model, no new storage, no network calls.

## Module Structure

```
crates/sdlc-cli/src/
  main.rs            ← add Changelog variant to Commands enum
  cmd/
    mod.rs           ← add pub mod changelog
    changelog.rs     ← new: all flag parsing, loading, filtering, rendering
```

No changes to `sdlc-core`. The run-record loading logic is self-contained in `changelog.rs` (a ~15-line directory scan).

## Data Flow

```
.sdlc/.runs/*.json
        │
        ▼
  load_runs(root)          ← read + deserialize, skip *.events.json
        │
        ▼
  filter_by_since(runs, since)  ← compute cutoff timestamp; handle ISO / relative / last-merge
        │
        ▼
  take first N (limit)
        │
        ▼
  classify(run) → Category
        │
   ┌────┴─────┐
   │          │
pretty_print  json_output
   │          │
stdout      stdout
```

## Structs

### ChangelogRun (internal)

```rust
struct ChangelogRun {
    id: String,
    category: Category,
    icon: &'static str,
    label: String,
    run_type: String,
    status: String,
    started_at: DateTime<Utc>,
    cost_usd: Option<f64>,
}
```

### Category enum

```rust
enum Category {
    RunFailed,
    FeatureMerged,
    Approval,
    PhaseAdvanced,
    AgentRun,
    RunStopped,
}

impl Category {
    fn icon(&self) -> &'static str { ... }
    fn as_str(&self) -> &'static str { ... }
}
```

### SinceSpec (parsed from --since)

```rust
enum SinceSpec {
    Iso(DateTime<Utc>),
    Relative(Duration),
    LastMerge,
}
```

## Parsing --since

| Input | Parsed as |
|---|---|
| `2026-03-01` | `SinceSpec::Iso(...)` — parsed as midnight UTC |
| `3d` / `7d` | `SinceSpec::Relative(Duration::days(N))` |
| `1w` | `SinceSpec::Relative(Duration::weeks(1))` |
| `last-merge` | `SinceSpec::LastMerge` |
| omitted | `SinceSpec::Relative(Duration::days(7))` (default) |

For `last-merge`: scan all runs (no time filter), find the most recent one where `run_type == "merge"` or `key.contains("merge")`, use its `started_at` as the cutoff. If no merge exists, fall back to 7-day default and emit a note to stderr.

## Classifying a RunRecord

```rust
fn classify(r: &RunRecord) -> Category {
    if r.status == "failed" || (r.status == "stopped" && r.error.is_some()) {
        Category::RunFailed
    } else if r.run_type == "merge" || r.key.contains("merge") {
        Category::FeatureMerged
    } else if r.run_type.contains("approve") || r.key.contains("approve") {
        Category::Approval
    } else if r.run_type.contains("transition") || r.key.contains("transition") {
        Category::PhaseAdvanced
    } else if r.status == "stopped" {
        Category::RunStopped
    } else {
        Category::AgentRun
    }
}
```

## Pretty-Print Layout

Single line per event. Label is left-padded to 45 chars; relative time is right-aligned:

```
⚠️  Ponder: dev-driver-tool             2 min ago
▶  Harvest ponder/agent-observability  14 min ago
```

Relative time formatting:

| Delta | Format |
|---|---|
| < 60s | `N sec ago` |
| < 3600s | `N min ago` |
| < 86400s | `N hr ago` |
| ≥ 86400s | `N days ago` |

If the event list is empty: print `No activity in the selected window.` and exit 0.

## JSON Output Schema

```json
{
  "since": "<ISO 8601 cutoff timestamp>",
  "limit": 20,
  "total": 3,
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

## CLI Registration in main.rs

Add to the `Commands` enum:

```rust
/// Show a digest of recent project activity (runs, merges, approvals)
Changelog {
    /// Show events since: ISO date (2026-03-01), relative (7d, 1w), or last-merge
    #[arg(long, default_value = "7d")]
    since: String,
    /// Maximum events to show
    #[arg(long, default_value_t = 20)]
    limit: usize,
},
```

Dispatch in the `match` block:

```rust
Commands::Changelog { since, limit } => cmd::changelog::run(&root, &since, limit, cli.json),
```

## Dependencies

No new crates needed. Relies on:
- `chrono` (already in `sdlc-cli/Cargo.toml`)
- `serde_json` (already present)
- `anyhow` (already present)

## Error Handling

- Unreadable run files: skip silently (same as `load_run_history` in sdlc-server)
- Unparseable `--since` value: return `anyhow::bail!("Invalid --since value: ...")`
- No runs directory: treat as empty list, print empty-state message

## Testing

Unit tests in `changelog.rs`:
- `test_classify_run_failed` — status="failed" → RunFailed
- `test_classify_merge` — run_type="merge" → FeatureMerged
- `test_parse_since_relative` — "7d" → Relative(7 days)
- `test_parse_since_iso` — "2026-03-01" → Iso(midnight UTC)
- `test_relative_time_format` — given a delta, format string is correct
