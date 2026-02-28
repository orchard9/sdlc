use crate::output::{print_json, print_table};
use anyhow::Context;
use sdlc_core::prepare::{self, write_wave_plan, GapSeverity, PrepareResult, ProjectPhase};
use std::path::Path;

pub fn run(root: &Path, milestone: Option<&str>, json: bool) -> anyhow::Result<()> {
    let result = prepare::prepare(root, milestone).context("failed to prepare milestone")?;

    if let Some(ref slug) = result.milestone {
        if !result.waves.is_empty() {
            write_wave_plan(root, slug, &result.waves).context("failed to write wave_plan.yaml")?;
        }
    }

    if json {
        print_json(&result)?;
        return Ok(());
    }

    render_human(&result);
    Ok(())
}

fn render_human(result: &PrepareResult) {
    // Phase + milestone
    print!("Phase: {}", result.project_phase);
    if let Some(title) = &result.milestone_title {
        print!("  Milestone: {}", title);
    }
    println!();

    // Progress
    if let Some(p) = &result.milestone_progress {
        println!(
            "Progress: {}/{} released, {} in progress, {} blocked, {} pending",
            p.released, p.total, p.in_progress, p.blocked, p.pending
        );
    }
    println!();

    // Gaps
    if !result.gaps.is_empty() {
        println!("GAPS");
        for gap in &result.gaps {
            let icon = match gap.severity {
                GapSeverity::Blocker => "\u{2717}", // ✗
                GapSeverity::Warning => "\u{26a0}", // ⚠
                GapSeverity::Info => "\u{2139}",    // ℹ
            };
            println!("  {} {}", icon, gap.message);
        }
        println!();
    }

    // Waves
    if result.waves.is_empty() {
        match &result.project_phase {
            ProjectPhase::Idle => println!("Nothing planned. Use /sdlc-ponder to start ideating."),
            ProjectPhase::Pondering => {
                println!("Active ponders in progress. Use /sdlc-ponder to continue.")
            }
            ProjectPhase::Verifying { .. } => {
                println!("All features released. Run verification / UAT.")
            }
            _ => println!("No actionable features in this milestone."),
        }
    } else {
        for wave in &result.waves {
            let worktree_marker = if wave.needs_worktrees {
                " [needs worktrees]"
            } else {
                ""
            };
            println!("WAVE {} — {}{}", wave.number, wave.label, worktree_marker);

            let rows: Vec<Vec<String>> = wave
                .items
                .iter()
                .map(|item| {
                    let wt = if item.needs_worktree { "yes" } else { "" };
                    vec![
                        item.slug.clone(),
                        item.phase.to_string(),
                        item.action.clone(),
                        item.blocked_by.join(", "),
                        wt.to_string(),
                    ]
                })
                .collect();

            print_table(&["SLUG", "PHASE", "ACTION", "BLOCKED BY", "WORKTREE"], rows);
            println!();
        }
    }

    // Blocked
    if !result.blocked.is_empty() {
        println!("BLOCKED");
        for b in &result.blocked {
            println!("  {} — {}", b.slug, b.reason);
        }
        println!();
    }

    // Next commands
    if !result.next_commands.is_empty() {
        println!("NEXT");
        for cmd in &result.next_commands {
            println!("  {}", cmd);
        }
    }
}
