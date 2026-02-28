//! Feedback note queue — quick-capture notes that accumulate and can be
//! submitted to the ponder workspace as a single ideation entry.
//!
//! Layout:
//!   .sdlc/feedback.yaml   — list of pending feedback notes
//!
//! IDs are sequential: F1, F2, F3, …

use crate::error::Result;
use crate::{io, paths};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackNote {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
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
    let note = FeedbackNote {
        id,
        content,
        created_at: Utc::now(),
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

/// Bundle all notes into a markdown document suitable as ponder context.
pub fn to_markdown(notes: &[FeedbackNote]) -> String {
    let mut out = String::from("# Feedback Notes\n\n");
    for note in notes {
        out.push_str(&format!(
            "**{}** — _{}_\n\n{}\n\n---\n\n",
            note.id,
            note.created_at.format("%Y-%m-%d %H:%M UTC"),
            note.content
        ));
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
}
