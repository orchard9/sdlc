use crate::output::print_json;
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::feature::Feature;
use sdlc_core::score::QualityScore;
use std::path::Path;

// ---------------------------------------------------------------------------
// Subcommand types
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum ScoreSubcommand {
    /// Set a quality score on a feature for a given lens
    Set {
        /// Feature slug
        slug: String,
        /// Lens name (e.g. product_fit, research_grounding, implementation)
        #[arg(long)]
        lens: String,
        /// Score value (0-100)
        #[arg(long)]
        value: u32,
        /// Agent ID that produced this score
        #[arg(long)]
        evaluator: String,
    },

    /// Show all lens scores for a feature
    Show {
        /// Feature slug
        slug: String,
    },

    /// Show score history (all scores chronologically)
    History {
        /// Feature slug
        slug: String,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcmd: ScoreSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        ScoreSubcommand::Set {
            slug,
            lens,
            value,
            evaluator,
        } => set_score(root, &slug, &lens, value, &evaluator, json),
        ScoreSubcommand::Show { slug } => show_scores(root, &slug, json),
        ScoreSubcommand::History { slug } => show_history(root, &slug, json),
    }
}

// ---------------------------------------------------------------------------
// set
// ---------------------------------------------------------------------------

fn set_score(
    root: &Path,
    slug: &str,
    lens: &str,
    value: u32,
    evaluator: &str,
    _json: bool,
) -> anyhow::Result<()> {
    if value > 100 {
        anyhow::bail!("score value must be 0-100, got {value}");
    }

    let mut feature = Feature::load(root, slug).context("failed to load feature")?;
    let score = QualityScore {
        lens: lens.to_string(),
        score: value,
        deductions: vec![],
        evaluator: evaluator.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    feature.add_score(score);
    feature.save(root).context("failed to save feature")?;
    println!("Score set: {slug} [{lens}] = {value} (by {evaluator})");
    Ok(())
}

// ---------------------------------------------------------------------------
// show
// ---------------------------------------------------------------------------

fn show_scores(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let feature = Feature::load(root, slug).context("failed to load feature")?;

    if json {
        let value = serde_json::json!({
            "slug": slug,
            "scores": feature.scores,
        });
        print_json(&value)?;
        return Ok(());
    }

    if feature.scores.is_empty() {
        println!("No scores for feature '{slug}'.");
        return Ok(());
    }

    println!("Scores for '{slug}':");
    for s in &feature.scores {
        let deduction_count = s.deductions.len();
        println!(
            "  {:<24} {:>3}/100  (evaluator: {}, deductions: {})",
            s.lens, s.score, s.evaluator, deduction_count,
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// history
// ---------------------------------------------------------------------------

fn show_history(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let feature = Feature::load(root, slug).context("failed to load feature")?;

    if json {
        let value = serde_json::json!({
            "slug": slug,
            "scores": feature.scores,
        });
        print_json(&value)?;
        return Ok(());
    }

    if feature.scores.is_empty() {
        println!("No score history for feature '{slug}'.");
        return Ok(());
    }

    println!("Score history for '{slug}':");
    for s in &feature.scores {
        println!(
            "  {} | {:<24} {:>3}/100 | {}",
            s.timestamp, s.lens, s.score, s.evaluator,
        );
    }
    Ok(())
}
