use crate::output::print_json;
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{
    classifier::{Classifier, EvalContext},
    config::Config,
    feature::Feature,
    rules::default_rules,
    search::{FeatureIndex, TaskIndex},
    state::State,
    types::ActionType,
};
use std::path::Path;

#[derive(Subcommand)]
pub enum QuerySubcommand {
    /// Show features that are blocked
    Blocked,
    /// Show features that are ready to work on
    Ready {
        /// Filter by phase (e.g. draft, specified, planned)
        #[arg(long)]
        phase: Option<String>,
    },
    /// Show features with artifacts awaiting approval
    NeedsApproval,
    /// Full-text search across feature titles, descriptions, and comments
    ///
    /// Supports AND/OR/NOT, field scoping (phase:ready, slug:auth),
    /// phrase queries ("exact phrase"), and prefix wildcards (auth*).
    /// Multiple bare words are ANDed by default.
    Search {
        /// Query string
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    /// Full-text search across task titles and descriptions
    ///
    /// Supports AND/OR/NOT, field scoping (status:blocked, status:pending),
    /// phrase queries ("exact phrase"), and prefix wildcards (login*).
    /// Multiple bare words are ANDed by default.
    SearchTasks {
        /// Query string
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
}

pub fn run(root: &Path, subcmd: QuerySubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        QuerySubcommand::Blocked => blocked(root, json),
        QuerySubcommand::Ready { phase } => ready(root, phase, json),
        QuerySubcommand::NeedsApproval => needs_approval(root, json),
        QuerySubcommand::Search { query, limit } => search(root, &query, limit, json),
        QuerySubcommand::SearchTasks { query, limit } => search_tasks(root, &query, limit, json),
    }
}

fn blocked(root: &Path, json: bool) -> anyhow::Result<()> {
    let features = Feature::list(root).context("failed to list features")?;
    let blocked: Vec<_> = features.iter().filter(|f| f.is_blocked()).collect();

    if json {
        let out: Vec<_> = blocked
            .iter()
            .map(|f| serde_json::json!({ "slug": f.slug, "blockers": f.blockers }))
            .collect();
        return print_json(&out);
    }

    if blocked.is_empty() {
        println!("No blocked features.");
    } else {
        for f in blocked {
            println!("{}: {}", f.slug, f.blockers.join(", "));
        }
    }
    Ok(())
}

fn ready(root: &Path, phase: Option<String>, json: bool) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config")?;
    let state = State::load(root).context("failed to load state")?;
    let features = Feature::list(root).context("failed to list features")?;
    let classifier = Classifier::new(default_rules());

    let ready: Vec<_> = features
        .iter()
        .filter(|f| !f.archived && !f.is_blocked())
        .filter(|f| phase.as_deref().is_none_or(|p| f.phase.to_string() == p))
        .filter_map(|f| {
            let ctx = EvalContext {
                feature: f,
                state: &state,
                config: &config,
                root,
            };
            let c = classifier.classify(&ctx);
            if matches!(
                c.action,
                ActionType::WaitForApproval | ActionType::Done | ActionType::UnblockDependency
            ) {
                None
            } else {
                Some((f, c))
            }
        })
        .collect();

    if json {
        let out: Vec<_> = ready
            .iter()
            .map(|(f, c)| {
                serde_json::json!({
                    "slug": f.slug,
                    "phase": f.phase.to_string(),
                    "action": c.action.as_str(),
                    "message": c.message,
                    "next_command": c.next_command,
                })
            })
            .collect();
        return print_json(&out);
    }

    if ready.is_empty() {
        println!("No features ready to work on.");
    } else {
        println!("Ready to work on:");
        for (f, _c) in ready {
            println!("  {} [{}]", f.slug, f.phase);
        }
    }
    Ok(())
}

fn needs_approval(root: &Path, json: bool) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config")?;
    let state = State::load(root).context("failed to load state")?;
    let features = Feature::list(root).context("failed to list features")?;
    let classifier = Classifier::new(default_rules());

    let pending: Vec<_> = features
        .iter()
        .filter(|f| !f.archived)
        .filter_map(|f| {
            let ctx = EvalContext {
                feature: f,
                state: &state,
                config: &config,
                root,
            };
            let c = classifier.classify(&ctx);
            if is_approval_action(c.action) {
                Some((f, c))
            } else {
                None
            }
        })
        .collect();

    if json {
        let out: Vec<_> = pending
            .iter()
            .map(|(f, c)| {
                serde_json::json!({
                    "slug": f.slug,
                    "action": c.action.as_str(),
                    "message": c.message,
                    "next_command": c.next_command,
                })
            })
            .collect();
        return print_json(&out);
    }

    if pending.is_empty() {
        println!("No features need approval.");
    } else {
        println!("Awaiting approval:");
        for (f, c) in pending {
            println!("  {} — {}", f.slug, c.message);
            println!("    Run: {}", c.next_command);
        }
    }
    Ok(())
}

fn is_approval_action(action: ActionType) -> bool {
    matches!(
        action,
        ActionType::ApproveSpec
            | ActionType::ApproveDesign
            | ActionType::ApproveTasks
            | ActionType::ApproveQaPlan
            | ActionType::ApproveReview
            | ActionType::ApproveAudit
            | ActionType::ApproveMerge
            | ActionType::WaitForApproval
    )
}

fn search(root: &Path, query_str: &str, limit: usize, json: bool) -> anyhow::Result<()> {
    let features = Feature::list(root).context("failed to list features")?;
    let index = FeatureIndex::build(&features, root).context("failed to build search index")?;
    let results = index.search(query_str, limit).context("search failed")?;

    if json {
        let out: Vec<_> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "slug":  r.slug,
                    "title": r.title,
                    "phase": r.phase,
                    "score": r.score,
                })
            })
            .collect();
        return print_json(&out);
    }

    if results.is_empty() {
        println!("No results.");
        return Ok(());
    }

    println!(
        "{} result{} for {:?}:",
        results.len(),
        if results.len() == 1 { "" } else { "s" },
        query_str
    );
    for r in &results {
        println!(
            "  [{:.2}] {:<30} {:<40} {}",
            r.score, r.slug, r.title, r.phase
        );
    }
    Ok(())
}

fn search_tasks(root: &Path, query_str: &str, limit: usize, json: bool) -> anyhow::Result<()> {
    let features = Feature::list(root).context("failed to list features")?;
    let index = TaskIndex::build(&features).context("failed to build task search index")?;
    let results = index
        .search(query_str, limit)
        .context("task search failed")?;

    if json {
        let out: Vec<_> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "feature_slug": r.feature_slug,
                    "task_id":      r.task_id,
                    "title":        r.title,
                    "status":       r.status,
                    "score":        r.score,
                })
            })
            .collect();
        return print_json(&out);
    }

    if results.is_empty() {
        println!("No results.");
        return Ok(());
    }

    println!(
        "{} result{} for {:?}:",
        results.len(),
        if results.len() == 1 { "" } else { "s" },
        query_str
    );
    for r in &results {
        println!(
            "  [{:.2}] {:<30} {:<20} {:<40} {}",
            r.score, r.feature_slug, r.task_id, r.title, r.status
        );
    }
    Ok(())
}
