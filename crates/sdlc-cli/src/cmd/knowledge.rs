use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::knowledge::{self, KnowledgeEntry, KnowledgeStatus, OriginKind, Source, SourceType};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum KnowledgeLibrarianSubcommand {
    /// Initialize the knowledge base by harvesting completed workspaces (idempotent)
    Init,
}

const EMPTY_STATE_MSG: &str =
    "Knowledge base not initialized. Run `sdlc knowledge librarian init` to seed from your project.";

#[derive(Subcommand)]
pub enum KnowledgeSubcommand {
    /// Show knowledge base status (initialized, entry count, catalog size, last maintained)
    Status,

    /// Add a new knowledge entry
    Add {
        /// Entry title (required)
        #[arg(long)]
        title: String,
        /// Classification code (e.g. 100.20); defaults to uncategorized
        #[arg(long)]
        code: Option<String>,
        /// Inline text content
        #[arg(long, conflicts_with_all = ["from_url", "from_file"])]
        content: Option<String>,
        /// URL to fetch content from (fetches page title via HTTP, best-effort)
        #[arg(long, conflicts_with_all = ["content", "from_file"])]
        from_url: Option<String>,
        /// File path to read content from
        #[arg(long, conflicts_with_all = ["content", "from_url"])]
        from_file: Option<PathBuf>,
    },

    /// List knowledge entries
    List {
        /// Filter by code prefix (e.g. 100)
        #[arg(long)]
        code_prefix: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Filter by status (draft or published)
        #[arg(long)]
        status: Option<String>,
    },

    /// Show a knowledge entry
    Show { slug: String },

    /// Full-text search across knowledge entries
    Search { query: String },

    /// Update a knowledge entry
    Update {
        slug: String,
        /// New classification code
        #[arg(long)]
        code: Option<String>,
        /// New status (draft or published)
        #[arg(long)]
        status: Option<String>,
        /// Append a tag
        #[arg(long)]
        tag: Option<String>,
        /// Append a related slug or code
        #[arg(long)]
        related: Option<String>,
        /// Update summary
        #[arg(long)]
        summary: Option<String>,
    },

    /// Manage the knowledge catalog taxonomy
    Catalog {
        #[command(subcommand)]
        subcommand: KnowledgeCatalogSubcommand,
    },

    /// Manage session logs for a knowledge entry
    Session {
        #[command(subcommand)]
        subcommand: KnowledgeSessionSubcommand,
    },

    /// Manage the knowledge librarian
    Librarian {
        #[command(subcommand)]
        subcommand: KnowledgeLibrarianSubcommand,
    },
}

#[derive(Subcommand)]
pub enum KnowledgeCatalogSubcommand {
    /// Show the taxonomy tree
    Show,
    /// Add a class (NNN) or division (NNN.NN) to the catalog
    Add {
        /// Classification code (NNN for class, NNN.NN for division)
        #[arg(long)]
        code: String,
        /// Category name
        #[arg(long)]
        name: String,
        /// Optional description
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum KnowledgeSessionSubcommand {
    /// Write a session log for a knowledge entry
    Log {
        slug: String,
        /// Markdown content (reads from stdin if omitted)
        #[arg(long, conflicts_with = "file")]
        content: Option<String>,
        /// Path to a Markdown file to use as the session content
        #[arg(long, conflicts_with = "content")]
        file: Option<PathBuf>,
    },
    /// List sessions for a knowledge entry
    List { slug: String },
    /// Read a specific session
    Read { slug: String, number: u32 },
}

pub fn run(root: &Path, subcmd: KnowledgeSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        KnowledgeSubcommand::Status => status(root, json),
        KnowledgeSubcommand::Add {
            title,
            code,
            content,
            from_url,
            from_file,
        } => add(
            root,
            &title,
            code.as_deref(),
            content.as_deref(),
            from_url.as_deref(),
            from_file.as_deref(),
            json,
        ),
        KnowledgeSubcommand::List {
            code_prefix,
            tag,
            status,
        } => list(
            root,
            code_prefix.as_deref(),
            tag.as_deref(),
            status.as_deref(),
            json,
        ),
        KnowledgeSubcommand::Show { slug } => show(root, &slug, json),
        KnowledgeSubcommand::Search { query } => search(root, &query, json),
        KnowledgeSubcommand::Update {
            slug,
            code,
            status,
            tag,
            related,
            summary,
        } => update(
            root,
            &slug,
            code.as_deref(),
            status.as_deref(),
            tag.as_deref(),
            related.as_deref(),
            summary.as_deref(),
            json,
        ),
        KnowledgeSubcommand::Catalog { subcommand } => match subcommand {
            KnowledgeCatalogSubcommand::Show => catalog_show(root, json),
            KnowledgeCatalogSubcommand::Add {
                code,
                name,
                description,
            } => catalog_add(root, &code, &name, description.as_deref(), json),
        },
        KnowledgeSubcommand::Session { subcommand } => match subcommand {
            KnowledgeSessionSubcommand::Log {
                slug,
                content,
                file,
            } => session_log(root, &slug, content.as_deref(), file.as_deref(), json),
            KnowledgeSessionSubcommand::List { slug } => session_list(root, &slug, json),
            KnowledgeSessionSubcommand::Read { slug, number } => {
                session_read(root, &slug, number, json)
            }
        },
        KnowledgeSubcommand::Librarian { subcommand } => run_librarian(root, subcommand, json),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

fn status(root: &Path, json: bool) -> anyhow::Result<()> {
    let knowledge_dir = root.join(".sdlc/knowledge");
    let entries = knowledge::list(root).context("failed to list knowledge entries")?;

    if !knowledge_dir.exists() || entries.is_empty() {
        if json {
            print_json(&serde_json::json!({
                "initialized": false,
                "message": EMPTY_STATE_MSG,
            }))?;
        } else {
            println!("{EMPTY_STATE_MSG}");
        }
        return Ok(());
    }

    let catalog = knowledge::load_catalog(root).context("failed to load catalog")?;
    let log = knowledge::load_maintenance_log(root).context("failed to load maintenance log")?;

    let entry_count = entries.len();
    let class_count = catalog.classes.len();
    let last_maintained = log
        .actions
        .last()
        .map(|a| a.timestamp.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "never".to_string());

    if json {
        print_json(&serde_json::json!({
            "initialized": true,
            "entry_count": entry_count,
            "catalog_class_count": class_count,
            "last_maintained": last_maintained,
        }))?;
    } else {
        println!("Knowledge base: initialized");
        println!("Entries:        {entry_count}");
        println!("Catalog:        {class_count} top-level classes");
        println!("Last maintained: {last_maintained}");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn add(
    root: &Path,
    title: &str,
    code: Option<&str>,
    content: Option<&str>,
    from_url: Option<&str>,
    from_file: Option<&Path>,
    json: bool,
) -> anyhow::Result<()> {
    let slug = slugify_title(title);
    let entry_code = code.unwrap_or("uncategorized");

    let mut entry = knowledge::create(root, &slug, title, entry_code)
        .with_context(|| format!("failed to create knowledge entry '{slug}'"))?;

    // Write content and update sources based on input mode
    match (content, from_url, from_file) {
        (Some(text), _, _) => {
            knowledge::append_content(root, &slug, text).context("failed to write content")?;
            // origin stays Manual (the default from create())
        }
        (_, Some(url), _) => {
            let page_title = fetch_page_title(url);
            let mut text = format!("Fetched from: {url}");
            if let Some(ref title_str) = page_title {
                text.push_str(&format!("\n\nPage title: {title_str}"));
            }
            knowledge::append_content(root, &slug, &text).context("failed to write URL content")?;

            // Update entry with Web source and origin
            entry.sources.push(Source {
                source_type: SourceType::Web,
                url: Some(url.to_string()),
                path: None,
                workspace: None,
                captured_at: chrono::Utc::now(),
            });
            entry.origin = OriginKind::Web;
            knowledge::save(root, &entry).context("failed to save entry")?;
        }
        (_, _, Some(path)) => {
            let file_content = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read file '{}'", path.display()))?;
            knowledge::append_content(root, &slug, &file_content)
                .context("failed to write file content")?;

            // Update entry with LocalFile source
            entry.sources.push(Source {
                source_type: SourceType::LocalFile,
                url: None,
                path: Some(path.display().to_string()),
                workspace: None,
                captured_at: chrono::Utc::now(),
            });
            knowledge::save(root, &entry).context("failed to save entry")?;
        }
        _ => {}
    }

    if json {
        print_json(&entry_to_json_summary(&entry))?;
    } else {
        println!("Created knowledge entry '{slug}'.");
    }
    Ok(())
}

fn list(
    root: &Path,
    code_prefix: Option<&str>,
    tag: Option<&str>,
    status_filter: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let mut entries = match code_prefix {
        Some(prefix) => {
            knowledge::list_by_code_prefix(root, prefix).context("failed to list knowledge")?
        }
        None => knowledge::list(root).context("failed to list knowledge")?,
    };

    if let Some(t) = tag {
        entries.retain(|e| e.tags.iter().any(|et| et == t));
    }
    if let Some(s) = status_filter {
        let status: KnowledgeStatus = s.parse().with_context(|| format!("invalid status: {s}"))?;
        entries.retain(|e| e.status == status);
    }

    if entries.is_empty() {
        if json {
            print_json(&serde_json::json!([]))?;
        } else {
            println!("{EMPTY_STATE_MSG}");
        }
        return Ok(());
    }

    if json {
        let items: Vec<serde_json::Value> = entries.iter().map(entry_to_json_summary).collect();
        print_json(&items)?;
        return Ok(());
    }

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|e| {
            let title_trunc: String = e.title.chars().take(30).collect();
            let summary_trunc: String = e
                .summary
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect();
            vec![
                e.code.clone(),
                title_trunc,
                summary_trunc,
                e.status.to_string(),
                e.updated_at.format("%Y-%m-%d").to_string(),
            ]
        })
        .collect();
    print_table(&["CODE", "TITLE", "SUMMARY", "STATUS", "UPDATED"], rows);
    Ok(())
}

fn show(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let entry = knowledge::load(root, slug)
        .with_context(|| format!("knowledge entry '{slug}' not found"))?;
    let content = knowledge::read_content(root, slug).unwrap_or_default();

    if json {
        let mut val = entry_to_json_full(&entry);
        val["content"] = serde_json::json!(content);
        print_json(&val)?;
        return Ok(());
    }

    println!("[{}] {}", entry.code, entry.title);
    if let Some(s) = &entry.summary {
        println!("\n{s}");
    }
    if !entry.tags.is_empty() {
        println!("\nTags: {}", entry.tags.join(", "));
    }
    if !entry.sources.is_empty() {
        println!("\nSources:");
        for source in &entry.sources {
            match source.source_type {
                SourceType::Web => {
                    println!("  - web: {}", source.url.as_deref().unwrap_or("(unknown)"));
                }
                SourceType::LocalFile => {
                    println!(
                        "  - file: {}",
                        source.path.as_deref().unwrap_or("(unknown)")
                    );
                }
                ref st => {
                    println!("  - {st}");
                }
            }
        }
    }
    if !content.is_empty() {
        println!("\n---\n{content}");
    }
    Ok(())
}

fn search(root: &Path, query: &str, json: bool) -> anyhow::Result<()> {
    let results =
        knowledge::full_text_search(root, query).context("failed to search knowledge base")?;

    if results.is_empty() {
        if json {
            print_json(&serde_json::json!([]))?;
        } else {
            let entries = knowledge::list(root).unwrap_or_default();
            if entries.is_empty() {
                println!("{EMPTY_STATE_MSG}");
            } else {
                println!("No results found for '{query}'.");
            }
        }
        return Ok(());
    }

    if json {
        let items: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "slug": r.entry.slug,
                    "title": r.entry.title,
                    "code": r.entry.code,
                    "excerpt": r.excerpt,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    let rows: Vec<Vec<String>> = results
        .iter()
        .map(|r| {
            let title_trunc: String = r.entry.title.chars().take(30).collect();
            let excerpt_trunc: String = r.excerpt.chars().take(60).collect();
            vec![r.entry.slug.clone(), title_trunc, excerpt_trunc]
        })
        .collect();
    print_table(&["SLUG", "TITLE", "EXCERPT"], rows);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn update(
    root: &Path,
    slug: &str,
    code: Option<&str>,
    status_str: Option<&str>,
    tag: Option<&str>,
    related: Option<&str>,
    summary: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let status = status_str
        .map(|s| {
            s.parse::<KnowledgeStatus>()
                .with_context(|| format!("invalid status: {s}"))
        })
        .transpose()?;

    let tags_add: Vec<String> = tag.map(|t| vec![t.to_string()]).unwrap_or_default();
    let related_add: Vec<String> = related.map(|r| vec![r.to_string()]).unwrap_or_default();

    let entry = knowledge::update(
        root,
        slug,
        None,
        code,
        status,
        summary,
        &tags_add,
        &related_add,
    )
    .with_context(|| format!("failed to update entry '{slug}'"))?;

    if json {
        print_json(&entry_to_json_summary(&entry))?;
    } else {
        println!("Updated knowledge entry '{slug}'.");
    }
    Ok(())
}

fn catalog_show(root: &Path, json: bool) -> anyhow::Result<()> {
    let catalog = knowledge::load_catalog(root).context("failed to load catalog")?;

    if catalog.classes.is_empty() {
        if json {
            print_json(&serde_json::json!({ "classes": [] }))?;
        } else {
            println!(
                "No catalog defined. Run sdlc knowledge librarian init or sdlc knowledge catalog add."
            );
        }
        return Ok(());
    }

    if json {
        let classes: Vec<serde_json::Value> = catalog
            .classes
            .iter()
            .map(|c| {
                let divisions: Vec<serde_json::Value> = c
                    .divisions
                    .iter()
                    .map(|d| {
                        serde_json::json!({
                            "code": d.code,
                            "name": d.name,
                            "description": d.description,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "code": c.code,
                    "name": c.name,
                    "description": c.description,
                    "divisions": divisions,
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "classes": classes,
            "updated_at": catalog.updated_at,
        }))?;
        return Ok(());
    }

    for class in &catalog.classes {
        println!("[{}] {}", class.code, class.name);
        if let Some(desc) = &class.description {
            println!("    {desc}");
        }
        for div in &class.divisions {
            println!("  [{}] {}", div.code, div.name);
            if let Some(desc) = &div.description {
                println!("      {desc}");
            }
        }
    }
    Ok(())
}

fn catalog_add(
    root: &Path,
    code: &str,
    name: &str,
    description: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let catalog = if code.contains('.') {
        // Division: parent class is everything before the first dot
        let class_code = code.split('.').next().unwrap_or(code);
        knowledge::add_division(root, class_code, code, name, description)
            .context("failed to add division")?
    } else {
        knowledge::add_class(root, code, name, description).context("failed to add class")?
    };

    if json {
        print_json(&serde_json::json!({
            "code": code,
            "name": name,
            "class_count": catalog.classes.len(),
        }))?;
    } else if code.contains('.') {
        println!("Added division [{code}] '{name}' to catalog.");
    } else {
        println!("Added class [{code}] '{name}' to catalog.");
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
    use std::io::Read as _;
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

    let n = knowledge::log_session(root, slug, &body)
        .with_context(|| format!("failed to log session for knowledge entry '{slug}'"))?;

    if json {
        print_json(&serde_json::json!({ "slug": slug, "session": n, "logged": true }))?;
    } else {
        println!("Logged session {n} for knowledge entry '{slug}'.");
    }
    Ok(())
}

fn session_list(root: &Path, slug: &str, json: bool) -> anyhow::Result<()> {
    let sessions = knowledge::list_sessions(root, slug)
        .with_context(|| format!("failed to list sessions for '{slug}'"))?;

    if json {
        let items: Vec<serde_json::Value> = sessions
            .iter()
            .map(|s| {
                serde_json::json!({
                    "session": s.session,
                    "timestamp": s.timestamp,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if sessions.is_empty() {
        println!("No sessions for knowledge entry '{slug}'.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = sessions
        .iter()
        .map(|s| {
            vec![
                s.session.to_string(),
                s.timestamp.format("%Y-%m-%d %H:%M").to_string(),
            ]
        })
        .collect();
    print_table(&["#", "DATE"], rows);
    Ok(())
}

fn session_read(root: &Path, slug: &str, number: u32, json: bool) -> anyhow::Result<()> {
    let content = knowledge::read_session(root, slug, number)
        .with_context(|| format!("session {number} not found for knowledge entry '{slug}'"))?;

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

fn run_librarian(
    root: &Path,
    subcmd: KnowledgeLibrarianSubcommand,
    json: bool,
) -> anyhow::Result<()> {
    match subcmd {
        KnowledgeLibrarianSubcommand::Init => {
            let report = knowledge::librarian_init(root).context("librarian init failed")?;

            let inv_new: usize = report
                .investigation_results
                .iter()
                .filter(|r| r.created)
                .count();
            let inv_upd: usize = report.investigation_results.len() - inv_new;
            let ponder_new: usize = report.ponder_results.iter().filter(|r| r.created).count();
            let ponder_upd: usize = report.ponder_results.len() - ponder_new;
            let guideline_count = report.guideline_results.len();

            if json {
                print_json(&serde_json::json!({
                    "investigations_new": inv_new,
                    "investigations_updated": inv_upd,
                    "ponders_new": ponder_new,
                    "ponders_updated": ponder_upd,
                    "guidelines": guideline_count,
                    "catalog_created": report.catalog_created,
                    "catalog_class_count": report.catalog_class_count,
                    "cross_ref_count": report.cross_ref_count,
                    "agent_file": report.agent_file_path.display().to_string(),
                }))?;
            } else {
                println!("Knowledge base initialized");
                println!(
                    "  Investigations harvested: {} ({} new, {} updated)",
                    report.investigation_results.len(),
                    inv_new,
                    inv_upd
                );
                println!(
                    "  Ponders harvested:        {} ({} new, {} updated)",
                    report.ponder_results.len(),
                    ponder_new,
                    ponder_upd
                );
                println!("  Guidelines linked:        {guideline_count}");
                println!(
                    "  Catalog:                  {} classes (created: {})",
                    report.catalog_class_count,
                    if report.catalog_created { "yes" } else { "no" }
                );
                println!("  Cross-references added:   {}", report.cross_ref_count);
                println!(
                    "  Librarian agent:          {}",
                    report.agent_file_path.display()
                );
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Derive a URL-safe slug from a title.
/// Lowercase, replace non-alnum chars with `-`, strip leading/trailing `-`, truncate at 40.
fn slugify_title(title: &str) -> String {
    let lower = title.to_lowercase();
    let mut result = String::new();
    let mut last_was_dash = false;
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c);
            last_was_dash = false;
        } else if !last_was_dash && !result.is_empty() {
            result.push('-');
            last_was_dash = true;
        }
    }
    while result.ends_with('-') {
        result.pop();
    }
    result.chars().take(40).collect()
}

/// Fetch the `<title>` tag from a URL. Best-effort — returns `None` on any failure.
fn fetch_page_title(url: &str) -> Option<String> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .ok()?;
    let body = response.into_string().ok()?;
    let lower = body.to_lowercase();
    let start = lower.find("<title>")? + 7;
    let relative_end = lower[start..].find("</title>")?;
    let title = body[start..start + relative_end].trim().to_string();
    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

// ---------------------------------------------------------------------------
// JSON helpers
// ---------------------------------------------------------------------------

fn entry_to_json_summary(e: &KnowledgeEntry) -> serde_json::Value {
    serde_json::json!({
        "slug": e.slug,
        "title": e.title,
        "code": e.code,
        "status": e.status.to_string(),
        "summary": e.summary,
        "tags": e.tags,
        "created_at": e.created_at,
        "updated_at": e.updated_at,
    })
}

fn entry_to_json_full(e: &KnowledgeEntry) -> serde_json::Value {
    let sources: Vec<serde_json::Value> = e
        .sources
        .iter()
        .map(|s| {
            serde_json::json!({
                "type": s.source_type.to_string(),
                "url": s.url,
                "path": s.path,
                "workspace": s.workspace,
                "captured_at": s.captured_at,
            })
        })
        .collect();

    serde_json::json!({
        "slug": e.slug,
        "title": e.title,
        "code": e.code,
        "status": e.status.to_string(),
        "summary": e.summary,
        "tags": e.tags,
        "sources": sources,
        "related": e.related,
        "origin": e.origin.to_string(),
        "harvested_from": e.harvested_from,
        "last_verified_at": e.last_verified_at,
        "staleness_flags": e.staleness_flags,
        "created_at": e.created_at,
        "updated_at": e.updated_at,
    })
}
