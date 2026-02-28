use crate::output::print_json;
use anyhow::Context;
use sdlc_core::{
    feature::Feature,
    milestone::{Milestone, MilestoneStatus},
    state::State,
};
use std::collections::HashSet;
use std::path::Path;

pub fn run(root: &Path, json: bool) -> anyhow::Result<()> {
    let state = State::load(root).context("failed to load state")?;
    let features = Feature::list(root).unwrap_or_default();
    let milestones = Milestone::list(root).unwrap_or_default();
    let active_milestones: Vec<&Milestone> = milestones
        .iter()
        .filter(|m| {
            !matches!(
                m.compute_status(&features),
                MilestoneStatus::Released | MilestoneStatus::Skipped
            )
        })
        .collect();

    if json {
        #[derive(serde::Serialize)]
        struct MilestoneFeature<'a> {
            slug: &'a str,
            phase: String,
            status: &'static str,
            title: &'a str,
        }

        #[derive(serde::Serialize)]
        struct MilestoneSummary<'a> {
            slug: &'a str,
            title: &'a str,
            status: String,
            features: Vec<MilestoneFeature<'a>>,
            done: usize,
            total: usize,
        }

        #[derive(serde::Serialize)]
        struct StateOutput<'a> {
            project: &'a str,
            milestones: Vec<MilestoneSummary<'a>>,
            unassigned: Vec<MilestoneFeature<'a>>,
            feature_count: usize,
            last_action: Option<&'a sdlc_core::state::HistoryEntry>,
            active_directives: &'a [sdlc_core::state::ActiveDirective],
            blocked: &'a [sdlc_core::state::BlockedItem],
        }

        let assigned: HashSet<&str> = milestones
            .iter()
            .flat_map(|m| m.features.iter().map(|s| s.as_str()))
            .collect();

        let milestone_summaries: Vec<MilestoneSummary> = milestones
            .iter()
            .map(|m| {
                let mf: Vec<MilestoneFeature> = m
                    .features
                    .iter()
                    .filter_map(|slug| features.iter().find(|f| f.slug == *slug))
                    .map(|f| MilestoneFeature {
                        slug: &f.slug,
                        phase: f.phase.to_string(),
                        status: feature_status(f),
                        title: &f.title,
                    })
                    .collect();
                let done = mf.iter().filter(|f| f.phase == "released").count();
                MilestoneSummary {
                    slug: &m.slug,
                    title: &m.title,
                    status: m.compute_status(&features).to_string(),
                    done,
                    total: mf.len(),
                    features: mf,
                }
            })
            .collect();

        let unassigned: Vec<MilestoneFeature> = features
            .iter()
            .filter(|f| !assigned.contains(f.slug.as_str()))
            .map(|f| MilestoneFeature {
                slug: &f.slug,
                phase: f.phase.to_string(),
                status: feature_status(f),
                title: &f.title,
            })
            .collect();

        let output = StateOutput {
            project: &state.project,
            milestones: milestone_summaries,
            unassigned,
            feature_count: features.len(),
            last_action: state.last_action(),
            active_directives: &state.active_directives,
            blocked: &state.blocked,
        };
        return print_json(&output);
    }

    // -- Human-readable output ------------------------------------------------

    println!("Project: {}", state.project);

    if features.is_empty() && active_milestones.is_empty() {
        println!("Features: 0");
        println!("\nNo features yet. Run: sdlc feature create <slug>");
        return Ok(());
    }

    // Collect all feature slugs assigned to any active milestone
    let assigned: HashSet<&str> = active_milestones
        .iter()
        .flat_map(|m| m.features.iter().map(|s| s.as_str()))
        .collect();

    // Print each active milestone with its features
    for ms in &active_milestones {
        let ms_features: Vec<&Feature> = ms
            .features
            .iter()
            .filter_map(|slug| features.iter().find(|f| f.slug == *slug))
            .collect();

        let done = ms_features
            .iter()
            .filter(|f| f.phase.to_string() == "released")
            .count();

        println!(
            "\nMilestone: {} ({}/{} done)",
            ms.title,
            done,
            ms_features.len()
        );

        if ms_features.is_empty() {
            println!("  (no features)");
        } else {
            let rows: Vec<Vec<String>> = ms_features.iter().map(|f| feature_row(f)).collect();
            print_table_indented(&["SLUG", "PHASE", "STATUS", "TITLE"], rows);
        }
    }

    // Unassigned features
    let unassigned: Vec<&Feature> = features
        .iter()
        .filter(|f| !assigned.contains(f.slug.as_str()))
        .collect();

    if !unassigned.is_empty() {
        if active_milestones.is_empty() {
            println!("\nFeatures:");
        } else {
            println!("\nUnassigned:");
        }
        let rows: Vec<Vec<String>> = unassigned.iter().map(|f| feature_row(f)).collect();
        print_table_indented(&["SLUG", "PHASE", "STATUS", "TITLE"], rows);
    }

    // Active directives
    if !state.active_directives.is_empty() {
        println!("\nActive directives:");
        for w in &state.active_directives {
            println!("  {} — {}", w.feature, w.action);
        }
    }

    // Blocked items
    if !state.blocked.is_empty() {
        println!("\nBlocked:");
        for b in &state.blocked {
            let reason = if b.reason.len() > 50 {
                format!("{}...", &b.reason[..47])
            } else {
                b.reason.clone()
            };
            println!("  {} — {}", b.feature, reason);
        }
    }

    Ok(())
}

fn feature_status(f: &Feature) -> &'static str {
    if f.is_blocked() {
        "blocked"
    } else if f.archived {
        "archived"
    } else {
        ""
    }
}

fn feature_row(f: &Feature) -> Vec<String> {
    vec![
        f.slug.clone(),
        f.phase.to_string(),
        feature_status(f).to_string(),
        f.title.clone(),
    ]
}

/// Like print_table but indented 2 spaces for nesting under a milestone.
fn print_table_indented(headers: &[&str], rows: Vec<Vec<String>>) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let header_row: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{:width$}", h, width = widths[i]))
        .collect();
    println!("  {}", header_row.join("  "));

    let sep: Vec<String> = widths.iter().map(|&w| "-".repeat(w)).collect();
    println!("  {}", sep.join("  "));

    for row in &rows {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let w = widths.get(i).copied().unwrap_or(0);
                format!("{:width$}", cell, width = w)
            })
            .collect();
        println!("  {}", cells.join("  "));
    }
}
