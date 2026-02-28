use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{
    ponder::{PonderEntry, PonderTeamMember},
    state::State,
};
use std::io::Read as _;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum PonderSubcommand {
    /// Create a new ponder entry
    Create {
        slug: String,
        /// Entry title
        #[arg(long)]
        title: String,
        /// Optional brief (written as brief.md)
        #[arg(long)]
        brief: Option<String>,
    },
    /// List ponder entries
    List {
        /// Filter by status (exploring, converging, committed, parked)
        #[arg(long)]
        status: Option<String>,
    },
    /// Show ponder entry details
    Show { slug: String },
    /// Capture content into the scrapbook
    Capture {
        slug: String,
        /// Content to write (provide this or --file)
        #[arg(long, conflicts_with = "file")]
        content: Option<String>,
        /// File to copy into the scrapbook
        #[arg(long, conflicts_with = "content")]
        file: Option<PathBuf>,
        /// Filename to use (defaults to source filename)
        #[arg(long = "as")]
        filename: Option<String>,
    },
    /// Manage thought partners
    Team {
        #[command(subcommand)]
        subcommand: TeamSubcommand,
    },
    /// Update ponder entry metadata
    Update {
        slug: String,
        /// New status
        #[arg(long)]
        status: Option<String>,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// Add tags (repeatable)
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    /// Archive (park) a ponder entry
    Archive { slug: String },
    /// List scrapbook artifacts
    Artifacts { slug: String },
    /// Manage session logs
    Session {
        #[command(subcommand)]
        subcommand: SessionSubcommand,
    },
}

#[derive(Subcommand)]
pub enum SessionSubcommand {
    /// Write a session log for a ponder entry
    Log {
        slug: String,
        /// Markdown content for the session (reads from stdin if omitted)
        #[arg(long, conflicts_with = "file")]
        content: Option<String>,
        /// Path to a Markdown file to use as the session content
        #[arg(long, conflicts_with = "content")]
        file: Option<PathBuf>,
    },
    /// List session metadata for a ponder entry
    List { slug: String },
    /// Read the full content of a session
    Read {
        slug: String,
        /// Session number to read
        number: u32,
    },
}

#[derive(Subcommand)]
pub enum TeamSubcommand {
    /// Add a thought partner
    Add {
        slug: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        role: String,
        #[arg(long)]
        context: String,
        #[arg(long)]
        agent: String,
    },
    /// List thought partners
    List { slug: String },
}

pub fn run(root: &Path, subcmd: PonderSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        PonderSubcommand::Create { slug, title, brief } => {
            create(root, &slug, &title, brief.as_deref(), json)
        }
        PonderSubcommand::List { status } => list(root, status.as_deref(), json),
        PonderSubcommand::Show { slug } => show(root, &slug, json),
        PonderSubcommand::Capture {
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
        PonderSubcommand::Team { subcommand } => match subcommand {
            TeamSubcommand::Add {
                slug,
                name,
                role,
                context,
                agent,
            } => team_add(root, &slug, &name, &role, &context, &agent, json),
            TeamSubcommand::List { slug } => team_list(root, &slug, json),
        },
        PonderSubcommand::Update {
            slug,
            status,
            title,
            tags,
        } => update(
            root,
            &slug,
            status.as_deref(),
            title.as_deref(),
            &tags,
            json,
        ),
        PonderSubcommand::Archive { slug } => archive(root, &slug, json),
        PonderSubcommand::Artifacts { slug } => artifacts(root, &slug, json),
        PonderSubcommand::Session { subcommand } => match subcommand {
            SessionSubcommand::Log {
                slug,
                content,
                file,
            } => session_log(root, &slug, content.as_deref(), file.as_deref(), json),
            SessionSubcommand::List { slug } => session_list(root, &slug, json),
            SessionSubcommand::Read { slug, number } => session_read(root, &slug, number, json),
        },
    }
}

fn create(
    root: &Path,
    slug: &str,
    title: &str,
    brief: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let entry = PonderEntry::create(root, slug, title)
        .with_context(|| format!("failed to create ponder entry '{slug}'"))?;

    if let Some(brief_content) = brief {
        sdlc_core::ponder::capture_content(root, slug, "brief.md", brief_content)
            .context("failed to write brief")?;
    }

    if let Ok(mut state) = State::load(root) {
        state.add_ponder(slug);
        state.save(root).context("failed to save state")?;
    }

    if json {
        print_json(&serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "status": entry.status.to_string(),
        }))?;
    } else {
        println!("Created ponder entry '{slug}'.");
    }
    Ok(())
}

fn list(root: &Path, status_filter: Option<&str>, json: bool) -> anyhow::Result<()> {
    let mut entries = PonderEntry::list(root).context("failed to list ponder entries")?;

    if let Some(status_str) = status_filter {
        let status: sdlc_core::ponder::PonderStatus = status_str
            .parse()
            .with_context(|| format!("invalid status: {status_str}"))?;
        entries.retain(|e| e.status == status);
    }

    if json {
        let items: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "slug": e.slug,
                    "title": e.title,
                    "status": e.status.to_string(),
                    "tags": e.tags,
                    "sessions": e.sessions,
                    "created_at": e.created_at,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if entries.is_empty() {
        println!("No ponder entries.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|e| {
            vec![
                e.slug.clone(),
                e.title.clone(),
                e.status.to_string(),
                e.tags.join(", "),
                e.sessions.to_string(),
            ]
        })
        .collect();
    print_table(&["SLUG", "TITLE", "STATUS", "TAGS", "SESSIONS"], rows);
    Ok(())
}

fn show(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let entry = PonderEntry::load(root, slug)
        .with_context(|| format!("ponder entry '{slug}' not found"))?;
    let team = sdlc_core::ponder::load_team(root, slug).context("failed to load team")?;
    let artifacts =
        sdlc_core::ponder::list_artifacts(root, slug).context("failed to list artifacts")?;

    if json {
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
        let team_list: Vec<serde_json::Value> = team
            .partners
            .iter()
            .map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "role": m.role,
                    "context": m.context,
                    "agent": m.agent,
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "status": entry.status.to_string(),
            "tags": entry.tags,
            "sessions": entry.sessions,
            "committed_at": entry.committed_at,
            "committed_to": entry.committed_to,
            "created_at": entry.created_at,
            "updated_at": entry.updated_at,
            "team": team_list,
            "artifacts": artifact_list,
        }))?;
        return Ok(());
    }

    println!("Ponder: {} — {}", entry.slug, entry.title);
    println!("Status:   {}", entry.status);
    if !entry.tags.is_empty() {
        println!("Tags:     {}", entry.tags.join(", "));
    }
    println!("Sessions: {}", entry.sessions);
    if !entry.committed_to.is_empty() {
        println!("Committed to: {}", entry.committed_to.join(", "));
    }
    println!("Team:     {} partner(s)", team.partners.len());
    for m in &team.partners {
        println!("  {} — {} ({})", m.name, m.role, m.context);
    }
    println!("Artifacts: {}", artifacts.len());
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
            sdlc_core::ponder::capture_content(root, slug, &target_filename, c)
                .with_context(|| format!("failed to capture content to '{target_filename}'"))?;
        }
        (None, Some(f)) => {
            target_filename = filename_override.map(|s| s.to_string()).unwrap_or_else(|| {
                f.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "capture".to_string())
            });
            sdlc_core::ponder::capture_file(root, slug, f, &target_filename)
                .with_context(|| format!("failed to capture file '{}'", f.display()))?;
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
        println!("Captured '{target_filename}' into ponder '{slug}'.");
    }
    Ok(())
}

fn team_add(
    root: &Path,
    slug: &str,
    name: &str,
    role: &str,
    context: &str,
    agent: &str,
    json: bool,
) -> anyhow::Result<()> {
    let member = PonderTeamMember {
        name: name.to_string(),
        role: role.to_string(),
        context: context.to_string(),
        agent: agent.to_string(),
        recruited_at: chrono::Utc::now(),
    };

    let team = sdlc_core::ponder::add_team_member(root, slug, member)
        .with_context(|| format!("failed to add team member to '{slug}'"))?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "name": name,
            "role": role,
            "team_size": team.partners.len(),
        }))?;
    } else {
        println!("Added '{name}' ({role}) to ponder '{slug}' team.");
    }
    Ok(())
}

fn team_list(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let team = sdlc_core::ponder::load_team(root, slug)
        .with_context(|| format!("failed to load team for '{slug}'"))?;

    if json {
        let items: Vec<serde_json::Value> = team
            .partners
            .iter()
            .map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "role": m.role,
                    "context": m.context,
                    "agent": m.agent,
                    "recruited_at": m.recruited_at,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if team.partners.is_empty() {
        println!("No team members for ponder '{slug}'.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = team
        .partners
        .iter()
        .map(|m| {
            vec![
                m.name.clone(),
                m.role.clone(),
                m.context.clone(),
                m.agent.clone(),
            ]
        })
        .collect();
    print_table(&["NAME", "ROLE", "CONTEXT", "AGENT"], rows);
    Ok(())
}

fn update(
    root: &Path,
    slug: &str,
    status: Option<&str>,
    title: Option<&str>,
    tags: &[String],
    json: bool,
) -> anyhow::Result<()> {
    if status.is_none() && title.is_none() && tags.is_empty() {
        anyhow::bail!("nothing to update: provide --status, --title, or --tag");
    }

    let mut entry = PonderEntry::load(root, slug)
        .with_context(|| format!("ponder entry '{slug}' not found"))?;

    if let Some(status_str) = status {
        let new_status: sdlc_core::ponder::PonderStatus = status_str
            .parse()
            .with_context(|| format!("invalid status: {status_str}"))?;
        entry.update_status(new_status);
    }
    if let Some(t) = title {
        entry.update_title(t);
    }
    for tag in tags {
        entry.add_tag(tag);
    }

    entry.save(root).context("failed to save ponder entry")?;

    // Sync active_ponders when status changes to parked/committed
    if matches!(
        entry.status,
        sdlc_core::ponder::PonderStatus::Parked | sdlc_core::ponder::PonderStatus::Committed
    ) {
        if let Ok(mut state) = State::load(root) {
            state.remove_ponder(slug);
            state.save(root).context("failed to save state")?;
        }
    }

    if json {
        print_json(&serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "status": entry.status.to_string(),
            "tags": entry.tags,
        }))?;
    } else {
        println!("Updated ponder entry '{slug}'.");
    }
    Ok(())
}

fn archive(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let mut entry = PonderEntry::load(root, slug)
        .with_context(|| format!("ponder entry '{slug}' not found"))?;

    entry.update_status(sdlc_core::ponder::PonderStatus::Parked);
    entry.save(root).context("failed to save ponder entry")?;

    if let Ok(mut state) = State::load(root) {
        state.remove_ponder(slug);
        state.save(root).context("failed to save state")?;
    }

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "status": "parked",
        }))?;
    } else {
        println!("Archived ponder entry '{slug}'.");
    }
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
            // Read from stdin
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .context("failed to read session content from stdin")?;
            buf
        }
    };

    let n = sdlc_core::ponder::log_session(root, slug, &body)
        .with_context(|| format!("failed to log session for '{slug}'"))?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "session": n,
            "logged": true,
        }))?;
    } else {
        println!("Logged session {n} for ponder '{slug}'.");
    }
    Ok(())
}

fn session_list(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let sessions = sdlc_core::ponder::list_sessions(root, slug)
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
        println!("No sessions for ponder '{slug}'.");
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
    let content = sdlc_core::ponder::read_session(root, slug, number)
        .with_context(|| format!("session {number} not found for ponder '{slug}'"))?;

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

fn artifacts(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let artifacts = sdlc_core::ponder::list_artifacts(root, slug)
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
        println!("No artifacts for ponder '{slug}'.");
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
