use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::investigation::{self, InvestigationEntry, InvestigationKind, InvestigationStatus};
use std::io::Read as _;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum InvestigateSubcommand {
    /// Create a new investigation
    Create {
        slug: String,
        /// Investigation title
        #[arg(long)]
        title: String,
        /// Investigation kind: root-cause, evolve, or guideline
        #[arg(long)]
        kind: String,
        /// Initial problem description or context
        #[arg(long)]
        context: Option<String>,
    },
    /// List investigations
    List {
        /// Filter by kind (root-cause, evolve, guideline)
        #[arg(long)]
        kind: Option<String>,
        /// Filter by status (in_progress, complete, parked)
        #[arg(long)]
        status: Option<String>,
    },
    /// Show investigation details
    Show { slug: String },
    /// Capture content as a workspace artifact
    Capture {
        slug: String,
        /// Content to write (provide this or --file)
        #[arg(long, conflicts_with = "file")]
        content: Option<String>,
        /// File to copy into the workspace
        #[arg(long, conflicts_with = "content")]
        file: Option<PathBuf>,
        /// Filename to use (defaults to source filename)
        #[arg(long = "as")]
        filename: Option<String>,
    },
    /// Update investigation metadata
    Update {
        slug: String,
        /// Advance to a new phase (e.g. "investigate", "synthesize", "output", "done")
        #[arg(long)]
        phase: Option<String>,
        /// Update status (in_progress, complete, parked)
        #[arg(long)]
        status: Option<String>,
        /// Update scope (evolve) or guideline scope (guideline)
        #[arg(long)]
        scope: Option<String>,
        /// Update confidence score 0–100 (root-cause)
        #[arg(long)]
        confidence: Option<u32>,
    },
    /// List artifacts in the investigation workspace
    Artifacts { slug: String },
    /// Manage session logs
    Session {
        #[command(subcommand)]
        subcommand: InvestigateSessionSubcommand,
    },
}

#[derive(Subcommand)]
pub enum InvestigateSessionSubcommand {
    /// Write a session log for an investigation
    Log {
        slug: String,
        /// Markdown content for the session (reads from stdin if omitted)
        #[arg(long, conflicts_with = "file")]
        content: Option<String>,
        /// Path to a Markdown file to use as the session content
        #[arg(long, conflicts_with = "content")]
        file: Option<PathBuf>,
    },
    /// List session metadata for an investigation
    List { slug: String },
    /// Read the full content of a session
    Read {
        slug: String,
        /// Session number to read
        number: u32,
    },
}

pub fn run(root: &Path, subcmd: InvestigateSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        InvestigateSubcommand::Create {
            slug,
            title,
            kind,
            context,
        } => create(root, &slug, &title, &kind, context, json),
        InvestigateSubcommand::List { kind, status } => {
            list(root, kind.as_deref(), status.as_deref(), json)
        }
        InvestigateSubcommand::Show { slug } => show(root, &slug, json),
        InvestigateSubcommand::Capture {
            slug,
            content,
            file,
            filename,
        } => capture(
            root,
            &slug,
            content.as_deref(),
            file.as_deref(),
            filename.as_deref(),
            json,
        ),
        InvestigateSubcommand::Update {
            slug,
            phase,
            status,
            scope,
            confidence,
        } => update(
            root,
            &slug,
            phase.as_deref(),
            status.as_deref(),
            scope.as_deref(),
            confidence,
            json,
        ),
        InvestigateSubcommand::Artifacts { slug } => artifacts(root, &slug, json),
        InvestigateSubcommand::Session { subcommand } => match subcommand {
            InvestigateSessionSubcommand::Log {
                slug,
                content,
                file,
            } => session_log(root, &slug, content.as_deref(), file.as_deref(), json),
            InvestigateSessionSubcommand::List { slug } => session_list(root, &slug, json),
            InvestigateSessionSubcommand::Read { slug, number } => {
                session_read(root, &slug, number, json)
            }
        },
    }
}

fn create(
    root: &Path,
    slug: &str,
    title: &str,
    kind_str: &str,
    context: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let kind: InvestigationKind = kind_str
        .parse()
        .with_context(|| format!("invalid kind: {kind_str}"))?;

    let entry = investigation::create(root, slug, title, kind, context)
        .with_context(|| format!("failed to create investigation '{slug}'"))?;

    if json {
        print_json(&serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "kind": entry.kind.to_string(),
            "phase": entry.phase,
            "status": entry.status.to_string(),
        }))?;
    } else {
        println!(
            "Created {} investigation '{}': {}",
            entry.kind, entry.slug, entry.title
        );
        println!("Phase: {} | Status: {}", entry.phase, entry.status);
    }
    Ok(())
}

fn list(
    root: &Path,
    kind_filter: Option<&str>,
    status_filter: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let kind_opt: Option<InvestigationKind> = kind_filter
        .map(|k| k.parse().with_context(|| format!("invalid kind: {k}")))
        .transpose()?;

    let status_opt: Option<InvestigationStatus> = status_filter
        .map(|s| s.parse().with_context(|| format!("invalid status: {s}")))
        .transpose()?;

    let mut entries = match kind_opt {
        Some(k) => investigation::list_by_kind(root, k).context("failed to list investigations")?,
        None => investigation::list(root).context("failed to list investigations")?,
    };

    if let Some(status) = status_opt {
        entries.retain(|e| e.status == status);
    }

    if json {
        let items: Vec<serde_json::Value> = entries.iter().map(entry_to_json_summary).collect();
        print_json(&items)?;
        return Ok(());
    }

    if entries.is_empty() {
        println!("No investigations.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|e| {
            vec![
                e.slug.clone(),
                e.title.clone(),
                e.kind.to_string(),
                e.phase.clone(),
                e.status.to_string(),
                e.sessions.to_string(),
            ]
        })
        .collect();
    print_table(
        &["SLUG", "TITLE", "KIND", "PHASE", "STATUS", "SESSIONS"],
        rows,
    );
    Ok(())
}

fn show(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let entry = investigation::load(root, slug)
        .with_context(|| format!("investigation '{slug}' not found"))?;
    let artifacts =
        investigation::list_artifacts(root, slug).context("failed to list artifacts")?;

    if json {
        let mut val = entry_to_json_full(&entry);
        let artifact_list: Vec<serde_json::Value> = artifacts
            .iter()
            .map(|a| {
                serde_json::json!({
                    "filename": a.filename,
                    "size_bytes": a.size_bytes,
                    "modified_at": a.modified_at,
                })
            })
            .collect();
        val["artifacts"] = serde_json::json!(artifact_list);
        print_json(&val)?;
        return Ok(());
    }

    println!(
        "Investigation: {} — {} ({})",
        entry.slug, entry.title, entry.kind
    );
    println!("Phase:   {} | Status: {}", entry.phase, entry.status);
    println!("Sessions: {}", entry.sessions);
    if let Some(ctx) = &entry.context {
        println!("Context: {ctx}");
    }
    if let Some(o) = &entry.orientation {
        println!("\nOrientation:");
        println!("  WHERE: {}", o.current);
        println!("  NEXT:  {}", o.next);
        println!("  COMMIT: {}", o.commit);
    }
    println!("\nArtifacts: {}", artifacts.len());
    for a in &artifacts {
        println!("  {} ({} bytes)", a.filename, a.size_bytes);
    }
    Ok(())
}

fn capture(
    root: &Path,
    slug: &str,
    content: Option<&str>,
    file: Option<&Path>,
    filename_override: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let target_filename;

    match (content, file) {
        (Some(c), _) => {
            target_filename = filename_override.unwrap_or("capture.md").to_string();
            investigation::capture_content(root, slug, &target_filename, c)
                .with_context(|| format!("failed to capture content to '{target_filename}'"))?;
        }
        (None, Some(f)) => {
            target_filename = filename_override.map(|s| s.to_string()).unwrap_or_else(|| {
                f.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "capture".to_string())
            });
            let body = std::fs::read_to_string(f)
                .with_context(|| format!("failed to read '{}'", f.display()))?;
            investigation::capture_content(root, slug, &target_filename, &body)
                .with_context(|| format!("failed to capture '{target_filename}'"))?;
        }
        (None, None) => {
            anyhow::bail!("provide --content or --file");
        }
    }

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "filename": target_filename,
            "captured": true,
        }))?;
    } else {
        println!("Captured '{target_filename}' into investigation '{slug}'.");
    }
    Ok(())
}

fn update(
    root: &Path,
    slug: &str,
    phase: Option<&str>,
    status_str: Option<&str>,
    scope: Option<&str>,
    confidence: Option<u32>,
    json: bool,
) -> anyhow::Result<()> {
    if phase.is_none() && status_str.is_none() && scope.is_none() && confidence.is_none() {
        anyhow::bail!("nothing to update: provide --phase, --status, --scope, or --confidence");
    }

    let mut entry = investigation::load(root, slug)
        .with_context(|| format!("investigation '{slug}' not found"))?;

    if let Some(p) = phase {
        entry.phase = p.to_string();
        entry.updated_at = chrono::Utc::now();
    }
    if let Some(s) = status_str {
        let new_status: InvestigationStatus =
            s.parse().with_context(|| format!("invalid status: {s}"))?;
        entry.status = new_status;
        entry.updated_at = chrono::Utc::now();
    }
    if let Some(s) = scope {
        match entry.kind {
            InvestigationKind::Evolve => entry.scope = Some(s.to_string()),
            InvestigationKind::Guideline => entry.guideline_scope = Some(s.to_string()),
            _ => anyhow::bail!("--scope is only valid for evolve and guideline investigations"),
        }
        entry.updated_at = chrono::Utc::now();
    }
    if let Some(c) = confidence {
        if entry.kind != InvestigationKind::RootCause {
            anyhow::bail!("--confidence is only valid for root-cause investigations");
        }
        entry.confidence = Some(c);
        entry.updated_at = chrono::Utc::now();
    }

    investigation::save(root, &entry).context("failed to save investigation")?;

    if json {
        print_json(&entry_to_json_summary(&entry))?;
    } else {
        println!("Updated investigation '{slug}'.");
    }
    Ok(())
}

fn artifacts(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let artifacts = investigation::list_artifacts(root, slug)
        .with_context(|| format!("failed to list artifacts for '{slug}'"))?;

    if json {
        let items: Vec<serde_json::Value> = artifacts
            .iter()
            .map(|a| {
                serde_json::json!({
                    "filename": a.filename,
                    "size_bytes": a.size_bytes,
                    "modified_at": a.modified_at,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if artifacts.is_empty() {
        println!("No artifacts for investigation '{slug}'.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = artifacts
        .iter()
        .map(|a| {
            vec![
                a.filename.clone(),
                format!("{} bytes", a.size_bytes),
                a.modified_at.format("%Y-%m-%d %H:%M").to_string(),
            ]
        })
        .collect();
    print_table(&["FILENAME", "SIZE", "MODIFIED"], rows);
    Ok(())
}

fn session_log(
    root: &Path,
    slug: &str,
    content: Option<&str>,
    file: Option<&Path>,
    json: bool,
) -> anyhow::Result<()> {
    let body = match (content, file) {
        (Some(c), _) => c.to_string(),
        (None, Some(f)) => std::fs::read_to_string(f)
            .with_context(|| format!("failed to read session file '{}'", f.display()))?,
        (None, None) => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .context("failed to read session content from stdin")?;
            buf
        }
    };

    let n = investigation::log_session(root, slug, &body)
        .with_context(|| format!("failed to log session for '{slug}'"))?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "session": n,
            "logged": true,
        }))?;
    } else {
        println!("Logged session {n} for investigation '{slug}'.");
    }
    Ok(())
}

fn session_list(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let sessions = investigation::list_sessions(root, slug)
        .with_context(|| format!("failed to list sessions for '{slug}'"))?;

    if json {
        let items: Vec<serde_json::Value> = sessions
            .iter()
            .map(|s| {
                serde_json::json!({
                    "session": s.session,
                    "timestamp": s.timestamp,
                    "orientation": s.orientation.as_ref().map(|o| serde_json::json!({
                        "current": o.current,
                        "next": o.next,
                        "commit": o.commit,
                    })),
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if sessions.is_empty() {
        println!("No sessions for investigation '{slug}'.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = sessions
        .iter()
        .map(|s| {
            vec![
                s.session.to_string(),
                s.timestamp.format("%Y-%m-%d %H:%M").to_string(),
                s.orientation
                    .as_ref()
                    .map(|o| o.current.clone())
                    .unwrap_or_default(),
            ]
        })
        .collect();
    print_table(&["#", "DATE", "WHERE WE ARE"], rows);
    Ok(())
}

fn session_read(root: &Path, slug: &str, number: u32, json: bool) -> anyhow::Result<()> {
    let content = investigation::read_session(root, slug, number)
        .with_context(|| format!("session {number} not found for investigation '{slug}'"))?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "session": number,
            "content": content,
        }))?;
    } else {
        print!("{content}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// JSON helpers
// ---------------------------------------------------------------------------

fn entry_to_json_summary(e: &InvestigationEntry) -> serde_json::Value {
    serde_json::json!({
        "slug": e.slug,
        "title": e.title,
        "kind": e.kind.to_string(),
        "phase": e.phase,
        "status": e.status.to_string(),
        "sessions": e.sessions,
        "created_at": e.created_at,
        "updated_at": e.updated_at,
    })
}

fn entry_to_json_full(e: &InvestigationEntry) -> serde_json::Value {
    let orientation = e.orientation.as_ref().map(|o| {
        serde_json::json!({
            "current": o.current,
            "next": o.next,
            "commit": o.commit,
        })
    });
    serde_json::json!({
        "slug": e.slug,
        "title": e.title,
        "kind": e.kind.to_string(),
        "phase": e.phase,
        "status": e.status.to_string(),
        "context": e.context,
        "sessions": e.sessions,
        "orientation": orientation,
        "created_at": e.created_at,
        "updated_at": e.updated_at,
        "confidence": e.confidence,
        "output_type": e.output_type,
        "output_ref": e.output_ref,
        "scope": e.scope,
        "lens_scores": e.lens_scores,
        "output_refs": e.output_refs,
        "guideline_scope": e.guideline_scope,
        "problem_statement": e.problem_statement,
        "evidence_counts": e.evidence_counts,
        "principles_count": e.principles_count,
        "publish_path": e.publish_path,
    })
}
