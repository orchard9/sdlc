use crate::output::print_json;
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{
    classifier::{Classifier, EvalContext},
    config::Config,
    feature::Feature,
    rules::default_rules,
    state::State,
    types::ActionType,
};
use std::path::Path;

#[derive(Subcommand)]
pub enum QuerySubcommand {
    /// Show features that are blocked
    Blocked,
    /// Show features that are ready to work on
    Ready,
    /// Show features with artifacts awaiting approval
    NeedsApproval,
}

pub fn run(root: &Path, subcmd: QuerySubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        QuerySubcommand::Blocked => blocked(root, json),
        QuerySubcommand::Ready => ready(root, json),
        QuerySubcommand::NeedsApproval => needs_approval(root, json),
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

fn ready(root: &Path, json: bool) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config")?;
    let state = State::load(root).context("failed to load state")?;
    let features = Feature::list(root).context("failed to list features")?;
    let classifier = Classifier::new(default_rules());

    let ready: Vec<_> = features
        .iter()
        .filter(|f| !f.archived && !f.is_blocked())
        .filter(|f| {
            let ctx = EvalContext {
                feature: f,
                state: &state,
                config: &config,
                root,
            };
            let c = classifier.classify(&ctx);
            !matches!(
                c.action,
                ActionType::WaitForApproval | ActionType::Done | ActionType::UnblockDependency
            )
        })
        .collect();

    if json {
        let out: Vec<_> = ready
            .iter()
            .map(|f| serde_json::json!({ "slug": f.slug, "phase": f.phase.to_string() }))
            .collect();
        return print_json(&out);
    }

    if ready.is_empty() {
        println!("No features ready to work on.");
    } else {
        println!("Ready to work on:");
        for f in ready {
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
            if matches!(
                c.action,
                ActionType::ApproveSpec
                    | ActionType::ApproveDesign
                    | ActionType::ApproveReview
                    | ActionType::ApproveMerge
            ) {
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
                    "command": c.next_command,
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
