use crate::output::print_json;
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{feature::Feature, types::ArtifactType};
use std::path::Path;
use std::str::FromStr;

#[derive(Subcommand)]
pub enum ArtifactSubcommand {
    /// Approve an artifact
    Approve {
        slug: String,
        artifact: String,
        #[arg(long)]
        by: Option<String>,
    },
    /// Reject an artifact
    Reject {
        slug: String,
        artifact: String,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Mark artifact as draft (written but not yet approved)
    Draft { slug: String, artifact: String },
}

pub fn run(root: &Path, subcmd: ArtifactSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        ArtifactSubcommand::Approve { slug, artifact, by } => {
            approve(root, &slug, &artifact, by, json)
        }
        ArtifactSubcommand::Reject {
            slug,
            artifact,
            reason,
        } => reject(root, &slug, &artifact, reason, json),
        ArtifactSubcommand::Draft { slug, artifact } => draft(root, &slug, &artifact, json),
    }
}

fn approve(
    root: &Path,
    slug: &str,
    artifact_str: &str,
    by: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let artifact_type = ArtifactType::from_str(artifact_str)
        .with_context(|| format!("unknown artifact type: {artifact_str}"))?;

    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;

    feature
        .approve_artifact(artifact_type, by.clone())
        .with_context(|| format!("failed to approve {artifact_str}"))?;
    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "artifact": artifact_str,
            "status": "approved",
            "by": by,
        }))?;
    } else {
        println!("Approved: {slug}/{artifact_str}");
    }
    Ok(())
}

fn reject(
    root: &Path,
    slug: &str,
    artifact_str: &str,
    reason: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let artifact_type = ArtifactType::from_str(artifact_str)
        .with_context(|| format!("unknown artifact type: {artifact_str}"))?;

    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;

    feature
        .reject_artifact(artifact_type, reason.clone())
        .with_context(|| format!("failed to reject {artifact_str}"))?;
    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "artifact": artifact_str,
            "status": "rejected",
            "reason": reason,
        }))?;
    } else {
        println!("Rejected: {slug}/{artifact_str}");
        if let Some(r) = &reason {
            println!("Reason: {r}");
        }
    }
    Ok(())
}

fn draft(root: &Path, slug: &str, artifact_str: &str, json: bool) -> anyhow::Result<()> {
    let artifact_type = ArtifactType::from_str(artifact_str)
        .with_context(|| format!("unknown artifact type: {artifact_str}"))?;

    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;

    feature
        .mark_artifact_draft(artifact_type)
        .with_context(|| format!("failed to mark {artifact_str} as draft"))?;
    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "artifact": artifact_str,
            "status": "draft",
        }))?;
    } else {
        println!("Marked as draft: {slug}/{artifact_str}");
    }
    Ok(())
}
