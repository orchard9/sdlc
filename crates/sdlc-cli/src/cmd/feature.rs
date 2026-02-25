use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{
    config::Config,
    feature::Feature,
    state::State,
    types::Phase,
};
use std::path::Path;
use std::str::FromStr;

#[derive(Subcommand)]
pub enum FeatureSubcommand {
    /// Create a new feature
    Create {
        slug: String,
        #[arg(long)]
        title: Option<String>,
        /// Optional one-liner description of the feature's intent
        #[arg(long)]
        description: Option<String>,
    },
    /// List all features
    List,
    /// Show feature details
    Show { slug: String },
    /// Transition a feature to a new phase
    Transition { slug: String, phase: String },
    /// Archive a feature
    Archive { slug: String },
}

pub fn run(root: &Path, subcmd: FeatureSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        FeatureSubcommand::Create { slug, title, description } => {
            create(root, &slug, title, description, json)
        }
        FeatureSubcommand::List => list(root, json),
        FeatureSubcommand::Show { slug } => show(root, &slug, json),
        FeatureSubcommand::Transition { slug, phase } => transition(root, &slug, &phase, json),
        FeatureSubcommand::Archive { slug } => archive(root, &slug, json),
    }
}

fn create(
    root: &Path,
    slug: &str,
    title: Option<String>,
    description: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let title = title.unwrap_or_else(|| slug.replace('-', " "));
    let feature = Feature::create_with_description(root, slug, &title, description)
        .with_context(|| format!("failed to create feature '{slug}'"))?;

    let mut state = State::load(root).context("failed to load state")?;
    state.add_active_feature(slug);
    state.save(root).context("failed to save state")?;

    if json {
        print_json(&feature)?;
    } else {
        println!("Created feature: {slug} — {title}");
        println!("Next: sdlc next --for {slug}");
    }
    Ok(())
}

fn list(root: &Path, json: bool) -> anyhow::Result<()> {
    let features = Feature::list(root).context("failed to list features")?;

    if json {
        let summaries: Vec<_> = features
            .iter()
            .map(|f| {
                serde_json::json!({
                    "slug": f.slug,
                    "title": f.title,
                    "description": f.description,
                    "phase": f.phase.to_string(),
                    "blocked": f.is_blocked(),
                    "archived": f.archived,
                })
            })
            .collect();
        print_json(&summaries)?;
        return Ok(());
    }

    if features.is_empty() {
        println!("No features yet.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = features
        .iter()
        .map(|f| {
            vec![
                f.slug.clone(),
                f.phase.to_string(),
                if f.archived { "archived".to_string() } else { String::new() },
                f.title.clone(),
            ]
        })
        .collect();
    print_table(&["SLUG", "PHASE", "STATUS", "TITLE"], rows);
    Ok(())
}

fn show(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let feature = Feature::load(root, slug)
        .with_context(|| format!("feature '{slug}' not found"))?;

    if json {
        print_json(&feature)?;
        return Ok(());
    }

    println!("Feature: {} — {}", feature.slug, feature.title);
    if let Some(ref desc) = feature.description {
        println!("Desc:    {desc}");
    }
    println!("Phase:   {}", feature.phase);
    println!("Created: {}", feature.created_at.format("%Y-%m-%d %H:%M"));

    println!("\nArtifacts:");
    for artifact in &feature.artifacts {
        println!("  {:<15} {}", artifact.artifact_type, artifact.status);
    }

    if !feature.tasks.is_empty() {
        println!("\nTasks ({}):", feature.tasks.len());
        for task in &feature.tasks {
            println!("  [{}] {} — {}", task.id, task.status, task.title);
        }
    }

    if feature.is_blocked() {
        println!("\nBlockers:");
        for b in &feature.blockers {
            println!("  - {b}");
        }
    }

    Ok(())
}

fn transition(root: &Path, slug: &str, phase_str: &str, json: bool) -> anyhow::Result<()> {
    let target = Phase::from_str(phase_str)
        .with_context(|| format!("unknown phase: {phase_str}"))?;

    let config = Config::load(root).context("failed to load config")?;
    let mut feature = Feature::load(root, slug)
        .with_context(|| format!("feature '{slug}' not found"))?;

    feature.transition(target, &config)
        .with_context(|| format!("cannot transition '{slug}' to {phase_str}"))?;
    feature.save(root).context("failed to save feature")?;

    let mut state = State::load(root).context("failed to load state")?;
    state.record_action(slug, sdlc_core::types::ActionType::ImplementTask, target, "transition");
    state.save(root).context("failed to save state")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "phase": target.to_string(),
        }))?;
    } else {
        println!("Transitioned '{slug}' to {target}");
    }
    Ok(())
}

fn archive(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let mut feature = Feature::load(root, slug)
        .with_context(|| format!("feature '{slug}' not found"))?;
    feature.archived = true;
    feature.save(root).context("failed to save feature")?;

    let mut state = State::load(root).context("failed to load state")?;
    state.remove_active_feature(slug);
    state.save(root).context("failed to save state")?;

    if json {
        print_json(&serde_json::json!({ "slug": slug, "archived": true }))?;
    } else {
        println!("Archived feature: {slug}");
    }
    Ok(())
}
