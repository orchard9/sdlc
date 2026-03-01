# Design: orchestrator-tick-cli

## orchestrate.rs structure

```rust
#[derive(Subcommand)]
pub enum OrchestrateSubcommand {
    /// Add a scheduled action
    Add {
        label: String,
        #[arg(long)] tool: String,
        #[arg(long)] input: String,           // raw JSON string
        #[arg(long, default_value = "now")] at: String,
        #[arg(long)] every: Option<u64>,      // seconds
    },
    /// List all actions
    List {
        #[arg(long)] status: Option<String>,
    },
}

pub fn run(root: &Path, subcommand: Option<OrchestrateSubcommand>,
           tick_rate: u64, db_path: Option<PathBuf>) -> anyhow::Result<()>
```

Top-level `sdlc orchestrate` (no subcommand) → start daemon.
With subcommand → run and exit.

## Tick loop

Fully synchronous — no tokio required. Uses `std::thread::sleep`.

The daemon loop exits only on `CTRL-C` (std::io::Error with `Interrupted` kind)
or an unrecoverable DB error. Tool execution errors per-action don't stop the loop.

## --at parsing

```rust
fn parse_at(s: &str) -> anyhow::Result<DateTime<Utc>> {
    if s == "now" {
        return Ok(Utc::now());
    }
    // "now+10s", "now+5m", "now+1h"
    if let Some(rest) = s.strip_prefix("now+") {
        let (num_str, unit) = rest.split_at(rest.len() - 1);
        let n: u64 = num_str.parse()?;
        let secs = match unit {
            "s" => n,
            "m" => n * 60,
            "h" => n * 3600,
            _ => anyhow::bail!("unknown unit '{unit}', use s/m/h"),
        };
        return Ok(Utc::now() + chrono::Duration::seconds(secs as i64));
    }
    // RFC3339
    Ok(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc))
}
```

## paths addition

```rust
pub fn orchestrator_db_path(root: &Path) -> PathBuf {
    root.join(SDLC_DIR).join("orchestrator.db")
}
```

## main.rs changes

```rust
// Import
use cmd::orchestrate::OrchestrateSubcommand;

// Commands variant
/// Run the tick-rate orchestrator daemon
Orchestrate {
    /// Seconds between ticks (default 60)
    #[arg(long, default_value_t = 60)]
    tick_rate: u64,
    /// Path to orchestrator DB
    #[arg(long)]
    db: Option<PathBuf>,
    #[command(subcommand)]
    subcommand: Option<OrchestrateSubcommand>,
},

// Dispatch
Commands::Orchestrate { tick_rate, db, subcommand } =>
    cmd::orchestrate::run(&root, subcommand, tick_rate, db),
```

## Table output (list)

```
ID        LABEL          TOOL            STATUS     UPDATED
a1b2...   my-service     quality-check   Completed  2026-03-01 00:01:23
```

Use `print_table` from `output.rs` with columns: id (first 8 chars), label, tool_name, status tag, updated_at.

## .gitignore entry

Append `.sdlc/orchestrator.db` to root `.gitignore`.
