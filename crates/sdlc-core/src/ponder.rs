use crate::error::{Result, SdlcError};
use crate::{paths, workspace};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Re-exports from workspace (backward-compatible type aliases)
// ---------------------------------------------------------------------------

/// Orientation compass written at the end of each session.
/// Re-exported from `workspace` — callers that use field access need not change.
pub use workspace::Orientation as PonderOrientation;

/// Session file metadata. Re-exported from `workspace`.
pub use workspace::SessionMeta;

/// Artifact file metadata. Re-exported from `workspace`.
pub use workspace::ArtifactMeta as PonderArtifactMeta;

// ---------------------------------------------------------------------------
// PonderStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PonderStatus {
    Exploring,
    Converging,
    Committed,
    Parked,
}

impl fmt::Display for PonderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PonderStatus::Exploring => "exploring",
            PonderStatus::Converging => "converging",
            PonderStatus::Committed => "committed",
            PonderStatus::Parked => "parked",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for PonderStatus {
    type Err = SdlcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "exploring" => Ok(PonderStatus::Exploring),
            "converging" => Ok(PonderStatus::Converging),
            "committed" => Ok(PonderStatus::Committed),
            "parked" => Ok(PonderStatus::Parked),
            other => Err(SdlcError::InvalidPonderStatus(other.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// PonderEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PonderEntry {
    pub slug: String,
    pub title: String,
    pub status: PonderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub committed_to: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sessions: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<workspace::Orientation>,
}

impl PonderEntry {
    pub fn new(slug: impl Into<String>, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            slug: slug.into(),
            title: title.into(),
            status: PonderStatus::Exploring,
            created_at: now,
            updated_at: now,
            committed_at: None,
            committed_to: Vec::new(),
            tags: Vec::new(),
            sessions: 0,
            orientation: None,
        }
    }

    // -----------------------------------------------------------------------
    // Persistence
    // -----------------------------------------------------------------------

    pub fn create(root: &Path, slug: impl Into<String>, title: impl Into<String>) -> Result<Self> {
        let slug = slug.into();
        paths::validate_slug(&slug)?;

        let dir = paths::ponder_dir(root, &slug);
        if dir.exists() {
            return Err(SdlcError::PonderExists(slug));
        }

        let entry = Self::new(slug, title);
        entry.save(root)?;
        Ok(entry)
    }

    pub fn load(root: &Path, slug: &str) -> Result<Self> {
        paths::validate_slug(slug)?;
        let manifest = paths::ponder_manifest(root, slug);
        if !manifest.exists() {
            return Err(SdlcError::PonderNotFound(slug.to_string()));
        }
        let data = std::fs::read_to_string(&manifest)?;
        let entry: PonderEntry = serde_yaml::from_str(&data)?;
        Ok(entry)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let dir = paths::ponder_dir(root, &self.slug);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        let manifest = paths::ponder_manifest(root, &self.slug);
        let data = serde_yaml::to_string(self)?;
        crate::io::atomic_write(&manifest, data.as_bytes())
    }

    pub fn list(root: &Path) -> Result<Vec<Self>> {
        let roadmap_dir = root.join(paths::ROADMAP_DIR);
        if !roadmap_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&roadmap_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let slug = entry.file_name().to_string_lossy().into_owned();
                match Self::load(root, &slug) {
                    Ok(e) => entries.push(e),
                    Err(SdlcError::PonderNotFound(_)) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        entries.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(entries)
    }

    // -----------------------------------------------------------------------
    // Mutations
    // -----------------------------------------------------------------------

    pub fn update_status(&mut self, status: PonderStatus) {
        self.status = status;
        if status == PonderStatus::Committed && self.committed_at.is_none() {
            self.committed_at = Some(Utc::now());
        }
        self.updated_at = Utc::now();
    }

    pub fn update_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = Utc::now();
    }

    /// Add a tag. Returns `true` if the tag was new, `false` if already present.
    pub fn add_tag(&mut self, tag: impl Into<String>) -> bool {
        let tag = tag.into();
        if self.tags.contains(&tag) {
            return false;
        }
        self.tags.push(tag);
        self.updated_at = Utc::now();
        true
    }

    /// Replace the full tag list, deduplicating entries.
    pub fn set_tags(&mut self, tags: Vec<String>) {
        let mut seen = std::collections::HashSet::new();
        self.tags = tags
            .into_iter()
            .filter(|t| seen.insert(t.clone()))
            .collect();
        self.updated_at = Utc::now();
    }

    pub fn increment_session(&mut self) {
        self.sessions += 1;
        self.updated_at = Utc::now();
    }
}

// ---------------------------------------------------------------------------
// Team types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PonderTeamMember {
    pub name: String,
    pub role: String,
    pub context: String,
    pub agent: String,
    pub recruited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PonderTeam {
    #[serde(default)]
    pub partners: Vec<PonderTeamMember>,
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

fn ensure_ponder_exists(root: &Path, slug: &str) -> Result<()> {
    paths::validate_slug(slug)?;
    if !paths::ponder_manifest(root, slug).exists() {
        return Err(SdlcError::PonderNotFound(slug.to_string()));
    }
    Ok(())
}

fn ponder_dir(root: &Path, slug: &str) -> PathBuf {
    paths::ponder_dir(root, slug)
}

// ---------------------------------------------------------------------------
// Scrapbook functions (delegate to workspace)
// ---------------------------------------------------------------------------

/// Write content to a file in the ponder directory.
pub fn capture_content(root: &Path, slug: &str, filename: &str, content: &str) -> Result<()> {
    ensure_ponder_exists(root, slug)?;
    workspace::write_artifact(&ponder_dir(root, slug), filename, content)
}

/// Copy a file into the ponder directory.
pub fn capture_file(root: &Path, slug: &str, src_path: &Path, filename: &str) -> Result<()> {
    ensure_ponder_exists(root, slug)?;
    workspace::write_artifact_from_file(&ponder_dir(root, slug), src_path, filename)
}

/// List non-manifest/team files in the ponder directory.
pub fn list_artifacts(root: &Path, slug: &str) -> Result<Vec<workspace::ArtifactMeta>> {
    ensure_ponder_exists(root, slug)?;
    workspace::list_artifacts(&ponder_dir(root, slug), &["manifest.yaml", "team.yaml"])
}

/// Read the content of an artifact file.
pub fn read_artifact(root: &Path, slug: &str, filename: &str) -> Result<String> {
    ensure_ponder_exists(root, slug)?;
    workspace::read_artifact(&ponder_dir(root, slug), filename)
}

// ---------------------------------------------------------------------------
// Team functions
// ---------------------------------------------------------------------------

pub fn load_team(root: &Path, slug: &str) -> Result<PonderTeam> {
    ensure_ponder_exists(root, slug)?;
    let path = paths::ponder_team_path(root, slug);
    if !path.exists() {
        return Ok(PonderTeam {
            partners: Vec::new(),
        });
    }
    let data = std::fs::read_to_string(&path)?;
    let team: PonderTeam = serde_yaml::from_str(&data)?;
    Ok(team)
}

pub fn save_team(root: &Path, slug: &str, team: &PonderTeam) -> Result<()> {
    ensure_ponder_exists(root, slug)?;
    let path = paths::ponder_team_path(root, slug);
    let data = serde_yaml::to_string(team)?;
    crate::io::atomic_write(&path, data.as_bytes())
}

pub fn add_team_member(root: &Path, slug: &str, member: PonderTeamMember) -> Result<PonderTeam> {
    let mut team = load_team(root, slug)?;
    if team.partners.iter().any(|p| p.name == member.name) {
        return Err(SdlcError::DuplicateTeamMember(member.name));
    }
    team.partners.push(member);
    save_team(root, slug, &team)?;
    Ok(team)
}

// ---------------------------------------------------------------------------
// Session functions (delegate to workspace, update manifest)
// ---------------------------------------------------------------------------

/// Write a session file and update the entry's session counter and orientation.
///
/// Returns the session number that was written.
pub fn log_session(root: &Path, slug: &str, content: &str) -> Result<u32> {
    ensure_ponder_exists(root, slug)?;
    let n = workspace::write_session(&ponder_dir(root, slug), content)?;

    // Mirror orientation onto manifest and bump session counter
    let mut entry = PonderEntry::load(root, slug)?;
    entry.increment_session();
    if let Some(meta) = workspace::parse_session_meta(content) {
        if let Some(orientation) = meta.orientation {
            entry.orientation = Some(orientation);
        }
    }
    entry.save(root)?;

    Ok(n)
}

/// List metadata for all session files, sorted by session number.
pub fn list_sessions(root: &Path, slug: &str) -> Result<Vec<workspace::SessionMeta>> {
    ensure_ponder_exists(root, slug)?;
    workspace::list_sessions(&ponder_dir(root, slug))
}

/// Read the full content of a specific session file.
pub fn read_session(root: &Path, slug: &str, n: u32) -> Result<String> {
    ensure_ponder_exists(root, slug)?;
    workspace::read_session(&ponder_dir(root, slug), n)
}

/// Public wrapper — parse `SessionMeta` from a session file's raw content.
pub fn parse_session_meta(content: &str) -> Option<workspace::SessionMeta> {
    workspace::parse_session_meta(content)
}

/// Return the next session number for this ponder entry (1 if no sessions yet).
pub fn next_session_number(root: &Path, slug: &str) -> Result<u32> {
    ensure_ponder_exists(root, slug)?;
    workspace::next_session_number(&ponder_dir(root, slug))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup(dir: &TempDir) {
        std::fs::create_dir_all(dir.path().join(".sdlc/roadmap")).unwrap();
    }

    #[test]
    fn ponder_create_load() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let entry = PonderEntry::create(dir.path(), "my-idea", "My Idea").unwrap();
        assert_eq!(entry.slug, "my-idea");
        assert_eq!(entry.status, PonderStatus::Exploring);
        assert_eq!(entry.sessions, 0);

        let loaded = PonderEntry::load(dir.path(), "my-idea").unwrap();
        assert_eq!(loaded.title, "My Idea");
        assert!(loaded.committed_at.is_none());
    }

    #[test]
    fn ponder_duplicate_fails() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        PonderEntry::create(dir.path(), "dupe", "Dupe").unwrap();
        assert!(matches!(
            PonderEntry::create(dir.path(), "dupe", "Dupe Again"),
            Err(SdlcError::PonderExists(_))
        ));
    }

    #[test]
    fn ponder_list_empty() {
        let dir = TempDir::new().unwrap();
        // No roadmap dir at all
        let entries = PonderEntry::list(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn ponder_list_sorted() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut e1 = PonderEntry::create(dir.path(), "aaa", "First").unwrap();
        // Fudge created_at so ordering is testable
        e1.created_at = Utc::now() - chrono::Duration::seconds(10);
        e1.save(dir.path()).unwrap();

        PonderEntry::create(dir.path(), "bbb", "Second").unwrap();

        let entries = PonderEntry::list(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].slug, "aaa");
        assert_eq!(entries[1].slug, "bbb");
    }

    #[test]
    fn ponder_status_update() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut entry = PonderEntry::create(dir.path(), "idea", "Idea").unwrap();
        assert_eq!(entry.status, PonderStatus::Exploring);

        entry.update_status(PonderStatus::Converging);
        assert_eq!(entry.status, PonderStatus::Converging);
        assert!(entry.committed_at.is_none());

        entry.update_status(PonderStatus::Committed);
        assert_eq!(entry.status, PonderStatus::Committed);
        assert!(entry.committed_at.is_some());
    }

    #[test]
    fn ponder_tags() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut entry = PonderEntry::create(dir.path(), "idea", "Idea").unwrap();
        assert!(entry.add_tag("ux"));
        assert!(entry.add_tag("backend"));
        assert!(!entry.add_tag("ux")); // duplicate
        assert_eq!(entry.tags, vec!["ux", "backend"]);
    }

    #[test]
    fn capture_content_and_list() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        PonderEntry::create(dir.path(), "idea", "Idea").unwrap();
        capture_content(
            dir.path(),
            "idea",
            "problem.md",
            "## Problem\n\nThe problem is...",
        )
        .unwrap();

        let artifacts = list_artifacts(dir.path(), "idea").unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].filename, "problem.md");
        assert!(artifacts[0].size_bytes > 0);
    }

    #[test]
    fn capture_file_and_read() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        PonderEntry::create(dir.path(), "idea", "Idea").unwrap();

        // Write a source file
        let src = dir.path().join("notes.txt");
        std::fs::write(&src, "my notes content").unwrap();

        capture_file(dir.path(), "idea", &src, "notes.txt").unwrap();

        let content = read_artifact(dir.path(), "idea", "notes.txt").unwrap();
        assert_eq!(content, "my notes content");
    }

    #[test]
    fn team_add_and_load() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        PonderEntry::create(dir.path(), "idea", "Idea").unwrap();

        let member = PonderTeamMember {
            name: "kai-tanaka".to_string(),
            role: "Architect".to_string(),
            context: "Built Spotify's preference engine".to_string(),
            agent: ".claude/agents/kai-tanaka.md".to_string(),
            recruited_at: Utc::now(),
        };

        let team = add_team_member(dir.path(), "idea", member).unwrap();
        assert_eq!(team.partners.len(), 1);
        assert_eq!(team.partners[0].name, "kai-tanaka");

        let loaded = load_team(dir.path(), "idea").unwrap();
        assert_eq!(loaded.partners.len(), 1);
    }

    #[test]
    fn team_duplicate_rejected() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        PonderEntry::create(dir.path(), "idea", "Idea").unwrap();

        let member = PonderTeamMember {
            name: "kai-tanaka".to_string(),
            role: "Architect".to_string(),
            context: "context".to_string(),
            agent: ".claude/agents/kai-tanaka.md".to_string(),
            recruited_at: Utc::now(),
        };
        add_team_member(dir.path(), "idea", member).unwrap();

        let dup = PonderTeamMember {
            name: "kai-tanaka".to_string(),
            role: "Different Role".to_string(),
            context: "different".to_string(),
            agent: ".claude/agents/kai-tanaka.md".to_string(),
            recruited_at: Utc::now(),
        };
        assert!(matches!(
            add_team_member(dir.path(), "idea", dup),
            Err(SdlcError::DuplicateTeamMember(_))
        ));
    }

    #[test]
    fn ponder_yaml_roundtrip() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut entry = PonderEntry::create(dir.path(), "idea", "Full Idea").unwrap();
        entry.add_tag("ux");
        entry.add_tag("api");
        entry.update_status(PonderStatus::Committed);
        entry.committed_to.push("milestone-1".to_string());
        entry.sessions = 5;
        entry.save(dir.path()).unwrap();

        let loaded = PonderEntry::load(dir.path(), "idea").unwrap();
        assert_eq!(loaded.slug, "idea");
        assert_eq!(loaded.status, PonderStatus::Committed);
        assert!(loaded.committed_at.is_some());
        assert_eq!(loaded.committed_to, vec!["milestone-1"]);
        assert_eq!(loaded.tags, vec!["ux", "api"]);
        assert_eq!(loaded.sessions, 5);
    }

    #[test]
    fn invalid_ponder_status_error() {
        let result: std::result::Result<PonderStatus, _> = "invalid".parse();
        assert!(matches!(result, Err(SdlcError::InvalidPonderStatus(_))));
    }

    #[test]
    fn session_log_and_list() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        PonderEntry::create(dir.path(), "idea", "Idea").unwrap();

        let content = r#"---
session: 1
timestamp: 2026-02-27T10:00:00Z
orientation:
  current: "Early discovery — exploring the problem space"
  next: "Recruit a skeptic and gather more context"
  commit: "When we have 2 competing designs and a tiebreaker"
---

KAI · Systems Architect
The core problem is retrieval, not storage.
"#;

        let n = log_session(dir.path(), "idea", content).unwrap();
        assert_eq!(n, 1);

        // Session counter incremented on manifest
        let entry = PonderEntry::load(dir.path(), "idea").unwrap();
        assert_eq!(entry.sessions, 1);
        // Orientation mirrored
        assert!(entry.orientation.is_some());
        let o = entry.orientation.unwrap();
        assert_eq!(o.current, "Early discovery — exploring the problem space");

        // Second session
        let content2 = r#"---
session: 2
timestamp: 2026-02-27T14:00:00Z
orientation:
  current: "Two designs on the table"
  next: "Get skeptic reaction on Option A"
  commit: "When skeptic approves one option"
---

JORDAN · Skeptic
Both assume memory is the problem. Prove it first.
"#;
        let n2 = log_session(dir.path(), "idea", content2).unwrap();
        assert_eq!(n2, 2);

        let sessions = list_sessions(dir.path(), "idea").unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].session, 1);
        assert_eq!(sessions[1].session, 2);
    }

    #[test]
    fn session_read_content() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        PonderEntry::create(dir.path(), "idea", "Idea").unwrap();

        let body = "---\nsession: 1\ntimestamp: 2026-02-27T10:00:00Z\n---\n\nHello session.\n";
        log_session(dir.path(), "idea", body).unwrap();

        let read_back = read_session(dir.path(), "idea", 1).unwrap();
        assert_eq!(read_back, body);
    }

    #[test]
    fn session_read_not_found() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        PonderEntry::create(dir.path(), "idea", "Idea").unwrap();

        assert!(matches!(
            read_session(dir.path(), "idea", 99),
            Err(SdlcError::SessionNotFound(99))
        ));
    }

    #[test]
    fn session_no_frontmatter_still_logs() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        PonderEntry::create(dir.path(), "idea", "Idea").unwrap();

        // Content without frontmatter — should log successfully, no orientation mirror
        let n = log_session(dir.path(), "idea", "Just plain content, no frontmatter.").unwrap();
        assert_eq!(n, 1);

        let entry = PonderEntry::load(dir.path(), "idea").unwrap();
        assert_eq!(entry.sessions, 1);
        assert!(entry.orientation.is_none());
    }

    #[test]
    fn path_traversal_rejected() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        PonderEntry::create(dir.path(), "idea", "Idea").unwrap();

        assert!(capture_content(dir.path(), "idea", "../escape.md", "bad").is_err());
        assert!(capture_content(dir.path(), "idea", "sub/dir.md", "bad").is_err());
        assert!(capture_content(dir.path(), "idea", "", "bad").is_err());
    }
}
