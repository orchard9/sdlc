//! Investigation workspace — root-cause, evolve, and guideline sessions.
//!
//! All three investigation types share a common `InvestigationEntry` manifest
//! (discriminated by `kind`) and use the `workspace` module for session and
//! artifact I/O. Type-specific fields are optional and omitted from YAML when `None`.

use crate::error::{Result, SdlcError};
use crate::{paths, workspace};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

// ---------------------------------------------------------------------------
// InvestigationKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationKind {
    RootCause,
    Evolve,
    Guideline,
}

impl InvestigationKind {
    /// The phase name an investigation starts in.
    pub fn initial_phase(self) -> &'static str {
        match self {
            InvestigationKind::RootCause => "triage",
            InvestigationKind::Evolve => "survey",
            InvestigationKind::Guideline => "problem",
        }
    }
}

impl fmt::Display for InvestigationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            InvestigationKind::RootCause => "root_cause",
            InvestigationKind::Evolve => "evolve",
            InvestigationKind::Guideline => "guideline",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for InvestigationKind {
    type Err = SdlcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "root_cause" | "root-cause" => Ok(InvestigationKind::RootCause),
            "evolve" => Ok(InvestigationKind::Evolve),
            "guideline" => Ok(InvestigationKind::Guideline),
            other => Err(SdlcError::InvalidInvestigationKind(other.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// InvestigationStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationStatus {
    InProgress,
    Complete,
    Parked,
}

impl fmt::Display for InvestigationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            InvestigationStatus::InProgress => "in_progress",
            InvestigationStatus::Complete => "complete",
            InvestigationStatus::Parked => "parked",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for InvestigationStatus {
    type Err = SdlcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "in_progress" => Ok(InvestigationStatus::InProgress),
            "complete" => Ok(InvestigationStatus::Complete),
            "parked" => Ok(InvestigationStatus::Parked),
            other => Err(SdlcError::InvalidInvestigationStatus(other.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// Kind-specific nested types
// ---------------------------------------------------------------------------

/// Maturity scores populated during the Evolve Analyze phase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LensScores {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pit_of_success: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coupling: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub growth_readiness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_documenting: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_modes: Option<String>,
}

/// Evidence category counts from the Guidelines Evidence phase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceCounts {
    #[serde(default)]
    pub anti_patterns: u32,
    #[serde(default)]
    pub good_examples: u32,
    #[serde(default)]
    pub prior_art: u32,
    #[serde(default)]
    pub adjacent: u32,
}

// ---------------------------------------------------------------------------
// InvestigationEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationEntry {
    pub slug: String,
    pub title: String,
    pub kind: InvestigationKind,
    /// Phase name — kind-specific (e.g. "triage", "investigate", "synthesize", "output", "done").
    pub phase: String,
    pub status: InvestigationStatus,
    /// Initial problem description or investigation context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default)]
    pub sessions: u32,
    /// Orientation from the most recent session (mirrored for fast UI access).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<workspace::Orientation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // -- Root-cause specific --------------------------------------------------
    /// Confidence score (0–100) set after the Synthesize phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u32>,
    /// "task" or "guideline" — set at the Output phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_type: Option<String>,
    /// Feature slug or guideline path produced at output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_ref: Option<String>,

    // -- Evolve specific ------------------------------------------------------
    /// What system/area is being analyzed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Maturity ratings from the five lenses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lens_scores: Option<LensScores>,
    /// Feature slugs or guideline paths created from selected paths.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_refs: Vec<String>,

    // -- Guideline specific ---------------------------------------------------
    /// Where this guideline applies (e.g. "Go API services").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guideline_scope: Option<String>,
    /// Short description of the recurring problem this guideline prevents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem_statement: Option<String>,
    /// Evidence category counts from the Evidence phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_counts: Option<EvidenceCounts>,
    /// Number of rules extracted so far.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principles_count: Option<u32>,
    /// Final publish path (set during Publish phase).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish_path: Option<String>,
}

impl InvestigationEntry {
    pub fn new(
        slug: impl Into<String>,
        title: impl Into<String>,
        kind: InvestigationKind,
        context: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            slug: slug.into(),
            title: title.into(),
            kind,
            phase: kind.initial_phase().to_string(),
            status: InvestigationStatus::InProgress,
            context,
            sessions: 0,
            orientation: None,
            created_at: now,
            updated_at: now,
            confidence: None,
            output_type: None,
            output_ref: None,
            scope: None,
            lens_scores: None,
            output_refs: Vec::new(),
            guideline_scope: None,
            problem_statement: None,
            evidence_counts: None,
            principles_count: None,
            publish_path: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn ensure_investigation_exists(root: &Path, slug: &str) -> Result<()> {
    paths::validate_slug(slug)?;
    if !paths::investigation_manifest(root, slug).exists() {
        return Err(SdlcError::InvestigationNotFound(slug.to_string()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// CRUD
// ---------------------------------------------------------------------------

/// Create a new investigation. Errors if the slug already exists.
pub fn create(
    root: &Path,
    slug: impl Into<String>,
    title: impl Into<String>,
    kind: InvestigationKind,
    context: Option<String>,
) -> Result<InvestigationEntry> {
    let slug = slug.into();
    paths::validate_slug(&slug)?;
    let dir = paths::investigation_dir(root, &slug);
    if dir.exists() {
        return Err(SdlcError::InvestigationExists(slug));
    }
    std::fs::create_dir_all(&dir)?;
    let entry = InvestigationEntry::new(slug, title, kind, context);
    save(root, &entry)?;
    Ok(entry)
}

/// Load an investigation manifest by slug.
pub fn load(root: &Path, slug: &str) -> Result<InvestigationEntry> {
    paths::validate_slug(slug)?;
    let manifest = paths::investigation_manifest(root, slug);
    if !manifest.exists() {
        return Err(SdlcError::InvestigationNotFound(slug.to_string()));
    }
    let data = std::fs::read_to_string(&manifest)?;
    let entry: InvestigationEntry = serde_yaml::from_str(&data)?;
    Ok(entry)
}

/// Persist an investigation manifest.
pub fn save(root: &Path, entry: &InvestigationEntry) -> Result<()> {
    let manifest = paths::investigation_manifest(root, &entry.slug);
    let data = serde_yaml::to_string(entry)?;
    crate::io::atomic_write(&manifest, data.as_bytes())
}

/// List all investigations sorted by creation time.
pub fn list(root: &Path) -> Result<Vec<InvestigationEntry>> {
    let inv_dir = root.join(paths::INVESTIGATIONS_DIR);
    if !inv_dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&inv_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let slug = entry.file_name().to_string_lossy().into_owned();
            match load(root, &slug) {
                Ok(e) => entries.push(e),
                Err(SdlcError::InvestigationNotFound(_)) => {}
                Err(e) => return Err(e),
            }
        }
    }
    entries.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(entries)
}

/// List investigations filtered by kind.
pub fn list_by_kind(root: &Path, kind: InvestigationKind) -> Result<Vec<InvestigationEntry>> {
    Ok(list(root)?.into_iter().filter(|e| e.kind == kind).collect())
}

// ---------------------------------------------------------------------------
// Artifact wrappers
// ---------------------------------------------------------------------------

/// Write content to a named file in the investigation directory.
pub fn capture_content(root: &Path, slug: &str, filename: &str, content: &str) -> Result<()> {
    ensure_investigation_exists(root, slug)?;
    workspace::write_artifact(&paths::investigation_dir(root, slug), filename, content)
}

/// List artifact files (excludes manifest.yaml).
pub fn list_artifacts(root: &Path, slug: &str) -> Result<Vec<workspace::ArtifactMeta>> {
    ensure_investigation_exists(root, slug)?;
    workspace::list_artifacts(&paths::investigation_dir(root, slug), &["manifest.yaml"])
}

/// Read the content of an artifact file.
pub fn read_artifact(root: &Path, slug: &str, filename: &str) -> Result<String> {
    ensure_investigation_exists(root, slug)?;
    workspace::read_artifact(&paths::investigation_dir(root, slug), filename)
}

// ---------------------------------------------------------------------------
// Session wrappers
// ---------------------------------------------------------------------------

/// Write a session file and update the manifest (session count + orientation mirror).
///
/// Returns the session number assigned.
pub fn log_session(root: &Path, slug: &str, content: &str) -> Result<u32> {
    ensure_investigation_exists(root, slug)?;
    let n = workspace::write_session(&paths::investigation_dir(root, slug), content)?;

    let mut entry = load(root, slug)?;
    entry.sessions += 1;
    entry.updated_at = Utc::now();
    if let Some(meta) = workspace::parse_session_meta(content) {
        if let Some(orientation) = meta.orientation {
            entry.orientation = Some(orientation);
        }
    }
    save(root, &entry)?;
    Ok(n)
}

/// List session metadata sorted by session number.
pub fn list_sessions(root: &Path, slug: &str) -> Result<Vec<workspace::SessionMeta>> {
    ensure_investigation_exists(root, slug)?;
    workspace::list_sessions(&paths::investigation_dir(root, slug))
}

/// Read the full content of session `n`.
pub fn read_session(root: &Path, slug: &str, n: u32) -> Result<String> {
    ensure_investigation_exists(root, slug)?;
    workspace::read_session(&paths::investigation_dir(root, slug), n)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(paths::INVESTIGATIONS_DIR)).unwrap();
        let p = dir.path().to_path_buf();
        (dir, p)
    }

    #[test]
    fn create_and_load() {
        let (_dir, root) = setup();
        let entry = create(
            &root,
            "auth-bug",
            "Auth null pointer",
            InvestigationKind::RootCause,
            Some("NullPointerException in AuthService".to_string()),
        )
        .unwrap();
        assert_eq!(entry.kind, InvestigationKind::RootCause);
        assert_eq!(entry.phase, "triage");
        assert_eq!(entry.status, InvestigationStatus::InProgress);

        let loaded = load(&root, "auth-bug").unwrap();
        assert_eq!(loaded.title, "Auth null pointer");
        assert_eq!(
            loaded.context.as_deref(),
            Some("NullPointerException in AuthService")
        );
    }

    #[test]
    fn duplicate_rejected() {
        let (_dir, root) = setup();
        create(&root, "bug", "Bug", InvestigationKind::RootCause, None).unwrap();
        assert!(matches!(
            create(
                &root,
                "bug",
                "Bug Again",
                InvestigationKind::RootCause,
                None
            ),
            Err(SdlcError::InvestigationExists(_))
        ));
    }

    #[test]
    fn list_all_and_by_kind() {
        let (_dir, root) = setup();
        create(&root, "bug-a", "Bug A", InvestigationKind::RootCause, None).unwrap();
        create(
            &root,
            "refactor-b",
            "Refactor B",
            InvestigationKind::Evolve,
            None,
        )
        .unwrap();
        create(
            &root,
            "guide-c",
            "Guide C",
            InvestigationKind::Guideline,
            None,
        )
        .unwrap();

        let all = list(&root).unwrap();
        assert_eq!(all.len(), 3);

        let rc = list_by_kind(&root, InvestigationKind::RootCause).unwrap();
        assert_eq!(rc.len(), 1);
        assert_eq!(rc[0].slug, "bug-a");

        let ev = list_by_kind(&root, InvestigationKind::Evolve).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].slug, "refactor-b");
    }

    #[test]
    fn initial_phases() {
        let (_dir, root) = setup();
        let rc = create(&root, "rc", "RC", InvestigationKind::RootCause, None).unwrap();
        assert_eq!(rc.phase, "triage");

        let ev = create(&root, "ev", "EV", InvestigationKind::Evolve, None).unwrap();
        assert_eq!(ev.phase, "survey");

        let gl = create(&root, "gl", "GL", InvestigationKind::Guideline, None).unwrap();
        assert_eq!(gl.phase, "problem");
    }

    #[test]
    fn session_log_updates_manifest() {
        let (_dir, root) = setup();
        create(&root, "inv", "Inv", InvestigationKind::RootCause, None).unwrap();

        let content = "---\nsession: 1\ntimestamp: 2026-02-27T10:00:00Z\norientation:\n  current: \"triaging\"\n  next: \"investigate code paths\"\n  commit: \"when root cause identified\"\n---\n\nSession body.";
        let n = log_session(&root, "inv", content).unwrap();
        assert_eq!(n, 1);

        let entry = load(&root, "inv").unwrap();
        assert_eq!(entry.sessions, 1);
        let o = entry.orientation.unwrap();
        assert_eq!(o.current, "triaging");
    }

    #[test]
    fn artifact_capture_and_list() {
        let (_dir, root) = setup();
        create(&root, "inv", "Inv", InvestigationKind::RootCause, None).unwrap();
        capture_content(&root, "inv", "triage.md", "## Triage\n\nThe bug is...").unwrap();

        let artifacts = list_artifacts(&root, "inv").unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].filename, "triage.md");
    }

    #[test]
    fn kind_roundtrip() {
        assert_eq!(
            "root_cause".parse::<InvestigationKind>().unwrap(),
            InvestigationKind::RootCause
        );
        assert_eq!(
            "root-cause".parse::<InvestigationKind>().unwrap(),
            InvestigationKind::RootCause
        );
        assert_eq!(
            "evolve".parse::<InvestigationKind>().unwrap(),
            InvestigationKind::Evolve
        );
        assert_eq!(
            "guideline".parse::<InvestigationKind>().unwrap(),
            InvestigationKind::Guideline
        );
        assert!(matches!(
            "unknown".parse::<InvestigationKind>(),
            Err(SdlcError::InvalidInvestigationKind(_))
        ));
    }

    #[test]
    fn save_and_reload_type_specific_fields() {
        let (_dir, root) = setup();
        let mut entry = create(
            &root,
            "lens-test",
            "Lens Test",
            InvestigationKind::Evolve,
            None,
        )
        .unwrap();
        entry.scope = Some("crates/sdlc-server/".to_string());
        entry.lens_scores = Some(LensScores {
            pit_of_success: Some("medium".to_string()),
            coupling: Some("low".to_string()),
            ..Default::default()
        });
        save(&root, &entry).unwrap();

        let loaded = load(&root, "lens-test").unwrap();
        assert_eq!(loaded.scope.as_deref(), Some("crates/sdlc-server/"));
        let scores = loaded.lens_scores.unwrap();
        assert_eq!(scores.pit_of_success.as_deref(), Some("medium"));
        assert_eq!(scores.coupling.as_deref(), Some("low"));
        assert!(scores.growth_readiness.is_none());
    }
}
