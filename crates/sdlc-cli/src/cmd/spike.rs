use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::spikes::{self, SpikeVerdict};
use std::path::Path;

#[derive(Subcommand)]
pub enum SpikeSubcommand {
    /// List all spikes (tabular: slug | verdict | date | title)
    List,
    /// Show full spike details and findings
    Show { slug: String },
    /// Promote a spike to a ponder entry pre-seeded with findings
    Promote {
        slug: String,
        /// Override the ponder slug (defaults to spike slug)
        #[arg(long = "as")]
        as_slug: Option<String>,
    },
}

pub fn run(root: &Path, subcmd: SpikeSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        SpikeSubcommand::List => list(root, json),
        SpikeSubcommand::Show { slug } => show(root, &slug, json),
        SpikeSubcommand::Promote { slug, as_slug } => {
            promote(root, &slug, as_slug.as_deref(), json)
        }
    }
}

fn list(root: &Path, json: bool) -> anyhow::Result<()> {
    let entries = spikes::list(root).context("failed to list spikes")?;

    if json {
        let items: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "slug": e.slug,
                    "title": e.title,
                    "verdict": e.verdict.as_ref().map(|v| v.to_string()),
                    "date": e.date,
                    "ponder_slug": e.ponder_slug,
                    "knowledge_slug": e.knowledge_slug,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if entries.is_empty() {
        println!("No spikes.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|e| {
            vec![
                e.slug.clone(),
                e.verdict
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                e.date.clone().unwrap_or_else(|| "-".to_string()),
                e.title.clone(),
            ]
        })
        .collect();
    print_table(&["SLUG", "VERDICT", "DATE", "TITLE"], rows);
    Ok(())
}

fn show(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let (entry, findings) =
        spikes::load(root, slug).with_context(|| format!("spike '{slug}' not found"))?;

    if json {
        print_json(&serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "verdict": entry.verdict.as_ref().map(|v| v.to_string()),
            "date": entry.date,
            "the_question": entry.the_question,
            "ponder_slug": entry.ponder_slug,
            "knowledge_slug": entry.knowledge_slug,
            "findings_content": findings,
        }))?;
        return Ok(());
    }

    println!("Spike: {} — {}", entry.slug, entry.title);
    let verdict_str = entry
        .verdict
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let date_str = entry.date.as_deref().unwrap_or("-");
    println!("Verdict: {}    Date: {}", verdict_str, date_str);

    if let Some(ref q) = entry.the_question {
        println!("The Question: {q}");
    }

    if !findings.is_empty() {
        println!("\n--- Findings ---");
        print!("{findings}");
        // Ensure trailing newline before hints
        if !findings.ends_with('\n') {
            println!();
        }
    } else {
        println!("\nNo findings.");
    }

    // Verdict-specific hints
    match &entry.verdict {
        Some(SpikeVerdict::Adopt) => {
            println!("\nHint: ADOPT — consider /sdlc-hypothetical-planning to implement this technology.");
        }
        Some(SpikeVerdict::Reject) => {
            if let Some(ref ks) = entry.knowledge_slug {
                println!("\nHint: REJECT — findings stored in knowledge base as '{ks}'.");
            } else {
                println!("\nHint: REJECT — run `sdlc spike list` to trigger auto-filing to the knowledge base.");
            }
        }
        Some(SpikeVerdict::Adapt) => {
            if let Some(ref ps) = entry.ponder_slug {
                println!("\nPonder: already promoted → '{ps}'");
            }
        }
        None => {}
    }

    Ok(())
}

fn promote(root: &Path, slug: &str, as_slug: Option<&str>, json: bool) -> anyhow::Result<()> {
    let ponder_slug = spikes::promote_to_ponder(root, slug, as_slug)
        .with_context(|| format!("failed to promote spike '{slug}' to ponder"))?;

    if json {
        print_json(&serde_json::json!({
            "spike_slug": slug,
            "ponder_slug": ponder_slug,
        }))?;
        return Ok(());
    }

    println!("Promoted spike '{slug}' to ponder '{ponder_slug}'.");
    println!("Next: sdlc ponder show {ponder_slug}");
    Ok(())
}
