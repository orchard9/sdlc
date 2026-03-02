//! `sdlc orchestrate` — tick-rate daemon and action management subcommands.
//!
//! # Subcommands
//!
//! - `sdlc orchestrate [--tick-rate <secs>] [--db <path>]` — start the daemon
//! - `sdlc orchestrate add <label> --tool <name> --input <json> [--at <spec>] [--every <secs>]`
//! - `sdlc orchestrate list [--status <filter>]`

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use sdlc_core::orchestrator::{Action, ActionDb, ActionStatus, WebhookPayload};

use crate::output::print_table;

// ---------------------------------------------------------------------------
// CLI types
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum OrchestrateSubcommand {
    /// Schedule an action to run on the next tick or at a specific time
    Add {
        /// Human-readable label (e.g. "my-service")
        label: String,
        /// Tool slug matching a directory under .sdlc/tools/<name>/
        #[arg(long)]
        tool: String,
        /// JSON input passed to the tool via stdin
        #[arg(long)]
        input: String,
        /// When to fire: "now", "now+10s", "now+5m", "now+1h", or RFC3339
        #[arg(long, default_value = "now")]
        at: String,
        /// Re-schedule every N seconds after each Completed run
        #[arg(long)]
        every: Option<u64>,
    },
    /// List all actions in the orchestrator DB
    List {
        /// Filter by status: pending, running, completed, failed
        #[arg(long)]
        status: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(
    root: &Path,
    subcommand: Option<OrchestrateSubcommand>,
    tick_rate: u64,
    db_path: Option<PathBuf>,
) -> Result<()> {
    let db_path = db_path.unwrap_or_else(|| sdlc_core::paths::orchestrator_db_path(root));
    let db = ActionDb::open(&db_path)
        .with_context(|| format!("failed to open orchestrator DB at {}", db_path.display()))?;

    match subcommand {
        None => run_daemon(root, &db, tick_rate),
        Some(OrchestrateSubcommand::Add {
            label,
            tool,
            input,
            at,
            every,
        }) => run_add(&db, &label, &tool, &input, &at, every),
        Some(OrchestrateSubcommand::List { status }) => run_list(&db, status.as_deref()),
    }
}

// ---------------------------------------------------------------------------
// Daemon
// ---------------------------------------------------------------------------

pub fn run_daemon(root: &Path, db: &ActionDb, tick_rate_secs: u64) -> Result<()> {
    let tick_rate = Duration::from_secs(tick_rate_secs);

    let recovered = db
        .startup_recovery(tick_rate * 2)
        .context("startup recovery failed")?;
    if recovered > 0 {
        eprintln!("orchestrate: recovered {recovered} stale Running action(s) → Failed");
    }

    eprintln!(
        "orchestrate: daemon started (tick={tick_rate_secs}s, db={})",
        sdlc_core::paths::orchestrator_db_path(root).display()
    );

    loop {
        let tick_start = Instant::now();
        run_one_tick(root, db)?;
        let elapsed = tick_start.elapsed();
        if elapsed < tick_rate {
            std::thread::sleep(tick_rate - elapsed);
        }
    }
}

/// Execute one tick of the orchestrator: dispatch all due Pending actions,
/// then dispatch all pending webhook payloads.
///
/// This is the inner body of the daemon loop, extracted so that tests can
/// call it directly without spawning a blocking thread.
pub fn run_one_tick(root: &Path, db: &ActionDb) -> Result<()> {
    // Phase 1: scheduled actions
    let now = Utc::now();
    let due = db.range_due(now).context("range_due failed")?;
    for action in due {
        dispatch(root, db, action)?;
    }

    // Phase 2: webhook payloads
    let webhooks = db
        .all_pending_webhooks()
        .context("all_pending_webhooks failed")?;
    for payload in webhooks {
        dispatch_webhook(root, db, payload)?;
    }

    Ok(())
}

fn dispatch_webhook(root: &Path, db: &ActionDb, payload: WebhookPayload) -> Result<()> {
    let route = db
        .find_route_by_path(&payload.route_path)
        .context("route lookup failed")?;

    let route = match route {
        None => {
            eprintln!(
                "orchestrate: webhook [{}] no route registered — dropping",
                payload.route_path
            );
            db.delete_webhook(payload.id)
                .context("delete_webhook failed")?;
            return Ok(());
        }
        Some(r) => r,
    };

    let tool_input = match route.render_input(&payload.raw_body) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "orchestrate: webhook [{}] template render error — {e}",
                payload.route_path
            );
            db.delete_webhook(payload.id)
                .context("delete_webhook (render error) failed")?;
            return Ok(());
        }
    };

    let script = sdlc_core::paths::tool_script(root, &route.tool_name);
    if !script.exists() {
        eprintln!(
            "orchestrate: webhook [{}] tool script not found: {}",
            payload.route_path,
            script.display()
        );
        db.delete_webhook(payload.id)
            .context("delete_webhook (missing tool) failed")?;
        return Ok(());
    }

    let input_json = serde_json::to_string(&tool_input).context("serialize tool_input failed")?;

    match sdlc_core::tool_runner::run_tool(&script, "--run", Some(&input_json), root, None) {
        Ok(_stdout) => {
            eprintln!(
                "orchestrate: webhook [{}] completed (tool={})",
                payload.route_path, route.tool_name
            );
        }
        Err(e) => {
            eprintln!("orchestrate: webhook [{}] failed — {e}", payload.route_path);
        }
    }

    // Always delete the payload after dispatch (success or failure)
    db.delete_webhook(payload.id)
        .context("delete_webhook (post-dispatch) failed")?;

    Ok(())
}

fn dispatch(root: &Path, db: &ActionDb, action: Action) -> Result<()> {
    db.set_status(action.id, ActionStatus::Running)
        .with_context(|| format!("set Running failed for {}", action.id))?;

    let script = sdlc_core::paths::tool_script(root, &action.tool_name);

    if !script.exists() {
        let reason = format!("tool script not found: {}", script.display());
        eprintln!("orchestrate: [{}] {reason}", action.label);
        db.set_status(action.id, ActionStatus::Failed { reason })
            .with_context(|| format!("set Failed failed for {}", action.id))?;
        return Ok(());
    }

    let input_json =
        serde_json::to_string(&action.tool_input).context("failed to serialize tool_input")?;

    let status =
        match sdlc_core::tool_runner::run_tool(&script, "--run", Some(&input_json), root, None) {
            Ok(stdout) => {
                let result: serde_json::Value =
                    serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
                eprintln!("orchestrate: [{}] completed", action.label);
                ActionStatus::Completed { result }
            }
            Err(e) => {
                let reason = e.to_string();
                eprintln!("orchestrate: [{}] failed — {reason}", action.label);
                ActionStatus::Failed { reason }
            }
        };

    let completed = matches!(status, ActionStatus::Completed { .. });

    db.set_status(action.id, status)
        .with_context(|| format!("set Completed/Failed failed for {}", action.id))?;

    // Reschedule if recurrence is set and the run completed successfully
    if completed {
        if let Some(interval) = action.recurrence {
            let next = rescheduled(&action, interval);
            db.insert(&next)
                .with_context(|| format!("reschedule insert failed for {}", action.label))?;
            eprintln!(
                "orchestrate: [{}] rescheduled in {}s",
                action.label,
                interval.as_secs()
            );
        }
    }

    Ok(())
}

fn rescheduled(action: &Action, interval: Duration) -> Action {
    let next_tick_at =
        Utc::now() + chrono::Duration::from_std(interval).unwrap_or(chrono::Duration::seconds(60));
    Action::new_scheduled(
        &action.label,
        &action.tool_name,
        action.tool_input.clone(),
        next_tick_at,
        action.recurrence,
    )
}

// ---------------------------------------------------------------------------
// Add
// ---------------------------------------------------------------------------

fn run_add(
    db: &ActionDb,
    label: &str,
    tool: &str,
    input: &str,
    at: &str,
    every: Option<u64>,
) -> Result<()> {
    let next_tick_at = parse_at(at).with_context(|| format!("invalid --at value: '{at}'"))?;

    let tool_input: serde_json::Value =
        serde_json::from_str(input).context("--input must be valid JSON")?;

    let recurrence = every.map(Duration::from_secs);

    let action = Action::new_scheduled(label, tool, tool_input, next_tick_at, recurrence);
    db.insert(&action).context("failed to insert action")?;

    println!(
        "Scheduled: {} (tool={}, at={}, id={})",
        label,
        tool,
        next_tick_at.format("%Y-%m-%d %H:%M:%SZ"),
        &action.id.to_string()[..8]
    );
    Ok(())
}

fn parse_at(s: &str) -> Result<DateTime<Utc>> {
    if s == "now" {
        return Ok(Utc::now());
    }
    if let Some(rest) = s.strip_prefix("now+") {
        let unit = rest
            .chars()
            .last()
            .ok_or_else(|| anyhow::anyhow!("empty offset after 'now+'"))?;
        let num_str = &rest[..rest.len() - 1];
        let n: u64 = num_str
            .parse()
            .with_context(|| format!("expected a number before unit, got '{num_str}'"))?;
        let secs = match unit {
            's' => n,
            'm' => n * 60,
            'h' => n * 3600,
            _ => anyhow::bail!("unknown time unit '{unit}', use s/m/h"),
        };
        return Ok(Utc::now() + chrono::Duration::seconds(secs as i64));
    }
    // RFC3339
    let dt = DateTime::parse_from_rfc3339(s)
        .with_context(|| format!("'{s}' is not a valid RFC3339 datetime or now+Ns offset"))?;
    Ok(dt.with_timezone(&Utc))
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

fn run_list(db: &ActionDb, status_filter: Option<&str>) -> Result<()> {
    let actions = db.list_all().context("failed to list actions")?;

    let actions: Vec<_> = if let Some(filter) = status_filter {
        let filter = filter.to_lowercase();
        actions
            .into_iter()
            .filter(|a| status_tag(&a.status).to_lowercase() == filter)
            .collect()
    } else {
        actions
    };

    if actions.is_empty() {
        println!("No actions found.");
        return Ok(());
    }

    let headers = &["ID", "LABEL", "TOOL", "STATUS", "UPDATED"];
    let rows: Vec<Vec<String>> = actions
        .iter()
        .map(|a| {
            vec![
                a.id.to_string()[..8].to_string(),
                a.label.clone(),
                a.tool_name.clone(),
                status_tag(&a.status).to_string(),
                a.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            ]
        })
        .collect();

    print_table(headers, rows);
    Ok(())
}

fn status_tag(status: &ActionStatus) -> &'static str {
    match status {
        ActionStatus::Pending => "Pending",
        ActionStatus::Running => "Running",
        ActionStatus::Completed { .. } => "Completed",
        ActionStatus::Failed { .. } => "Failed",
    }
}
