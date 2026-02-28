//! Generic session and artifact primitives.
//!
//! All functions operate on a concrete directory (`dir: &Path`).
//! Neither ponder nor investigation details leak into this module —
//! it is a pure I/O layer that any workspace-like entity can reuse.

use crate::error::{Result, SdlcError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Orientation compass written by the agent at the end of each session.
/// Shared by ponder and all investigation types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orientation {
    /// Where we are — a one-liner describing the current state of thinking.
    pub current: String,
    /// What should happen next — concrete next action or focus.
    pub next: String,
    /// What unlocks commitment — the condition that signals readiness to proceed.
    pub commit: String,
}

/// Metadata extracted from a session file's YAML frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub session: u32,
    pub timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
}

/// File metadata for a workspace artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMeta {
    pub filename: String,
    pub size_bytes: u64,
    pub modified_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Reject filenames that could escape the directory via path traversal.
pub fn validate_artifact_filename(filename: &str) -> Result<()> {
    if filename.is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
        || filename.contains('\0')
    {
        return Err(SdlcError::InvalidArtifactFilename(filename.to_string()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Frontmatter parsing
// ---------------------------------------------------------------------------

/// Extract the YAML content between the first pair of `---` delimiters.
fn extract_frontmatter(content: &str) -> Option<&str> {
    let rest = content.strip_prefix("---")?;
    let rest = if let Some(r) = rest.strip_prefix('\n') {
        r
    } else if let Some(r) = rest.strip_prefix("\r\n") {
        r
    } else {
        return None;
    };
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Parse `SessionMeta` from a session file's raw content.
pub fn parse_session_meta(content: &str) -> Option<SessionMeta> {
    let fm = extract_frontmatter(content)?;
    serde_yaml::from_str(fm).ok()
}

// ---------------------------------------------------------------------------
// Session path helpers
// ---------------------------------------------------------------------------

/// Returns `dir/sessions/`.
pub fn sessions_dir(dir: &Path) -> PathBuf {
    dir.join("sessions")
}

/// Returns `dir/sessions/session-NNN.md`.
pub fn session_path(dir: &Path, n: u32) -> PathBuf {
    sessions_dir(dir).join(format!("session-{n:03}.md"))
}

pub fn next_session_number(dir: &Path) -> Result<u32> {
    let sdir = sessions_dir(dir);
    if !sdir.exists() {
        return Ok(1);
    }
    let mut max = 0u32;
    for entry in std::fs::read_dir(&sdir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(stem) = name.strip_suffix(".md") {
            if let Some(num_str) = stem.strip_prefix("session-") {
                if let Ok(n) = num_str.parse::<u32>() {
                    if n > max {
                        max = n;
                    }
                }
            }
        }
    }
    Ok(max + 1)
}

// ---------------------------------------------------------------------------
// Session functions
// ---------------------------------------------------------------------------

/// Write a session file to `dir/sessions/session-NNN.md`.
///
/// Returns the session number assigned. Does **not** update any manifest —
/// that is the caller's responsibility so each entry type can do it differently.
pub fn write_session(dir: &Path, content: &str) -> Result<u32> {
    let n = next_session_number(dir)?;
    let sdir = sessions_dir(dir);
    if !sdir.exists() {
        std::fs::create_dir_all(&sdir)?;
    }
    let path = session_path(dir, n);
    crate::io::atomic_write(&path, content.as_bytes())?;
    Ok(n)
}

/// List session metadata sorted ascending by session number.
pub fn list_sessions(dir: &Path) -> Result<Vec<SessionMeta>> {
    let sdir = sessions_dir(dir);
    if !sdir.exists() {
        return Ok(Vec::new());
    }
    let mut metas = Vec::new();
    for entry in std::fs::read_dir(&sdir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.ends_with(".md") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path())?;
        if let Some(meta) = parse_session_meta(&content) {
            metas.push(meta);
        }
    }
    metas.sort_by_key(|m| m.session);
    Ok(metas)
}

/// Read the full content of session `n`.
pub fn read_session(dir: &Path, n: u32) -> Result<String> {
    let path = session_path(dir, n);
    if !path.exists() {
        return Err(SdlcError::SessionNotFound(n));
    }
    Ok(std::fs::read_to_string(&path)?)
}

// ---------------------------------------------------------------------------
// Artifact functions
// ---------------------------------------------------------------------------

/// Write content to a named file in `dir`.
pub fn write_artifact(dir: &Path, filename: &str, content: &str) -> Result<()> {
    validate_artifact_filename(filename)?;
    let path = dir.join(filename);
    crate::io::atomic_write(&path, content.as_bytes())
}

/// Copy a file from `src` into `dir` under `filename`.
pub fn write_artifact_from_file(dir: &Path, src: &Path, filename: &str) -> Result<()> {
    validate_artifact_filename(filename)?;
    let content = std::fs::read(src)?;
    let dest = dir.join(filename);
    crate::io::atomic_write(&dest, &content)
}

/// List files in `dir`, skipping entries in `skip` and any subdirectories.
pub fn list_artifacts(dir: &Path, skip: &[&str]) -> Result<Vec<ArtifactMeta>> {
    let mut artifacts = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if skip.contains(&name.as_str()) || entry.file_type()?.is_dir() {
            continue;
        }
        let meta = entry.metadata()?;
        let modified_at = meta
            .modified()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(|_| Utc::now());
        artifacts.push(ArtifactMeta {
            filename: name,
            size_bytes: meta.len(),
            modified_at,
        });
    }
    artifacts.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(artifacts)
}

/// Read the content of a named file in `dir`.
pub fn read_artifact(dir: &Path, filename: &str) -> Result<String> {
    validate_artifact_filename(filename)?;
    let path = dir.join(filename);
    if !path.exists() {
        return Err(SdlcError::ArtifactNotFound(filename.to_string()));
    }
    Ok(std::fs::read_to_string(&path)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let p = dir.path().to_path_buf();
        (dir, p)
    }

    #[test]
    fn write_and_read_session() {
        let (_dir, p) = tmp();
        let content = "---\nsession: 1\ntimestamp: 2026-02-27T10:00:00Z\n---\n\nHello.\n";
        let n = write_session(&p, content).unwrap();
        assert_eq!(n, 1);
        let back = read_session(&p, 1).unwrap();
        assert_eq!(back, content);
    }

    #[test]
    fn session_numbering_increments() {
        let (_dir, p) = tmp();
        let c = "---\nsession: 1\ntimestamp: 2026-02-27T10:00:00Z\n---\nbody";
        let n1 = write_session(&p, c).unwrap();
        let n2 = write_session(&p, c).unwrap();
        assert_eq!(n1, 1);
        assert_eq!(n2, 2);
    }

    #[test]
    fn list_sessions_sorted() {
        let (_dir, p) = tmp();
        let mk = |n: u32| format!("---\nsession: {n}\ntimestamp: 2026-02-27T10:00:00Z\n---\nbody");
        write_session(&p, &mk(1)).unwrap();
        write_session(&p, &mk(2)).unwrap();
        let list = list_sessions(&p).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].session, 1);
        assert_eq!(list[1].session, 2);
    }

    #[test]
    fn read_session_not_found() {
        let (_dir, p) = tmp();
        assert!(matches!(
            read_session(&p, 99),
            Err(SdlcError::SessionNotFound(99))
        ));
    }

    #[test]
    fn write_and_list_artifacts() {
        let (_dir, p) = tmp();
        write_artifact(&p, "notes.md", "hello").unwrap();
        let arts = list_artifacts(&p, &["manifest.yaml"]).unwrap();
        assert_eq!(arts.len(), 1);
        assert_eq!(arts[0].filename, "notes.md");
    }

    #[test]
    fn list_artifacts_skips_and_no_subdirs() {
        let (_dir, p) = tmp();
        write_artifact(&p, "manifest.yaml", "x").unwrap();
        write_artifact(&p, "notes.md", "y").unwrap();
        std::fs::create_dir_all(p.join("sessions")).unwrap();
        let arts = list_artifacts(&p, &["manifest.yaml"]).unwrap();
        assert_eq!(arts.len(), 1);
        assert_eq!(arts[0].filename, "notes.md");
    }

    #[test]
    fn read_artifact_ok() {
        let (_dir, p) = tmp();
        write_artifact(&p, "a.md", "content").unwrap();
        assert_eq!(read_artifact(&p, "a.md").unwrap(), "content");
    }

    #[test]
    fn read_artifact_not_found() {
        let (_dir, p) = tmp();
        assert!(matches!(
            read_artifact(&p, "missing.md"),
            Err(SdlcError::ArtifactNotFound(_))
        ));
    }

    #[test]
    fn artifact_path_traversal_rejected() {
        let (_dir, p) = tmp();
        assert!(write_artifact(&p, "../escape", "bad").is_err());
        assert!(write_artifact(&p, "sub/dir.md", "bad").is_err());
        assert!(write_artifact(&p, "", "bad").is_err());
    }

    #[test]
    fn parse_session_meta_extracts_orientation() {
        let content = "---\nsession: 1\ntimestamp: 2026-02-27T10:00:00Z\norientation:\n  current: \"here\"\n  next: \"there\"\n  commit: \"when done\"\n---\nbody";
        let meta = parse_session_meta(content).unwrap();
        assert_eq!(meta.session, 1);
        let o = meta.orientation.unwrap();
        assert_eq!(o.current, "here");
    }

    #[test]
    fn parse_session_meta_no_frontmatter_returns_none() {
        assert!(parse_session_meta("just plain content").is_none());
    }
}
