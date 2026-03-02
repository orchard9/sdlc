//! Feedback note queue — quick-capture notes that accumulate and can be
//! submitted to the ponder workspace as a single ideation entry.
//!
//! Layout:
//!   .sdlc/feedback.yaml   — list of pending feedback notes
//!
//! IDs are sequential: F1, F2, F3, …

use crate::error::{Result, SdlcError};
use crate::{io, paths};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A piece of contextual enrichment attached to a FeedbackNote.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Enrichment {
    /// Source identifier (e.g. "ux-review", "agent:advisor").
    pub source: String,
    /// The enrichment content.
    pub content: String,
    /// When this enrichment was added.
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackNote {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    /// When this note was last updated (content change).
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    /// Contextual enrichments attached to this note.
    #[serde(default)]
    pub enrichments: Vec<Enrichment>,
}

// ---------------------------------------------------------------------------
// Internal file I/O
// ---------------------------------------------------------------------------

fn load_all(root: &Path) -> Result<Vec<FeedbackNote>> {
    let path = paths::feedback_path(root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_yaml::from_str(&content)?)
}

fn save_all(root: &Path, notes: &[FeedbackNote]) -> Result<()> {
    let path = paths::feedback_path(root);
    let content = serde_yaml::to_string(notes)?;
    io::atomic_write(&path, content.as_bytes())
}

fn next_id(notes: &[FeedbackNote]) -> String {
    // Use the highest numeric suffix + 1 so deletes don't reset the counter.
    let max = notes
        .iter()
        .filter_map(|n| n.id.strip_prefix('F')?.parse::<usize>().ok())
        .max()
        .unwrap_or(0);
    format!("F{}", max + 1)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Add a new feedback note. Returns the created note.
pub fn add(root: &Path, content: impl Into<String>) -> Result<FeedbackNote> {
    let content = content.into();
    let mut notes = load_all(root)?;
    let id = next_id(&notes);
    let now = Utc::now();
    let note = FeedbackNote {
        id,
        content,
        created_at: now,
        updated_at: now,
        enrichments: Vec::new(),
    };
    notes.push(note.clone());
    save_all(root, &notes)?;
    Ok(note)
}

/// List all feedback notes (oldest first).
pub fn list(root: &Path) -> Result<Vec<FeedbackNote>> {
    load_all(root)
}

/// Delete a feedback note by ID. Returns `Ok(())` if deleted, error if not found.
pub fn delete(root: &Path, id: &str) -> Result<bool> {
    let mut notes = load_all(root)?;
    let before = notes.len();
    notes.retain(|n| n.id != id);
    if notes.len() == before {
        return Ok(false);
    }
    save_all(root, &notes)?;
    Ok(true)
}

/// Clear all feedback notes.
pub fn clear(root: &Path) -> Result<()> {
    save_all(root, &[])
}

/// Update the content of an existing note. Returns the updated note, or `None`
/// if the ID was not found.
pub fn update(
    root: &Path,
    id: &str,
    new_content: impl Into<String>,
) -> Result<Option<FeedbackNote>> {
    let mut notes = load_all(root)?;
    let new_content = new_content.into();
    let mut updated = None;
    for note in notes.iter_mut() {
        if note.id == id {
            note.content = new_content.clone();
            note.updated_at = Utc::now();
            updated = Some(note.clone());
            break;
        }
    }
    if updated.is_some() {
        save_all(root, &notes)?;
    }
    Ok(updated)
}

/// Attach an enrichment to an existing note. Returns the updated note.
///
/// Returns `Err(SdlcError::FeedbackNoteNotFound)` when `id` does not match
/// any note.
pub fn enrich(
    root: &Path,
    id: &str,
    source: impl Into<String>,
    content: impl Into<String>,
) -> Result<FeedbackNote> {
    let mut notes = load_all(root)?;
    let enrichment = Enrichment {
        source: source.into(),
        content: content.into(),
        added_at: Utc::now(),
    };
    let mut found = false;
    let mut result = None;
    for note in notes.iter_mut() {
        if note.id == id {
            note.enrichments.push(enrichment.clone());
            note.updated_at = Utc::now();
            result = Some(note.clone());
            found = true;
            break;
        }
    }
    if !found {
        return Err(SdlcError::FeedbackNoteNotFound(id.to_string()));
    }
    save_all(root, &notes)?;
    Ok(result.unwrap())
}

/// Bundle all notes into a markdown document suitable as ponder context.
pub fn to_markdown(notes: &[FeedbackNote]) -> String {
    let mut out = String::from("# Feedback Notes\n\n");
    for note in notes {
        out.push_str(&format!(
            "**{}** — _{}_\n\n{}\n\n",
            note.id,
            note.created_at.format("%Y-%m-%d %H:%M UTC"),
            note.content
        ));
        if !note.enrichments.is_empty() {
            out.push_str("**Enrichments:**\n\n");
            for e in &note.enrichments {
                out.push_str(&format!(
                    "- *{}* ({}): {}\n",
                    e.source,
                    e.added_at.format("%Y-%m-%d %H:%M UTC"),
                    e.content
                ));
            }
            out.push('\n');
        }
        out.push_str("---\n\n");
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn init_dir() -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        dir
    }

    #[test]
    fn add_and_list() {
        let dir = init_dir();
        let note = add(dir.path(), "This is feedback").unwrap();
        assert_eq!(note.id, "F1");
        assert_eq!(note.content, "This is feedback");

        let notes = list(dir.path()).unwrap();
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn sequential_ids() {
        let dir = init_dir();
        let n1 = add(dir.path(), "First").unwrap();
        let n2 = add(dir.path(), "Second").unwrap();
        assert_eq!(n1.id, "F1");
        assert_eq!(n2.id, "F2");
    }

    #[test]
    fn delete_note() {
        let dir = init_dir();
        add(dir.path(), "Keep").unwrap();
        add(dir.path(), "Remove").unwrap();

        let deleted = delete(dir.path(), "F2").unwrap();
        assert!(deleted);

        let notes = list(dir.path()).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, "F1");
    }

    #[test]
    fn delete_missing_returns_false() {
        let dir = init_dir();
        let deleted = delete(dir.path(), "F99").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn id_does_not_reset_after_delete() {
        let dir = init_dir();
        add(dir.path(), "A").unwrap();
        add(dir.path(), "B").unwrap();
        delete(dir.path(), "F2").unwrap();
        let n3 = add(dir.path(), "C").unwrap();
        assert_eq!(n3.id, "F2"); // next sequential after max(F1) = F2
    }

    #[test]
    fn clear_removes_all() {
        let dir = init_dir();
        add(dir.path(), "A").unwrap();
        add(dir.path(), "B").unwrap();
        clear(dir.path()).unwrap();
        assert!(list(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn to_markdown_format() {
        let dir = init_dir();
        add(dir.path(), "Some idea").unwrap();
        let notes = list(dir.path()).unwrap();
        let md = to_markdown(&notes);
        assert!(md.contains("# Feedback Notes"));
        assert!(md.contains("F1"));
        assert!(md.contains("Some idea"));
    }

    // --- enrichment tests ---

    #[test]
    fn update_content() {
        let dir = init_dir();
        add(dir.path(), "Original content").unwrap();
        let updated = update(dir.path(), "F1", "Updated content").unwrap();
        assert!(updated.is_some());
        let note = updated.unwrap();
        assert_eq!(note.content, "Updated content");
        assert_eq!(note.id, "F1");

        // Verify persisted
        let notes = list(dir.path()).unwrap();
        assert_eq!(notes[0].content, "Updated content");
    }

    #[test]
    fn update_missing_returns_none() {
        let dir = init_dir();
        let result = update(dir.path(), "F99", "New content").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn enrich_adds_enrichment() {
        let dir = init_dir();
        add(dir.path(), "Needs context").unwrap();
        let note = enrich(dir.path(), "F1", "ux-review", "Users find this confusing").unwrap();
        assert_eq!(note.enrichments.len(), 1);
        assert_eq!(note.enrichments[0].source, "ux-review");
        assert_eq!(note.enrichments[0].content, "Users find this confusing");

        // Verify persisted
        let notes = list(dir.path()).unwrap();
        assert_eq!(notes[0].enrichments.len(), 1);
    }

    #[test]
    fn enrich_missing_returns_error() {
        let dir = init_dir();
        let result = enrich(dir.path(), "F99", "src", "ctx");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, SdlcError::FeedbackNoteNotFound(_)),
            "expected FeedbackNoteNotFound, got {err:?}"
        );
    }

    #[test]
    fn enrich_multiple_accumulates() {
        let dir = init_dir();
        add(dir.path(), "Multi").unwrap();
        enrich(dir.path(), "F1", "src-a", "First enrichment").unwrap();
        let note = enrich(dir.path(), "F1", "src-b", "Second enrichment").unwrap();
        assert_eq!(note.enrichments.len(), 2);

        let notes = list(dir.path()).unwrap();
        assert_eq!(notes[0].enrichments.len(), 2);
    }

    #[test]
    fn old_yaml_backward_compat_no_enrichments() {
        let dir = init_dir();
        // Write a YAML that looks like old-format (no updated_at, no enrichments)
        let old_yaml = "- id: F1\n  content: Old note\n  created_at: \"2024-01-01T00:00:00Z\"\n";
        let path = dir.path().join(".sdlc").join("feedback.yaml");
        std::fs::write(&path, old_yaml).unwrap();

        let notes = list(dir.path()).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, "F1");
        assert!(
            notes[0].enrichments.is_empty(),
            "old notes default to empty enrichments"
        );
    }

    #[test]
    fn to_markdown_includes_enrichments() {
        let dir = init_dir();
        add(dir.path(), "Main point").unwrap();
        enrich(dir.path(), "F1", "research", "Supporting evidence here").unwrap();
        let notes = list(dir.path()).unwrap();
        let md = to_markdown(&notes);
        assert!(md.contains("Enrichments"));
        assert!(md.contains("research"));
        assert!(md.contains("Supporting evidence here"));
    }

    #[test]
    fn to_markdown_no_enrichment_section_when_empty() {
        let dir = init_dir();
        add(dir.path(), "Plain note").unwrap();
        let notes = list(dir.path()).unwrap();
        let md = to_markdown(&notes);
        assert!(
            !md.contains("Enrichments"),
            "no enrichments section when empty"
        );
    }
}
