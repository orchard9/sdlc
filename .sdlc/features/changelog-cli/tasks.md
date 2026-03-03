# Tasks: changelog-cli

## T1 — Create crates/sdlc-cli/src/cmd/changelog.rs

Implement the changelog module with:
- `run(root, since_str, limit, json)` entry point
- `SinceSpec` enum (Iso, Relative, LastMerge) with parsing from string
- `Category` enum with icon and str methods
- `ChangelogRun` struct (id, category, icon, label, run_type, status, started_at, cost_usd)
- `load_runs(root)` — scan `.sdlc/.runs/*.json`, skip `*.events.json`, deserialize into minimal struct
- `filter_by_since(runs, since_spec)` — compute cutoff from spec, handle `LastMerge` fallback
- `classify(run)` — map run_type/status to Category
- `format_relative(delta)` — format Duration as "N sec ago", "N min ago", "N hr ago", "N days ago"
- `pretty_print(runs)` — label padded to 45 chars + relative time
- `json_output(runs, since_cutoff, limit)` — emit JSON matching the spec schema
- Unit tests: `test_classify_run_failed`, `test_classify_merge`, `test_parse_since_relative`, `test_parse_since_iso`, `test_relative_time_format`

## T2 — Implement --since flag parsing (ISO / relative / last-merge)

Covered in T1 via `SinceSpec`. Details:
- ISO: parse `NaiveDate`, convert to `DateTime<Utc>` at midnight
- Relative: parse `"Nd"` or `"Nw"` suffix; `d` = days, `w` = weeks
- `last-merge`: find newest run where `run_type == "merge"` or `key.contains("merge")`; fall back to 7d with stderr warning if none found
- Invalid input → `anyhow::bail!("Invalid --since value: ...")`

## T3 — Pretty-print with icons

Covered in T1 via `Category::icon()` and `pretty_print()`:
- RunFailed → `⚠️`
- FeatureMerged → `🚀`
- Approval → `✅`
- PhaseAdvanced → `🔄`
- AgentRun → `▶`
- RunStopped → `⏹`
- Default: last 7d, limit 20
- Empty-state: `No activity in the selected window.`

## T4 — Register sdlc changelog subcommand in main.rs

Add to `Commands` enum in `crates/sdlc-cli/src/main.rs`:
```rust
/// Show a digest of recent project activity (runs, merges, approvals)
Changelog {
    #[arg(long, default_value = "7d")]
    since: String,
    #[arg(long, default_value_t = 20)]
    limit: usize,
},
```

Add `pub mod changelog;` to `crates/sdlc-cli/src/cmd/mod.rs`.

Add dispatch arm to `match cli.command { ... }`:
```rust
Commands::Changelog { since, limit } => cmd::changelog::run(&root, &since, limit, cli.json),
```
