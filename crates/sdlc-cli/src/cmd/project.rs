use crate::output::{print_json, print_table};
use anyhow::Context;
use chrono::Utc;
use clap::Subcommand;
use sdlc_core::{feature::Feature, state::State, types::TaskStatus};
use std::collections::HashMap;
use std::path::Path;

#[derive(Subcommand)]
pub enum ProjectSubcommand {
    /// Overall project health dashboard
    Status,
    /// Velocity and phase dwell-time statistics
    Stats,
    /// List all blocked features with duration and reason
    Blockers,
    /// Survey milestone, find gaps, organize into parallelizable waves
    Prepare {
        /// Milestone slug (auto-detects if omitted)
        #[arg(long)]
        milestone: Option<String>,
    },
}

pub fn run(root: &Path, subcmd: ProjectSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        ProjectSubcommand::Status => status(root, json),
        ProjectSubcommand::Stats => stats(root, json),
        ProjectSubcommand::Blockers => blockers(root, json),
        ProjectSubcommand::Prepare { milestone } => {
            super::prepare::run(root, milestone.as_deref(), json)
        }
    }
}

fn status(root: &Path, json: bool) -> anyhow::Result<()> {
    let features = Feature::list(root).context("failed to list features")?;
    let state = State::load(root).context("failed to load state")?;

    let active: Vec<_> = features.iter().filter(|f| !f.archived).collect();
    let active_count = active.len();

    // Phase distribution (preserve natural phase order)
    let mut phase_counts: HashMap<String, usize> = HashMap::new();
    for f in &active {
        *phase_counts.entry(f.phase.to_string()).or_insert(0) += 1;
    }

    // Task summary across all active features
    let mut total = 0usize;
    let mut completed = 0usize;
    let mut in_progress = 0usize;
    let mut task_blocked = 0usize;
    let mut pending = 0usize;
    for f in &active {
        for t in &f.tasks {
            total += 1;
            match t.status {
                TaskStatus::Completed => completed += 1,
                TaskStatus::InProgress => in_progress += 1,
                TaskStatus::Blocked => task_blocked += 1,
                TaskStatus::Pending => pending += 1,
            }
        }
    }

    let blocked_count = state.blocked.len();

    if json {
        let phases_json = serde_json::to_value(&phase_counts)?;
        print_json(&serde_json::json!({
            "project": state.project,
            "active_feature_count": active_count,
            "phases": phases_json,
            "tasks": {
                "total": total,
                "completed": completed,
                "in_progress": in_progress,
                "blocked": task_blocked,
                "pending": pending,
            },
            "blocked_count": blocked_count,
        }))?;
        return Ok(());
    }

    println!(
        "Project: {}   Active features: {}",
        state.project, active_count
    );
    println!();

    if !phase_counts.is_empty() {
        println!("PHASE DISTRIBUTION");
        for phase in sdlc_core::types::Phase::all() {
            if let Some(count) = phase_counts.get(phase.as_str()) {
                println!("  {:<18} {}", phase, count);
            }
        }
        println!();
    }

    println!(
        "TASK SUMMARY\n  Total: {} | Completed: {} | In Progress: {} | Blocked: {} | Pending: {}",
        total, completed, in_progress, task_blocked, pending
    );
    println!();

    if blocked_count == 0 {
        println!("BLOCKERS  No blocked features");
    } else {
        println!("BLOCKERS  {} feature(s) blocked", blocked_count);
    }

    Ok(())
}

fn stats(root: &Path, json: bool) -> anyhow::Result<()> {
    let features = Feature::list(root).context("failed to list features")?;
    let state = State::load(root).context("failed to load state")?;

    let now = Utc::now();
    let seven_days_ago = now - chrono::Duration::days(7);

    let actions_7d = state
        .history
        .iter()
        .filter(|h| h.timestamp >= seven_days_ago)
        .count();

    // Count phase transitions recorded in the last 7 days across all features
    let transitions_7d: usize = features
        .iter()
        .flat_map(|f| f.phase_history.iter())
        .filter(|pt| pt.entered >= seven_days_ago)
        .count();

    // Average phase dwell times across all features
    let mut dwell_totals: HashMap<String, (f64, usize)> = HashMap::new();
    for feature in &features {
        for transition in &feature.phase_history {
            let exited = transition.exited.unwrap_or(now);
            let duration_secs = (exited - transition.entered).num_seconds();
            let days = duration_secs as f64 / 86400.0;
            let entry = dwell_totals
                .entry(transition.phase.to_string())
                .or_insert((0.0, 0));
            entry.0 += days;
            entry.1 += 1;
        }
    }

    let avg_dwell_days: HashMap<String, f64> = dwell_totals
        .iter()
        .map(|(phase, (total, count))| (phase.clone(), *total / *count as f64))
        .collect();

    if json {
        let avg_dwell_json = serde_json::to_value(&avg_dwell_days)?;
        print_json(&serde_json::json!({
            "actions_7d": actions_7d,
            "transitions_7d": transitions_7d,
            "avg_dwell_days": avg_dwell_json,
        }))?;
        return Ok(());
    }

    println!("Actions (last 7 days): {}", actions_7d);
    println!("Phase transitions:     {}", transitions_7d);

    if !avg_dwell_days.is_empty() {
        println!();
        println!("AVERAGE PHASE DWELL");
        // Print in natural phase order
        for phase in sdlc_core::types::Phase::all() {
            if let Some(days) = avg_dwell_days.get(phase.as_str()) {
                println!("  {:<18} {:.1} days", phase, days);
            }
        }
    }

    Ok(())
}

fn blockers(root: &Path, json: bool) -> anyhow::Result<()> {
    let state = State::load(root).context("failed to load state")?;
    let features = Feature::list(root).context("failed to list features")?;
    let now = Utc::now();

    // (slug, phase, days_blocked, reason)
    let mut blocker_rows: Vec<(String, String, i64, String)> = Vec::new();

    // Blockers tracked in state.blocked
    for item in &state.blocked {
        let days = (now - item.since).num_days();
        let phase = features
            .iter()
            .find(|f| f.slug == item.feature)
            .map(|f| f.phase.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        blocker_rows.push((item.feature.clone(), phase, days, item.reason.clone()));
    }

    // Also surface features with feature-level blockers not yet in state.blocked
    for feature in &features {
        if feature.archived {
            continue;
        }
        if !feature.blockers.is_empty() && !state.blocked.iter().any(|b| b.feature == feature.slug)
        {
            for reason in &feature.blockers {
                blocker_rows.push((
                    feature.slug.clone(),
                    feature.phase.to_string(),
                    0,
                    reason.clone(),
                ));
            }
        }
    }

    if json {
        let items: Vec<serde_json::Value> = blocker_rows
            .iter()
            .map(|(slug, phase, days, reason)| {
                serde_json::json!({
                    "slug": slug,
                    "phase": phase,
                    "days_blocked": days,
                    "reason": reason,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if blocker_rows.is_empty() {
        println!("No blocked features.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = blocker_rows
        .iter()
        .map(|(slug, phase, days, reason)| {
            vec![
                slug.clone(),
                phase.clone(),
                format!("{} days", days),
                reason.clone(),
            ]
        })
        .collect();
    print_table(&["FEATURE", "PHASE", "BLOCKED FOR", "REASON"], rows);
    Ok(())
}
