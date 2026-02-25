use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{
    comment::{add_comment, resolve_comment, CommentFlag, CommentTarget},
    feature::Feature,
    types::ArtifactType,
};
use std::path::Path;

#[derive(Subcommand)]
pub enum CommentSubcommand {
    /// Add a comment to a feature, task, or artifact
    Create {
        slug: String,
        body: String,
        /// Attach comment to a specific task (e.g. T1)
        #[arg(long)]
        task: Option<String>,
        /// Attach comment to a specific artifact (e.g. spec, design)
        #[arg(long)]
        artifact: Option<String>,
        /// Flag: blocker, question, decision, fyi
        #[arg(long)]
        flag: Option<String>,
        /// Author name or agent identifier
        #[arg(long)]
        by: Option<String>,
    },
    /// List comments on a feature, optionally filtered to a task
    List {
        slug: String,
        /// Show only comments on this task (e.g. T1)
        #[arg(long)]
        task: Option<String>,
    },
    /// Resolve (remove) a comment, clearing any pipeline block it caused
    Resolve { slug: String, comment_id: String },
}

pub fn run(root: &Path, subcmd: CommentSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        CommentSubcommand::Create {
            slug,
            body,
            task,
            artifact,
            flag,
            by,
        } => create(
            root,
            &slug,
            &body,
            task.as_deref(),
            artifact.as_deref(),
            flag.as_deref(),
            by.as_deref(),
            json,
        ),
        CommentSubcommand::List { slug, task } => list(root, &slug, task.as_deref(), json),
        CommentSubcommand::Resolve { slug, comment_id } => resolve(root, &slug, &comment_id, json),
    }
}

fn parse_flag(s: &str) -> anyhow::Result<CommentFlag> {
    match s {
        "blocker" => Ok(CommentFlag::Blocker),
        "question" => Ok(CommentFlag::Question),
        "decision" => Ok(CommentFlag::Decision),
        "fyi" => Ok(CommentFlag::Fyi),
        other => anyhow::bail!(
            "unknown flag '{}' — valid values: blocker, question, decision, fyi",
            other
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn create(
    root: &Path,
    slug: &str,
    body: &str,
    task: Option<&str>,
    artifact: Option<&str>,
    flag: Option<&str>,
    by: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;

    let target = if let Some(task_id) = task {
        CommentTarget::Task {
            task_id: task_id.to_string(),
        }
    } else if let Some(art) = artifact {
        let artifact_type: ArtifactType = art
            .parse()
            .with_context(|| format!("unknown artifact type '{art}'"))?;
        CommentTarget::Artifact { artifact_type }
    } else {
        CommentTarget::Feature
    };

    let comment_flag = flag.map(parse_flag).transpose()?;
    let id = add_comment(
        &mut feature.comments,
        &mut feature.next_comment_seq,
        body,
        comment_flag,
        target,
        by.map(str::to_string),
    );
    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "comment_id": id,
            "body": body,
        }))?;
    } else {
        println!("Added comment [{id}]");
    }
    Ok(())
}

fn list(root: &Path, slug: &str, task: Option<&str>, json: bool) -> anyhow::Result<()> {
    let feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;

    let comments: Vec<_> = feature
        .comments
        .iter()
        .filter(|c| match task {
            Some(tid) => {
                matches!(&c.target, CommentTarget::Task { task_id } if task_id == tid)
            }
            None => true,
        })
        .collect();

    if json {
        print_json(&comments)?;
        return Ok(());
    }

    if comments.is_empty() {
        let scope = task.map(|t| format!(" on task {t}")).unwrap_or_default();
        println!("No comments{scope} for '{slug}'.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = comments
        .iter()
        .map(|c| {
            vec![
                c.id.clone(),
                c.flag.as_ref().map(|f| f.to_string()).unwrap_or_default(),
                c.target.to_string(),
                c.author.clone().unwrap_or_default(),
                c.body.clone(),
            ]
        })
        .collect();
    print_table(&["ID", "FLAG", "TARGET", "AUTHOR", "BODY"], rows);
    Ok(())
}

fn resolve(root: &Path, slug: &str, comment_id: &str, json: bool) -> anyhow::Result<()> {
    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;

    let removed = resolve_comment(&mut feature.comments, comment_id);
    if !removed {
        anyhow::bail!("comment '{}' not found on feature '{}'", comment_id, slug);
    }

    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "comment_id": comment_id,
            "resolved": true,
        }))?;
    } else {
        println!("Resolved comment [{comment_id}] on feature '{slug}'.");
    }
    Ok(())
}
