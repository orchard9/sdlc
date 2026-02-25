use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{
    classifier::{Classifier, EvalContext},
    config::Config,
    feature::Feature,
    milestone::Milestone,
    rules::default_rules,
    state::State,
    types::ActionType,
};
use std::path::Path;

#[derive(Subcommand)]
pub enum MilestoneSubcommand {
    /// Create a new milestone
    Create {
        slug: String,
        /// Milestone title
        #[arg(long)]
        title: String,
        /// Initial feature slugs (repeatable: --feature auth --feature payments)
        #[arg(long = "feature")]
        features: Vec<String>,
    },
    /// List all milestones
    List,
    /// Show milestone details and its features
    Info { slug: String },
    /// List all tasks across every feature in a milestone
    Tasks { slug: String },
    /// Add a feature to a milestone
    AddFeature {
        slug: String,
        feature_slug: String,
        /// Insert at position N (0-based); appends if omitted
        #[arg(long, value_name = "N")]
        position: Option<usize>,
    },
    /// Remove a feature from a milestone
    RemoveFeature { slug: String, feature_slug: String },
    /// Reorder features in a milestone
    Reorder {
        slug: String,
        /// Feature slugs in desired order
        features: Vec<String>,
    },
    /// Mark a milestone complete
    Complete { slug: String },
    /// Cancel a milestone
    Cancel { slug: String },
    /// Update milestone metadata
    Update {
        slug: String,
        #[arg(long)]
        title: Option<String>,
    },
    /// Run the classifier on every feature in the milestone
    Review { slug: String },
    /// Orchestrate all features in a milestone using sdlc_milestone_driver
    Run {
        /// Milestone slug to orchestrate
        slug: String,
        /// Execution mode: advise | guided | auto
        #[arg(long, default_value = "auto")]
        mode: String,
    },
}

pub fn run(root: &Path, subcmd: MilestoneSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        MilestoneSubcommand::Create {
            slug,
            title,
            features,
        } => create(root, &slug, &title, &features, json),
        MilestoneSubcommand::List => list(root, json),
        MilestoneSubcommand::Info { slug } => info(root, &slug, json),
        MilestoneSubcommand::Tasks { slug } => tasks(root, &slug, json),
        MilestoneSubcommand::AddFeature {
            slug,
            feature_slug,
            position,
        } => add_feature(root, &slug, &feature_slug, position, json),
        MilestoneSubcommand::RemoveFeature { slug, feature_slug } => {
            remove_feature(root, &slug, &feature_slug, json)
        }
        MilestoneSubcommand::Reorder { slug, features } => reorder(root, &slug, &features, json),
        MilestoneSubcommand::Complete { slug } => complete(root, &slug, json),
        MilestoneSubcommand::Cancel { slug } => cancel(root, &slug, json),
        MilestoneSubcommand::Update { slug, title } => update(root, &slug, title.as_deref(), json),
        MilestoneSubcommand::Review { slug } => review(root, &slug, json),
        MilestoneSubcommand::Run { slug, mode } => run_driver(root, &slug, &mode),
    }
}

fn run_driver(root: &Path, slug: &str, mode: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("python")
        .args([
            "-m",
            "sdlc_milestone_driver",
            "--milestone",
            slug,
            "--mode",
            mode,
            "--root",
            root.to_str().unwrap_or("."),
        ])
        .status()
        .context("failed to launch sdlc_milestone_driver")?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn create(
    root: &Path,
    slug: &str,
    title: &str,
    initial_features: &[String],
    json: bool,
) -> anyhow::Result<()> {
    let mut milestone = Milestone::create(root, slug, title)
        .with_context(|| format!("failed to create milestone '{slug}'"))?;

    for f in initial_features {
        milestone.add_feature(f);
    }
    milestone.save(root).context("failed to save milestone")?;

    let mut state = State::load(root).context("failed to load state")?;
    state.add_milestone(slug);
    state.save(root).context("failed to save state")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "title": title,
            "features": milestone.features,
        }))?;
    } else {
        println!("Created milestone '{slug}'.");
        if !milestone.features.is_empty() {
            println!("  Features: {}", milestone.features.join(", "));
        }
    }
    Ok(())
}

fn list(root: &Path, json: bool) -> anyhow::Result<()> {
    let milestones = Milestone::list(root).context("failed to list milestones")?;

    if json {
        let items: Vec<serde_json::Value> = milestones
            .iter()
            .map(|m| {
                serde_json::json!({
                    "slug": m.slug,
                    "title": m.title,
                    "status": m.status.to_string(),
                    "feature_count": m.features.len(),
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if milestones.is_empty() {
        println!("No milestones.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = milestones
        .iter()
        .map(|m| {
            vec![
                m.slug.clone(),
                m.title.clone(),
                m.status.to_string(),
                m.features.len().to_string(),
            ]
        })
        .collect();
    print_table(&["SLUG", "TITLE", "STATUS", "FEATURES"], rows);
    Ok(())
}

fn info(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let milestone =
        Milestone::load(root, slug).with_context(|| format!("milestone '{slug}' not found"))?;

    if json {
        print_json(&serde_json::json!({
            "slug": milestone.slug,
            "title": milestone.title,
            "status": milestone.status.to_string(),
            "features": milestone.features,
            "created_at": milestone.created_at,
            "updated_at": milestone.updated_at,
            "completed_at": milestone.completed_at,
            "cancelled_at": milestone.cancelled_at,
        }))?;
        return Ok(());
    }

    println!("Milestone: {} — {}", milestone.slug, milestone.title);
    println!("Status:    {}", milestone.status);
    println!("Features:  {}", milestone.features.len());
    if milestone.features.is_empty() {
        println!("  (none)");
    } else {
        for f in &milestone.features {
            println!("  {}", f);
        }
    }
    Ok(())
}

fn tasks(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let milestone =
        Milestone::load(root, slug).with_context(|| format!("milestone '{slug}' not found"))?;

    let mut all_tasks: Vec<(String, String, String, String)> = Vec::new(); // (feature, id, status, title)
    for feature_slug in &milestone.features {
        match Feature::load(root, feature_slug) {
            Ok(feature) => {
                for task in &feature.tasks {
                    all_tasks.push((
                        feature_slug.clone(),
                        task.id.clone(),
                        task.status.to_string(),
                        task.title.clone(),
                    ));
                }
            }
            Err(_) => {
                all_tasks.push((
                    feature_slug.clone(),
                    "?".to_string(),
                    "?".to_string(),
                    "(feature not found)".to_string(),
                ));
            }
        }
    }

    if json {
        let items: Vec<serde_json::Value> = all_tasks
            .iter()
            .map(|(feature, id, status, title)| {
                serde_json::json!({
                    "feature": feature,
                    "task_id": id,
                    "status": status,
                    "title": title,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if all_tasks.is_empty() {
        println!("No tasks in milestone '{slug}'.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = all_tasks
        .into_iter()
        .map(|(feature, id, status, title)| vec![feature, id, status, title])
        .collect();
    print_table(&["FEATURE", "TASK ID", "STATUS", "TITLE"], rows);
    Ok(())
}

fn add_feature(
    root: &Path,
    slug: &str,
    feature_slug: &str,
    position: Option<usize>,
    json: bool,
) -> anyhow::Result<()> {
    let mut milestone =
        Milestone::load(root, slug).with_context(|| format!("milestone '{slug}' not found"))?;

    let added = if let Some(pos) = position {
        milestone.add_feature_at(feature_slug, pos)
    } else {
        milestone.add_feature(feature_slug)
    };

    if !added {
        anyhow::bail!(
            "feature '{}' is already in milestone '{}'",
            feature_slug,
            slug
        );
    }
    milestone.save(root).context("failed to save milestone")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "feature_slug": feature_slug,
            "added": true,
        }))?;
    } else {
        println!("Added feature '{feature_slug}' to milestone '{slug}'.");
    }
    Ok(())
}

fn reorder(root: &Path, slug: &str, features: &[String], json: bool) -> anyhow::Result<()> {
    let mut milestone =
        Milestone::load(root, slug).with_context(|| format!("milestone '{slug}' not found"))?;

    let refs: Vec<&str> = features.iter().map(|s| s.as_str()).collect();
    milestone.reorder_features(&refs)?;
    milestone.save(root).context("failed to save milestone")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "features": milestone.features,
        }))?;
    } else {
        for (i, f) in milestone.features.iter().enumerate() {
            println!("{}. {}", i + 1, f);
        }
    }
    Ok(())
}

fn remove_feature(root: &Path, slug: &str, feature_slug: &str, json: bool) -> anyhow::Result<()> {
    let mut milestone =
        Milestone::load(root, slug).with_context(|| format!("milestone '{slug}' not found"))?;

    let removed = milestone.remove_feature(feature_slug);
    if !removed {
        anyhow::bail!(
            "feature '{}' not found in milestone '{}'",
            feature_slug,
            slug
        );
    }
    milestone.save(root).context("failed to save milestone")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "feature_slug": feature_slug,
            "removed": true,
        }))?;
    } else {
        println!("Removed feature '{feature_slug}' from milestone '{slug}'.");
    }
    Ok(())
}

fn complete(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let mut milestone =
        Milestone::load(root, slug).with_context(|| format!("milestone '{slug}' not found"))?;

    milestone.complete();
    milestone.save(root).context("failed to save milestone")?;

    if json {
        print_json(&serde_json::json!({ "slug": slug, "status": "complete" }))?;
    } else {
        println!("Milestone '{slug}' marked complete.");
    }
    Ok(())
}

fn cancel(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let mut milestone =
        Milestone::load(root, slug).with_context(|| format!("milestone '{slug}' not found"))?;

    milestone.cancel();
    milestone.save(root).context("failed to save milestone")?;

    if json {
        print_json(&serde_json::json!({ "slug": slug, "status": "cancelled" }))?;
    } else {
        println!("Milestone '{slug}' cancelled.");
    }
    Ok(())
}

fn update(root: &Path, slug: &str, title: Option<&str>, json: bool) -> anyhow::Result<()> {
    let mut milestone =
        Milestone::load(root, slug).with_context(|| format!("milestone '{slug}' not found"))?;

    if let Some(t) = title {
        milestone.update_title(t);
    }
    milestone.save(root).context("failed to save milestone")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "title": milestone.title,
        }))?;
    } else {
        println!("Updated milestone '{slug}'.");
    }
    Ok(())
}

fn review(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let milestone =
        Milestone::load(root, slug).with_context(|| format!("milestone '{slug}' not found"))?;

    let state = State::load(root).context("failed to load state")?;
    let config = Config::load(root).unwrap_or_else(|_| Config::new(&state.project));
    let classifier = Classifier::new(default_rules());

    // (feature_slug, phase, next_action, task_id, blocked_label)
    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();

    for feature_slug in &milestone.features {
        match Feature::load(root, feature_slug) {
            Ok(feature) => {
                let ctx = EvalContext {
                    feature: &feature,
                    state: &state,
                    config: &config,
                    root,
                };
                let c = classifier.classify(&ctx);

                let next_action = if let Some(tid) = &c.task_id {
                    format!("{} {}", c.action, tid)
                } else {
                    c.action.to_string()
                };

                // BLOCKED column: use classifier output so blocker comments surface
                let blocked = if c.action == ActionType::UnblockDependency
                    || (c.action == ActionType::WaitForApproval
                        && c.message.contains("blocker comment"))
                {
                    // Truncate message to keep table readable
                    let detail: String = c.message.chars().take(50).collect();
                    format!("yes ({})", detail)
                } else {
                    "no".to_string()
                };

                rows.push((
                    feature_slug.clone(),
                    feature.phase.to_string(),
                    next_action,
                    c.message.clone(),
                    blocked,
                ));
            }
            Err(_) => {
                rows.push((
                    feature_slug.clone(),
                    "?".to_string(),
                    "?".to_string(),
                    "feature not found".to_string(),
                    "?".to_string(),
                ));
            }
        }
    }

    if json {
        let items: Vec<serde_json::Value> = rows
            .iter()
            .map(|(feature, phase, next_action, message, blocked)| {
                serde_json::json!({
                    "feature": feature,
                    "phase": phase,
                    "next_action": next_action,
                    "message": message,
                    "blocked": blocked != "no",
                    "blocked_reason": if blocked == "no" { None } else { Some(blocked) },
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "milestone": slug,
            "feature_count": milestone.features.len(),
            "features": items,
        }))?;
        return Ok(());
    }

    println!(
        "Milestone: {} ({} feature{})\n",
        slug,
        milestone.features.len(),
        if milestone.features.len() == 1 {
            ""
        } else {
            "s"
        }
    );

    if rows.is_empty() {
        println!("No features in this milestone.");
        return Ok(());
    }

    let table_rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|(feature, phase, next_action, _message, blocked)| {
            vec![feature, phase, next_action, blocked]
        })
        .collect();
    print_table(&["FEATURE", "PHASE", "NEXT ACTION", "BLOCKED"], table_rows);
    Ok(())
}
