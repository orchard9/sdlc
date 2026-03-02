//! Knowledge base data layer.
//!
//! Entries live at `.sdlc/knowledge/<slug>/` — slug-only, no code prefix in path.
//! The classification code lives in `entry.yaml` only, so reclassification
//! requires no filesystem rename.

use crate::error::{Result, SdlcError};
use crate::paths::{
    knowledge_catalog_path, knowledge_content_path, knowledge_dir, knowledge_maintenance_log_path,
    knowledge_manifest, validate_slug, KNOWLEDGE_DIR, MANIFEST_FILE,
};
use crate::workspace;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Whether the entry is still being built out or ready for use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeStatus {
    Draft,
    Published,
}

impl fmt::Display for KnowledgeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KnowledgeStatus::Draft => write!(f, "draft"),
            KnowledgeStatus::Published => write!(f, "published"),
        }
    }
}

impl FromStr for KnowledgeStatus {
    type Err = SdlcError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "draft" => Ok(KnowledgeStatus::Draft),
            "published" => Ok(KnowledgeStatus::Published),
            other => Err(SdlcError::InvalidKnowledgeStatus(other.to_string())),
        }
    }
}

/// Where an entry's content came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Web,
    LocalFile,
    Manual,
    Harvested,
    Guideline,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Web => write!(f, "web"),
            SourceType::LocalFile => write!(f, "local_file"),
            SourceType::Manual => write!(f, "manual"),
            SourceType::Harvested => write!(f, "harvested"),
            SourceType::Guideline => write!(f, "guideline"),
        }
    }
}

/// How the entry was first created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginKind {
    Manual,
    Web,
    Research,
    Harvested,
    Guideline,
}

impl fmt::Display for OriginKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OriginKind::Manual => write!(f, "manual"),
            OriginKind::Web => write!(f, "web"),
            OriginKind::Research => write!(f, "research"),
            OriginKind::Harvested => write!(f, "harvested"),
            OriginKind::Guideline => write!(f, "guideline"),
        }
    }
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// A provenance record for where an entry's content came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: SourceType,
    /// Web sources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Local file sources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Harvested: "investigation/auth-bug" or "ponder/api-design".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    pub captured_at: DateTime<Utc>,
}

/// A knowledge base entry — the unit of institutional memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub slug: String,
    pub title: String,
    /// Dewey-inspired classification code, e.g. "100.20", or "uncategorized".
    pub code: String,
    pub status: KnowledgeStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
    /// Codes or slugs of related entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<String>,
    pub origin: OriginKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harvested_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_verified_at: Option<DateTime<Utc>>,
    /// "url_404" | "code_ref_gone" | "superseded_by" | "aged_out"
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub staleness_flags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A leaf-level section in the catalog taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogSection {
    pub code: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A division within a catalog class, containing optional sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogDivision {
    pub code: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<CatalogSection>,
}

/// A top-level class in the catalog taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogClass {
    pub code: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub divisions: Vec<CatalogDivision>,
}

/// The full knowledge catalog — the taxonomy tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<CatalogClass>,
    pub updated_at: DateTime<Utc>,
}

/// A single recorded action from a librarian maintenance pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceAction {
    pub timestamp: DateTime<Utc>,
    /// "url_check" | "harvest" | "cross_ref" | "catalog_update" | "duplicate_flag"
    pub action_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    pub detail: String,
}

/// The persistent log of all maintenance actions taken by the librarian.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceLog {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<MaintenanceAction>,
}

/// A search hit — an entry plus the first matching excerpt.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: KnowledgeEntry,
    /// First matching line with up to 120 chars of context.
    pub excerpt: String,
}

// ---------------------------------------------------------------------------
// Code validation
// ---------------------------------------------------------------------------

static CODE_RE: OnceLock<Regex> = OnceLock::new();

fn code_re() -> &'static Regex {
    CODE_RE.get_or_init(|| Regex::new(r"^\d{3}(\.\d{1,2}(\.\d)?)?$").unwrap())
}

/// Validate a Dewey-style classification code.
///
/// Accepts `"uncategorized"` or codes matching `NNN`, `NNN.NN`, `NNN.NN.N`.
pub fn validate_code(code: &str) -> Result<()> {
    if code == "uncategorized" || code_re().is_match(code) {
        return Ok(());
    }
    Err(SdlcError::InvalidKnowledgeCode(code.to_string()))
}

// ---------------------------------------------------------------------------
// Entry CRUD
// ---------------------------------------------------------------------------

/// Create a new knowledge entry at `.sdlc/knowledge/<slug>/`.
///
/// Fails if the slug or code is invalid, or if the entry already exists.
pub fn create(
    root: &Path,
    slug: impl Into<String>,
    title: impl Into<String>,
    code: impl Into<String>,
) -> Result<KnowledgeEntry> {
    let slug = slug.into();
    let title = title.into();
    let code = code.into();

    validate_slug(&slug)?;
    validate_code(&code)?;

    let dir = knowledge_dir(root, &slug);
    if dir.exists() {
        return Err(SdlcError::KnowledgeExists(slug));
    }
    std::fs::create_dir_all(&dir)?;

    // Write empty content.md
    let content_path = knowledge_content_path(root, &slug);
    crate::io::atomic_write(&content_path, b"")?;

    let now = Utc::now();
    let entry = KnowledgeEntry {
        slug: slug.clone(),
        title,
        code,
        status: KnowledgeStatus::Draft,
        tags: Vec::new(),
        summary: None,
        sources: Vec::new(),
        related: Vec::new(),
        origin: OriginKind::Manual,
        harvested_from: None,
        last_verified_at: None,
        staleness_flags: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    save(root, &entry)?;
    Ok(entry)
}

/// Load an existing knowledge entry from disk.
pub fn load(root: &Path, slug: &str) -> Result<KnowledgeEntry> {
    validate_slug(slug)?;
    let manifest = knowledge_manifest(root, slug);
    if !manifest.exists() {
        return Err(SdlcError::KnowledgeNotFound(slug.to_string()));
    }
    let raw = std::fs::read_to_string(&manifest)?;
    Ok(serde_yaml::from_str(&raw)?)
}

/// Persist a knowledge entry to disk.
pub fn save(root: &Path, entry: &KnowledgeEntry) -> Result<()> {
    let manifest = knowledge_manifest(root, &entry.slug);
    let raw = serde_yaml::to_string(entry)?;
    crate::io::atomic_write(&manifest, raw.as_bytes())
}

/// List all knowledge entries in the project.
///
/// Skips `catalog.yaml` and `maintenance-log.yaml` (they are files, not dirs).
/// Returns an empty vec if the `.sdlc/knowledge/` directory doesn't exist.
pub fn list(root: &Path) -> Result<Vec<KnowledgeEntry>> {
    let base = root.join(KNOWLEDGE_DIR);
    if !base.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&base)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        // Skip top-level files like catalog.yaml and maintenance-log.yaml
        if entry.file_type()?.is_file() {
            continue;
        }
        let manifest = entry.path().join(MANIFEST_FILE);
        if !manifest.exists() {
            continue;
        }
        match load(root, &name) {
            Ok(e) => entries.push(e),
            Err(_) => continue,
        }
    }
    entries.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(entries)
}

/// List entries whose code starts with the given prefix.
pub fn list_by_code_prefix(root: &Path, prefix: &str) -> Result<Vec<KnowledgeEntry>> {
    Ok(list(root)?
        .into_iter()
        .filter(|e| e.code.starts_with(prefix))
        .collect())
}

/// Update fields on an existing entry. Only non-`None` parameters are applied.
/// `tags_add` and `related_add` are appended (deduped).
#[allow(clippy::too_many_arguments)]
pub fn update(
    root: &Path,
    slug: &str,
    title: Option<&str>,
    code: Option<&str>,
    status: Option<KnowledgeStatus>,
    summary: Option<&str>,
    tags_add: &[String],
    related_add: &[String],
) -> Result<KnowledgeEntry> {
    let mut entry = load(root, slug)?;

    if let Some(t) = title {
        entry.title = t.to_string();
    }
    if let Some(c) = code {
        validate_code(c)?;
        entry.code = c.to_string();
    }
    if let Some(s) = status {
        entry.status = s;
    }
    if let Some(s) = summary {
        entry.summary = Some(s.to_string());
    }
    for tag in tags_add {
        if !entry.tags.contains(tag) {
            entry.tags.push(tag.clone());
        }
    }
    for rel in related_add {
        if !entry.related.contains(rel) {
            entry.related.push(rel.clone());
        }
    }
    entry.updated_at = Utc::now();

    save(root, &entry)?;
    Ok(entry)
}

// ---------------------------------------------------------------------------
// Content management
// ---------------------------------------------------------------------------

/// Append text to the entry's `content.md`, creating a blank line separator.
pub fn append_content(root: &Path, slug: &str, text: &str) -> Result<()> {
    let path = knowledge_content_path(root, slug);
    let existing = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let new_content = if existing.is_empty() {
        text.to_string()
    } else {
        format!("{}\n\n{}", existing.trim_end(), text)
    };
    crate::io::atomic_write(&path, new_content.as_bytes())
}

/// Read the full text of an entry's `content.md`.
pub fn read_content(root: &Path, slug: &str) -> Result<String> {
    let path = knowledge_content_path(root, slug);
    Ok(std::fs::read_to_string(&path).unwrap_or_default())
}

// ---------------------------------------------------------------------------
// Full-text search
// ---------------------------------------------------------------------------

/// Search all entries for `query` (case-insensitive).
///
/// Searches: title, summary, tags, and the full `content.md` body.
/// Title/summary matches rank before content-only matches.
/// Returns `[]` if the knowledge directory is absent — not an error.
pub fn full_text_search(root: &Path, query: &str) -> Result<Vec<SearchResult>> {
    let entries = list(root)?;
    let query_lower = query.to_lowercase();

    let mut metadata_hits: Vec<SearchResult> = Vec::new();
    let mut content_hits: Vec<SearchResult> = Vec::new();

    for entry in entries {
        let mut excerpt: Option<String> = None;
        let mut is_metadata_hit = false;

        // Build metadata corpus
        let metadata_corpus = format!(
            "{}\n{}\n{}",
            entry.title,
            entry.summary.as_deref().unwrap_or(""),
            entry.tags.join(" ")
        );

        if metadata_corpus.to_lowercase().contains(&query_lower) {
            is_metadata_hit = true;
            // Find the first matching line for the excerpt
            for line in metadata_corpus.lines() {
                if line.to_lowercase().contains(&query_lower) {
                    let trimmed = line.trim();
                    excerpt = Some(trimmed.chars().take(120).collect());
                    break;
                }
            }
        }

        // Also search content.md
        let content = read_content(root, &entry.slug)?;
        if !content.is_empty() {
            for line in content.lines() {
                if line.to_lowercase().contains(&query_lower) {
                    if excerpt.is_none() {
                        let trimmed = line.trim();
                        excerpt = Some(trimmed.chars().take(120).collect());
                    }
                    if !is_metadata_hit {
                        // It's a content-only hit — we have an excerpt already
                        break;
                    }
                    break;
                }
            }
        }

        if let Some(exc) = excerpt {
            let result = SearchResult {
                entry,
                excerpt: exc,
            };
            if is_metadata_hit {
                metadata_hits.push(result);
            } else {
                content_hits.push(result);
            }
        }
    }

    let mut results = metadata_hits;
    results.extend(content_hits);
    Ok(results)
}

// ---------------------------------------------------------------------------
// Session wrappers (delegate to workspace.rs)
// ---------------------------------------------------------------------------

/// Log a session for a knowledge entry. Returns the session number assigned.
pub fn log_session(root: &Path, slug: &str, content: &str) -> Result<u32> {
    let dir = knowledge_dir(root, slug);
    if !dir.exists() {
        return Err(SdlcError::KnowledgeNotFound(slug.to_string()));
    }
    workspace::write_session(&dir, content)
}

/// List session metadata for an entry, sorted ascending.
pub fn list_sessions(root: &Path, slug: &str) -> Result<Vec<workspace::SessionMeta>> {
    let dir = knowledge_dir(root, slug);
    workspace::list_sessions(&dir)
}

/// Read the full content of session `n` for an entry.
pub fn read_session(root: &Path, slug: &str, n: u32) -> Result<String> {
    let dir = knowledge_dir(root, slug);
    workspace::read_session(&dir, n)
}

// ---------------------------------------------------------------------------
// Artifact wrappers (delegate to workspace.rs)
// ---------------------------------------------------------------------------

/// Write a named artifact into the entry's directory.
pub fn capture_named_artifact(
    root: &Path,
    slug: &str,
    filename: &str,
    content: &str,
) -> Result<()> {
    let dir = knowledge_dir(root, slug);
    workspace::write_artifact(&dir, filename, content)
}

/// Copy a file from `src` into the entry's directory under `filename`.
pub fn capture_named_artifact_from_file(
    root: &Path,
    slug: &str,
    src: &Path,
    filename: &str,
) -> Result<()> {
    let dir = knowledge_dir(root, slug);
    workspace::write_artifact_from_file(&dir, src, filename)
}

/// List artifacts in the entry's directory, skipping manifest and content files.
pub fn list_named_artifacts(root: &Path, slug: &str) -> Result<Vec<workspace::ArtifactMeta>> {
    let dir = knowledge_dir(root, slug);
    workspace::list_artifacts(&dir, &["manifest.yaml", "content.md"])
}

/// Read a named artifact from the entry's directory.
pub fn read_named_artifact(root: &Path, slug: &str, filename: &str) -> Result<String> {
    let dir = knowledge_dir(root, slug);
    workspace::read_artifact(&dir, filename)
}

// ---------------------------------------------------------------------------
// Catalog management
// ---------------------------------------------------------------------------

/// Load the catalog. Returns an empty catalog if `catalog.yaml` doesn't exist.
pub fn load_catalog(root: &Path) -> Result<Catalog> {
    let path = knowledge_catalog_path(root);
    if !path.exists() {
        return Ok(Catalog {
            classes: Vec::new(),
            updated_at: Utc::now(),
        });
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(serde_yaml::from_str(&raw)?)
}

/// Persist the catalog to disk.
pub fn save_catalog(root: &Path, catalog: &Catalog) -> Result<()> {
    let base = root.join(KNOWLEDGE_DIR);
    if !base.exists() {
        std::fs::create_dir_all(&base)?;
    }
    let path = knowledge_catalog_path(root);
    let raw = serde_yaml::to_string(catalog)?;
    crate::io::atomic_write(&path, raw.as_bytes())
}

/// Add a new top-level class to the catalog.
pub fn add_class(
    root: &Path,
    code: &str,
    name: &str,
    description: Option<&str>,
) -> Result<Catalog> {
    let mut catalog = load_catalog(root)?;
    catalog.classes.push(CatalogClass {
        code: code.to_string(),
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        divisions: Vec::new(),
    });
    catalog.updated_at = Utc::now();
    save_catalog(root, &catalog)?;
    Ok(catalog)
}

/// Add a division to an existing catalog class identified by `class_code`.
pub fn add_division(
    root: &Path,
    class_code: &str,
    code: &str,
    name: &str,
    description: Option<&str>,
) -> Result<Catalog> {
    let mut catalog = load_catalog(root)?;
    let class = catalog
        .classes
        .iter_mut()
        .find(|c| c.code == class_code)
        .ok_or_else(|| SdlcError::KnowledgeNotFound(format!("catalog class {class_code}")))?;
    class.divisions.push(CatalogDivision {
        code: code.to_string(),
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        sections: Vec::new(),
    });
    catalog.updated_at = Utc::now();
    save_catalog(root, &catalog)?;
    Ok(catalog)
}

// ---------------------------------------------------------------------------
// Maintenance log
// ---------------------------------------------------------------------------

/// Load the maintenance log. Returns an empty log if the file doesn't exist.
pub fn load_maintenance_log(root: &Path) -> Result<MaintenanceLog> {
    let path = knowledge_maintenance_log_path(root);
    if !path.exists() {
        return Ok(MaintenanceLog {
            actions: Vec::new(),
        });
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(serde_yaml::from_str(&raw)?)
}

/// Append a maintenance action to the log.
pub fn append_maintenance_action(root: &Path, action: MaintenanceAction) -> Result<()> {
    let mut log = load_maintenance_log(root)?;
    log.actions.push(action);
    let path = knowledge_maintenance_log_path(root);
    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let raw = serde_yaml::to_string(&log)?;
    crate::io::atomic_write(&path, raw.as_bytes())
}

// ---------------------------------------------------------------------------
// Librarian init — harvest, catalog, agent file, cross-ref
// ---------------------------------------------------------------------------

/// Result of harvesting a single workspace entry into the knowledge base.
#[derive(Debug, Clone)]
pub struct HarvestResult {
    pub slug: String,
    /// `true` = new entry created; `false` = appended to existing entry.
    pub created: bool,
    /// e.g. "investigation/auth-bug" or "ponder/api-design"
    pub source: String,
}

/// Summary report returned by `librarian_init`.
#[derive(Debug)]
pub struct LibrarianInitReport {
    pub investigation_results: Vec<HarvestResult>,
    pub ponder_results: Vec<HarvestResult>,
    pub guideline_results: Vec<HarvestResult>,
    pub catalog_created: bool,
    pub catalog_class_count: usize,
    pub agent_file_path: std::path::PathBuf,
    pub cross_ref_count: usize,
}

/// Embedded template for the knowledge-librarian agent file.
const LIBRARIAN_AGENT_TEMPLATE: &str = r###"---
model: claude-sonnet-4-6
description: Knowledge librarian for {PROJECT_NAME} — classifies, cross-references, and maintains the project knowledge base
tools: Bash, Read, Write, Edit, Glob, Grep
---

# Knowledge Librarian: {PROJECT_NAME}

You are the knowledge librarian for **{PROJECT_NAME}**. You curate the project knowledge base at `.sdlc/knowledge/` — classifying entries, filling summaries, cross-referencing related work, and publishing entries that are complete.

## Current Catalog

```yaml
{CATALOG_YAML}
```

## Core Commands

```bash
sdlc knowledge status                              # overview
sdlc knowledge list                                # all entries
sdlc knowledge list --code-prefix 100             # filter by class
sdlc knowledge show <slug>                         # read an entry
sdlc knowledge update <slug> --code 100.20         # reclassify
sdlc knowledge update <slug> --status published    # publish
sdlc knowledge search "<query>"                    # full-text search
```

## Your Protocol

When asked to maintain the knowledge base:
1. `sdlc knowledge list` — identify entries with `code: uncategorized`
2. Classify each based on title, summary, and tags using the catalog above
3. Fill missing summaries (1-2 sentences, key insight only)
4. Find cross-references: entries with overlapping topics → add to `related[]`
5. Publish entries that are complete and accurate

When adding new knowledge from a workspace:
- Set `origin: harvested`, `harvested_from: "investigation/<slug>"` or `"ponder/<slug>"`
- Write durable insights only — decisions, conclusions, patterns. Not raw dialogue.
- Start with `status: draft`; publish when the content is solid
"###;

/// Bootstrap the knowledge base for a project (idempotent, 9-step).
///
/// Safe to re-run at any time — each step is a no-op if already complete.
pub fn librarian_init(root: &Path) -> Result<LibrarianInitReport> {
    // Step 1: Ensure .sdlc/knowledge/ directory exists
    let kdir = root.join(KNOWLEDGE_DIR);
    std::fs::create_dir_all(&kdir)?;

    // Steps 2–4: Harvest completed workspaces
    let investigation_results = harvest_investigations(root)?;
    let ponder_results = harvest_ponders(root)?;
    let guideline_results = harvest_guidelines(root)?;

    // Step 5: Seed catalog if it doesn't exist yet
    let catalog_path = knowledge_catalog_path(root);
    let (catalog_created, catalog) = if !catalog_path.exists() {
        let cat = seed_catalog(root)?;
        (true, cat)
    } else {
        (false, load_catalog(root)?)
    };
    let catalog_class_count = catalog.classes.len();

    // Step 6: Write librarian agent file (always overwrites to pick up catalog changes)
    let agent_file_path = write_librarian_agent_file(root)?;

    // Step 7: Cross-reference pass
    let cross_ref_count = cross_ref_pass(root)?;

    // Step 8: Log maintenance action
    append_maintenance_action(
        root,
        MaintenanceAction {
            timestamp: Utc::now(),
            action_type: "harvest".to_string(),
            slug: None,
            detail: "librarian init".to_string(),
        },
    )?;

    // Step 9: Return report
    Ok(LibrarianInitReport {
        investigation_results,
        ponder_results,
        guideline_results,
        catalog_created,
        catalog_class_count,
        agent_file_path,
        cross_ref_count,
    })
}

/// Write (or overwrite) the knowledge-librarian agent file at
/// `{root}/.claude/agents/knowledge-librarian.md`.
pub fn write_librarian_agent_file(root: &Path) -> Result<std::path::PathBuf> {
    let catalog = load_catalog(root)?;
    let project_name = extract_project_name(root);

    // Serialize catalog classes into compact YAML for the template
    let catalog_yaml = serde_yaml::to_string(&catalog).unwrap_or_default();

    let content = LIBRARIAN_AGENT_TEMPLATE
        .replace("{PROJECT_NAME}", &project_name)
        .replace("{CATALOG_YAML}", catalog_yaml.trim_end());

    let agents_dir = root.join(".claude/agents");
    std::fs::create_dir_all(&agents_dir)?;

    let agent_file = agents_dir.join("knowledge-librarian.md");
    crate::io::atomic_write(&agent_file, content.as_bytes())?;

    Ok(agent_file)
}

/// Harvest completed (non-guideline) investigations into the knowledge base.
fn harvest_investigations(root: &Path) -> Result<Vec<HarvestResult>> {
    let investigations = crate::investigation::list(root)?;
    let mut results = Vec::new();

    for inv in investigations {
        if inv.status != crate::investigation::InvestigationStatus::Complete {
            continue;
        }
        // Guidelines are handled by harvest_guidelines
        if inv.kind == crate::investigation::InvestigationKind::Guideline {
            continue;
        }

        let knowledge_slug = format!("investigation-{}", inv.slug);
        let source = format!("investigation/{}", inv.slug);
        let tags = vec![inv.kind.to_string(), "investigation".to_string()];
        let summary: String = inv
            .context
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(200)
            .collect();

        // Build content: title + context + first session if available
        let mut content = if let Some(ctx) = &inv.context {
            format!("# {}\n\n{}", inv.title, ctx)
        } else {
            format!("# {}", inv.title)
        };

        if let Ok(sessions) = crate::investigation::list_sessions(root, &inv.slug) {
            if let Some(first) = sessions.first() {
                if let Ok(body) = crate::investigation::read_session(root, &inv.slug, first.session)
                {
                    content.push_str("\n\n## Session 1\n\n");
                    content.push_str(&body);
                }
            }
        }

        let created = upsert_knowledge_entry(
            root,
            &knowledge_slug,
            &inv.title,
            &summary,
            &tags,
            OriginKind::Harvested,
            &source,
            &content,
        )?;

        results.push(HarvestResult {
            slug: knowledge_slug,
            created,
            source,
        });
    }

    Ok(results)
}

/// Harvest committed ponder entries into the knowledge base.
fn harvest_ponders(root: &Path) -> Result<Vec<HarvestResult>> {
    let ponders = crate::ponder::PonderEntry::list(root)?;
    let mut results = Vec::new();

    for ponder in ponders {
        if ponder.status != crate::ponder::PonderStatus::Committed {
            continue;
        }

        let knowledge_slug = format!("ponder-{}", ponder.slug);
        let source = format!("ponder/{}", ponder.slug);

        let mut tags = ponder.tags.clone();
        if !tags.iter().any(|t| t == "ponder") {
            tags.push("ponder".to_string());
        }

        let content = read_ponder_content(root, &ponder.slug);

        let created = upsert_knowledge_entry(
            root,
            &knowledge_slug,
            &ponder.title,
            "",
            &tags,
            OriginKind::Harvested,
            &source,
            &content,
        )?;

        results.push(HarvestResult {
            slug: knowledge_slug,
            created,
            source,
        });
    }

    Ok(results)
}

/// Harvest published guidelines into the knowledge base.
fn harvest_guidelines(root: &Path) -> Result<Vec<HarvestResult>> {
    let investigations = crate::investigation::list(root)?;
    let mut results = Vec::new();

    for inv in investigations {
        if inv.kind != crate::investigation::InvestigationKind::Guideline {
            continue;
        }
        let Some(ref publish_path) = inv.publish_path else {
            continue;
        };

        let knowledge_slug = format!("guideline-{}", inv.slug);
        let source = format!("investigation/{}", inv.slug);
        let content = std::fs::read_to_string(publish_path).unwrap_or_default();

        let created = upsert_knowledge_entry(
            root,
            &knowledge_slug,
            &inv.title,
            "",
            &["guideline".to_string()],
            OriginKind::Guideline,
            &source,
            &content,
        )?;

        results.push(HarvestResult {
            slug: knowledge_slug,
            created,
            source,
        });
    }

    Ok(results)
}

/// Seed the catalog from ARCHITECTURE.md H2 headings, or from defaults if absent.
fn seed_catalog(root: &Path) -> Result<Catalog> {
    let arch_path = root.join("ARCHITECTURE.md");
    let headings: Vec<String> = if arch_path.exists() {
        std::fs::read_to_string(&arch_path)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| line.strip_prefix("## ").map(|h| h.trim().to_string()))
            .filter(|h| !h.is_empty())
            .take(7)
            .collect()
    } else {
        Vec::new()
    };

    let codes = ["100", "200", "300", "400", "500", "600", "700"];

    let class_pairs: Vec<(&str, &str)> = if headings.len() >= 3 {
        codes
            .iter()
            .zip(headings.iter())
            .map(|(c, n)| (*c, n.as_str()))
            .collect()
    } else {
        vec![
            ("100", "Architecture & Design"),
            ("200", "Development"),
            ("300", "Process"),
            ("400", "Research"),
            ("500", "Operations"),
        ]
    };

    for (code, name) in &class_pairs {
        add_class(root, code, name, None)?;
    }

    load_catalog(root)
}

/// Link entries that share ≥2 tags via `related[]`. Returns count of new links added.
fn cross_ref_pass(root: &Path) -> Result<usize> {
    let mut entries = list(root)?;
    let n = entries.len();
    let mut link_count = 0;

    for i in 0..n {
        for j in (i + 1)..n {
            let overlap_count = entries[i]
                .tags
                .iter()
                .filter(|t| entries[j].tags.contains(t))
                .count();

            if overlap_count < 2 {
                continue;
            }

            let slug_i = entries[i].slug.clone();
            let slug_j = entries[j].slug.clone();

            let mut new_link = false;
            if !entries[i].related.contains(&slug_j) {
                entries[i].related.push(slug_j.clone());
                new_link = true;
            }
            if !entries[j].related.contains(&slug_i) {
                entries[j].related.push(slug_i.clone());
                new_link = true;
            }

            if new_link {
                entries[i].updated_at = Utc::now();
                entries[j].updated_at = Utc::now();
                save(root, &entries[i])?;
                save(root, &entries[j])?;
                link_count += 1;
            }
        }
    }

    Ok(link_count)
}

/// Upsert a knowledge entry: create if new, append to content.md if existing.
/// Returns `true` if a new entry was created, `false` if an existing one was updated.
#[allow(clippy::too_many_arguments)]
fn upsert_knowledge_entry(
    root: &Path,
    slug: &str,
    title: &str,
    summary: &str,
    tags: &[String],
    origin: OriginKind,
    harvested_from: &str,
    content: &str,
) -> Result<bool> {
    match load(root, slug) {
        Ok(mut entry) => {
            // Entry exists — append a new section to content.md
            if !content.is_empty() {
                let section = format!("---\n\n{}", content);
                append_content(root, slug, &section)?;
            }
            entry.updated_at = Utc::now();
            save(root, &entry)?;
            Ok(false)
        }
        Err(SdlcError::KnowledgeNotFound(_)) => {
            let mut entry = create(root, slug, title, "uncategorized")?;
            if !summary.is_empty() {
                entry.summary = Some(summary.to_string());
            }
            for tag in tags {
                if !entry.tags.contains(tag) {
                    entry.tags.push(tag.clone());
                }
            }
            entry.origin = origin;
            entry.harvested_from = Some(harvested_from.to_string());
            save(root, &entry)?;
            if !content.is_empty() {
                append_content(root, slug, content)?;
            }
            Ok(true)
        }
        Err(e) => Err(e),
    }
}

/// Read decision/plan artifacts from a ponder entry, falling back to first session.
fn read_ponder_content(root: &Path, slug: &str) -> String {
    for artifact_name in &["decisions-final.md", "decisions.md", "plan.md"] {
        if let Ok(content) = crate::ponder::read_artifact(root, slug, artifact_name) {
            if !content.is_empty() {
                return content;
            }
        }
    }
    // Fall back to first session
    if let Ok(sessions) = crate::ponder::list_sessions(root, slug) {
        if let Some(first) = sessions.first() {
            if let Ok(body) = crate::ponder::read_session(root, slug, first.session) {
                return body;
            }
        }
    }
    String::new()
}

/// Extract project name from CLAUDE.md or VISION.md (first H1), falling back to dir name.
fn extract_project_name(root: &Path) -> String {
    for filename in &["CLAUDE.md", "VISION.md"] {
        if let Ok(content) = std::fs::read_to_string(root.join(filename)) {
            for line in content.lines() {
                if let Some(name) = line.strip_prefix("# ") {
                    let name = name.trim();
                    if !name.is_empty() {
                        return name.to_string();
                    }
                }
            }
        }
    }
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn tmp() -> (TempDir, PathBuf) {
        let d = TempDir::new().unwrap();
        let p = d.path().to_path_buf();
        (d, p)
    }

    #[test]
    fn create_returns_entry_with_correct_fields() {
        let (_d, root) = tmp();
        let entry = create(&root, "sse-events", "SSE Event System", "100.20").unwrap();
        assert_eq!(entry.slug, "sse-events");
        assert_eq!(entry.title, "SSE Event System");
        assert_eq!(entry.code, "100.20");
        assert_eq!(entry.status, KnowledgeStatus::Draft);
        assert!(entry.tags.is_empty());
        assert!(entry.summary.is_none());
        // directory and content.md exist
        assert!(knowledge_dir(&root, "sse-events").exists());
        assert!(knowledge_content_path(&root, "sse-events").exists());
    }

    #[test]
    fn duplicate_slug_rejected() {
        let (_d, root) = tmp();
        create(&root, "sse-events", "First", "100").unwrap();
        let err = create(&root, "sse-events", "Second", "200").unwrap_err();
        assert!(matches!(err, SdlcError::KnowledgeExists(_)));
    }

    #[test]
    fn load_roundtrip() {
        let (_d, root) = tmp();
        create(&root, "auth-pattern", "Auth Pattern", "200.10").unwrap();
        let loaded = load(&root, "auth-pattern").unwrap();
        assert_eq!(loaded.slug, "auth-pattern");
        assert_eq!(loaded.code, "200.10");
    }

    #[test]
    fn list_returns_all() {
        let (_d, root) = tmp();
        create(&root, "entry-a", "Entry A", "100").unwrap();
        create(&root, "entry-b", "Entry B", "200").unwrap();
        create(&root, "entry-c", "Entry C", "300").unwrap();
        let entries = list(&root).unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn list_by_code_prefix_filters() {
        let (_d, root) = tmp();
        create(&root, "entry-a", "Entry A", "100").unwrap();
        create(&root, "entry-b", "Entry B", "100.20").unwrap();
        create(&root, "entry-c", "Entry C", "200").unwrap();
        let results = list_by_code_prefix(&root, "100").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn update_modifies_fields() {
        let (_d, root) = tmp();
        create(&root, "my-entry", "Original", "100").unwrap();
        let original = load(&root, "my-entry").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let updated = update(
            &root,
            "my-entry",
            Some("Updated Title"),
            Some("200.10"),
            None,
            None,
            &[],
            &[],
        )
        .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.code, "200.10");
        assert!(updated.updated_at > original.updated_at);
    }

    #[test]
    fn reclassify_does_not_rename_dir() {
        let (_d, root) = tmp();
        create(&root, "my-entry", "Entry", "100").unwrap();
        let dir_before = knowledge_dir(&root, "my-entry");
        assert!(dir_before.exists());

        update(
            &root,
            "my-entry",
            None,
            Some("200.50"),
            None,
            None,
            &[],
            &[],
        )
        .unwrap();

        // Directory name is unchanged
        assert!(dir_before.exists());
        // New code is persisted in manifest
        let loaded = load(&root, "my-entry").unwrap();
        assert_eq!(loaded.code, "200.50");
    }

    #[test]
    fn validate_code_accepts_valid() {
        for code in ["100", "100.20", "100.20.3", "uncategorized"] {
            validate_code(code).unwrap_or_else(|_| panic!("expected valid: {code}"));
        }
    }

    #[test]
    fn validate_code_rejects_invalid() {
        for code in ["abc", "1000", "100.200", "100.20.30", "10", "1"] {
            assert!(validate_code(code).is_err(), "expected invalid: {code}");
        }
    }

    #[test]
    fn append_and_read_content() {
        let (_d, root) = tmp();
        create(&root, "my-entry", "Entry", "100").unwrap();

        append_content(&root, "my-entry", "First paragraph.").unwrap();
        let after_first = read_content(&root, "my-entry").unwrap();
        assert_eq!(after_first, "First paragraph.");

        append_content(&root, "my-entry", "Second paragraph.").unwrap();
        let after_second = read_content(&root, "my-entry").unwrap();
        assert!(after_second.contains("First paragraph."));
        assert!(after_second.contains("Second paragraph."));
    }

    #[test]
    fn full_text_search_in_title() {
        let (_d, root) = tmp();
        create(&root, "sse-events", "SSE Event System", "100").unwrap();
        create(&root, "auth-jwt", "JWT Authentication", "200").unwrap();

        let results = full_text_search(&root, "SSE").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.slug, "sse-events");
        assert!(!results[0].excerpt.is_empty());
    }

    #[test]
    fn full_text_search_in_content() {
        let (_d, root) = tmp();
        create(&root, "spawn-pattern", "Agent Spawning", "100").unwrap();
        append_content(
            &root,
            "spawn-pattern",
            "Use spawn_agent_run to start tasks.",
        )
        .unwrap();

        let results = full_text_search(&root, "spawn_agent_run").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.slug, "spawn-pattern");
        assert!(results[0].excerpt.contains("spawn_agent_run"));
    }

    #[test]
    fn full_text_search_empty_base() {
        let (_d, root) = tmp();
        let results = full_text_search(&root, "anything").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn log_session_increments_count() {
        let (_d, root) = tmp();
        create(&root, "my-entry", "Entry", "100").unwrap();

        let session_content =
            "---\nsession: 1\ntimestamp: 2026-03-01T10:00:00Z\n---\n\nSession notes.";
        let n1 = log_session(&root, "my-entry", session_content).unwrap();
        assert_eq!(n1, 1);

        let n2 = log_session(&root, "my-entry", session_content).unwrap();
        assert_eq!(n2, 2);

        let sessions = list_sessions(&root, "my-entry").unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn catalog_add_class_and_load() {
        let (_d, root) = tmp();
        let catalog =
            add_class(&root, "100", "Architecture", Some("System design patterns")).unwrap();
        assert_eq!(catalog.classes.len(), 1);
        assert_eq!(catalog.classes[0].code, "100");

        let loaded = load_catalog(&root).unwrap();
        assert_eq!(loaded.classes.len(), 1);
        assert_eq!(loaded.classes[0].name, "Architecture");
    }

    #[test]
    fn catalog_load_missing_returns_empty() {
        let (_d, root) = tmp();
        let catalog = load_catalog(&root).unwrap();
        assert!(catalog.classes.is_empty());
    }

    #[test]
    fn maintenance_log_append_and_load() {
        let (_d, root) = tmp();
        // Need .sdlc/knowledge dir to exist first
        std::fs::create_dir_all(root.join(".sdlc/knowledge")).unwrap();

        let action = MaintenanceAction {
            timestamp: Utc::now(),
            action_type: "url_check".to_string(),
            slug: Some("my-entry".to_string()),
            detail: "404 detected".to_string(),
        };
        append_maintenance_action(&root, action).unwrap();

        let log = load_maintenance_log(&root).unwrap();
        assert_eq!(log.actions.len(), 1);
        assert_eq!(log.actions[0].action_type, "url_check");
    }

    #[test]
    fn maintenance_log_missing_returns_empty() {
        let (_d, root) = tmp();
        let log = load_maintenance_log(&root).unwrap();
        assert!(log.actions.is_empty());
    }

    // -----------------------------------------------------------------------
    // librarian_init tests
    // -----------------------------------------------------------------------

    #[test]
    fn librarian_init_on_empty_project() {
        let (_d, root) = tmp();
        let report = librarian_init(&root).unwrap();
        assert_eq!(report.investigation_results.len(), 0);
        assert_eq!(report.ponder_results.len(), 0);
        assert_eq!(report.guideline_results.len(), 0);
        assert!(report.catalog_created);
        assert!(report.catalog_class_count > 0);
        assert!(report.agent_file_path.exists());
    }

    #[test]
    fn librarian_init_creates_agent_file() {
        let (_d, root) = tmp();
        librarian_init(&root).unwrap();
        let agent_file = root.join(".claude/agents/knowledge-librarian.md");
        assert!(agent_file.exists());
        let content = std::fs::read_to_string(&agent_file).unwrap();
        assert!(content.contains("Knowledge Librarian"));
    }

    #[test]
    fn librarian_init_idempotent() {
        let (_d, root) = tmp();
        // Create a completed investigation
        std::fs::create_dir_all(root.join(".sdlc/investigations")).unwrap();
        let mut inv = crate::investigation::create(
            &root,
            "auth-bug",
            "Auth Bug",
            crate::investigation::InvestigationKind::RootCause,
            Some("Root cause of auth bug".to_string()),
        )
        .unwrap();
        inv.status = crate::investigation::InvestigationStatus::Complete;
        crate::investigation::save(&root, &inv).unwrap();

        let report1 = librarian_init(&root).unwrap();
        assert_eq!(report1.investigation_results.len(), 1);
        assert!(report1.investigation_results[0].created);

        let report2 = librarian_init(&root).unwrap();
        assert_eq!(report2.investigation_results.len(), 1);
        // On second run, entry already exists — not created again
        assert!(!report2.investigation_results[0].created);

        // No duplicate entries
        let entries = list(&root).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn harvest_investigation_creates_entry() {
        let (_d, root) = tmp();
        std::fs::create_dir_all(root.join(".sdlc/investigations")).unwrap();
        let mut inv = crate::investigation::create(
            &root,
            "db-slowness",
            "DB Slowness",
            crate::investigation::InvestigationKind::RootCause,
            Some("Slow queries in prod".to_string()),
        )
        .unwrap();
        inv.status = crate::investigation::InvestigationStatus::Complete;
        crate::investigation::save(&root, &inv).unwrap();

        let results = harvest_investigations(&root).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "investigation-db-slowness");
        assert!(results[0].created);
        assert_eq!(results[0].source, "investigation/db-slowness");

        let entry = load(&root, "investigation-db-slowness").unwrap();
        assert_eq!(entry.origin, OriginKind::Harvested);
        assert_eq!(
            entry.harvested_from.as_deref(),
            Some("investigation/db-slowness")
        );
    }

    #[test]
    fn harvest_investigation_in_progress_skipped() {
        let (_d, root) = tmp();
        std::fs::create_dir_all(root.join(".sdlc/investigations")).unwrap();
        // Default status is InProgress
        crate::investigation::create(
            &root,
            "open-bug",
            "Open Bug",
            crate::investigation::InvestigationKind::RootCause,
            None,
        )
        .unwrap();

        let results = harvest_investigations(&root).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn harvest_ponder_committed_creates_entry() {
        let (_d, root) = tmp();
        std::fs::create_dir_all(root.join(".sdlc/roadmap")).unwrap();
        let mut ponder =
            crate::ponder::PonderEntry::create(&root, "api-design", "API Design").unwrap();
        ponder.update_status(crate::ponder::PonderStatus::Committed);
        ponder.save(&root).unwrap();

        let results = harvest_ponders(&root).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "ponder-api-design");
        assert!(results[0].created);
        assert_eq!(results[0].source, "ponder/api-design");

        let entry = load(&root, "ponder-api-design").unwrap();
        assert_eq!(entry.origin, OriginKind::Harvested);
        assert!(entry.tags.contains(&"ponder".to_string()));
    }

    #[test]
    fn harvest_ponder_exploring_skipped() {
        let (_d, root) = tmp();
        std::fs::create_dir_all(root.join(".sdlc/roadmap")).unwrap();
        // Default status is Exploring
        crate::ponder::PonderEntry::create(&root, "early-idea", "Early Idea").unwrap();

        let results = harvest_ponders(&root).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn harvest_guideline_creates_entry() {
        let (_d, root) = tmp();
        std::fs::create_dir_all(root.join(".sdlc/investigations")).unwrap();

        // Write a guideline file
        let guideline_path = root.join("error-handling.md");
        std::fs::write(
            &guideline_path,
            "# Error Handling Guideline\n\nAlways use ? operator.",
        )
        .unwrap();

        let mut inv = crate::investigation::create(
            &root,
            "error-handling",
            "Error Handling",
            crate::investigation::InvestigationKind::Guideline,
            None,
        )
        .unwrap();
        inv.publish_path = Some(guideline_path.to_string_lossy().to_string());
        crate::investigation::save(&root, &inv).unwrap();

        let results = harvest_guidelines(&root).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "guideline-error-handling");
        assert!(results[0].created);

        let entry = load(&root, "guideline-error-handling").unwrap();
        assert_eq!(entry.origin, OriginKind::Guideline);
        assert!(entry.tags.contains(&"guideline".to_string()));
    }

    #[test]
    fn seed_catalog_uses_defaults() {
        let (_d, root) = tmp();
        std::fs::create_dir_all(root.join(".sdlc/knowledge")).unwrap();
        // No ARCHITECTURE.md → should use defaults
        let catalog = seed_catalog(&root).unwrap();
        assert_eq!(catalog.classes.len(), 5);
        assert_eq!(catalog.classes[0].code, "100");
        assert_eq!(catalog.classes[0].name, "Architecture & Design");
        assert_eq!(catalog.classes[4].code, "500");
    }

    #[test]
    fn seed_catalog_uses_architecture_headings() {
        let (_d, root) = tmp();
        std::fs::create_dir_all(root.join(".sdlc/knowledge")).unwrap();
        std::fs::write(
            root.join("ARCHITECTURE.md"),
            "# Architecture\n\n## Core Systems\n\n## Data Layer\n\n## API Layer\n",
        )
        .unwrap();

        let catalog = seed_catalog(&root).unwrap();
        assert_eq!(catalog.classes.len(), 3);
        assert_eq!(catalog.classes[0].name, "Core Systems");
        assert_eq!(catalog.classes[1].name, "Data Layer");
    }

    #[test]
    fn cross_ref_pass_links_entries() {
        let (_d, root) = tmp();
        // Create two entries with 2 shared tags
        let mut e1 = create(&root, "entry-a", "Entry A", "100").unwrap();
        e1.tags = vec![
            "rust".to_string(),
            "async".to_string(),
            "server".to_string(),
        ];
        save(&root, &e1).unwrap();

        let mut e2 = create(&root, "entry-b", "Entry B", "200").unwrap();
        e2.tags = vec!["rust".to_string(), "async".to_string()];
        save(&root, &e2).unwrap();

        let count = cross_ref_pass(&root).unwrap();
        assert_eq!(count, 1);

        let a = load(&root, "entry-a").unwrap();
        let b = load(&root, "entry-b").unwrap();
        assert!(a.related.contains(&"entry-b".to_string()));
        assert!(b.related.contains(&"entry-a".to_string()));
    }

    #[test]
    fn cross_ref_pass_no_duplicate_links() {
        let (_d, root) = tmp();
        let mut e1 = create(&root, "entry-a", "Entry A", "100").unwrap();
        e1.tags = vec!["rust".to_string(), "async".to_string()];
        save(&root, &e1).unwrap();

        let mut e2 = create(&root, "entry-b", "Entry B", "200").unwrap();
        e2.tags = vec!["rust".to_string(), "async".to_string()];
        save(&root, &e2).unwrap();

        cross_ref_pass(&root).unwrap();
        cross_ref_pass(&root).unwrap(); // run twice

        let a = load(&root, "entry-a").unwrap();
        // Should have exactly one reference to entry-b, not two
        let b_count = a.related.iter().filter(|r| *r == "entry-b").count();
        assert_eq!(b_count, 1);
    }
}
