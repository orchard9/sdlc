//! Tool interaction persistence.
//!
//! Records every `--run` invocation for each tool, enabling history browsing
//! and audit trails. Each interaction is stored as a YAML file in
//! `.sdlc/tool-interactions/<tool-name>/`.
//!
//! AMA threads use a separate structure stored under
//! `.sdlc/tool-interactions/ama/threads/`.

use crate::{error::Result, io};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single `--run` invocation record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInteractionRecord {
    pub id: String,
    pub tool_name: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub input: serde_json::Value,
    /// None while the tool is running; populated on completion.
    pub result: Option<serde_json::Value>,
    /// "running" | "completed" | "failed"
    pub status: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub notes: Option<String>,
    /// true when a streaming log sidecar (`<id>.log`) exists.
    #[serde(default)]
    pub streaming_log: bool,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

pub fn interactions_dir(root: &Path, tool_name: &str) -> PathBuf {
    root.join(".sdlc").join("tool-interactions").join(tool_name)
}

fn interaction_path(root: &Path, tool_name: &str, id: &str) -> PathBuf {
    interactions_dir(root, tool_name).join(format!("{id}.yaml"))
}

// ---------------------------------------------------------------------------
// I/O functions
// ---------------------------------------------------------------------------

/// Persist an interaction record (create or overwrite).
pub fn save_interaction(root: &Path, record: &ToolInteractionRecord) -> Result<()> {
    let path = interaction_path(root, &record.tool_name, &record.id);
    let data = serde_yaml::to_string(record)?;
    io::atomic_write(&path, data.as_bytes())
}

/// Load a single interaction by tool name and id.
pub fn load_interaction(root: &Path, tool_name: &str, id: &str) -> Result<ToolInteractionRecord> {
    let path = interaction_path(root, tool_name, id);
    let data = std::fs::read_to_string(&path)?;
    let record = serde_yaml::from_str(&data)?;
    Ok(record)
}

/// List interactions for a tool, newest first. `limit = 0` returns all.
pub fn list_interactions(
    root: &Path,
    tool_name: &str,
    limit: usize,
) -> Result<Vec<ToolInteractionRecord>> {
    let dir = interactions_dir(root, tool_name);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut records: Vec<ToolInteractionRecord> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "yaml")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let data = std::fs::read_to_string(e.path()).ok()?;
            serde_yaml::from_str(&data).ok()
        })
        .collect();

    // Sort newest first by id (timestamp-based IDs sort lexicographically)
    records.sort_by(|a, b| b.id.cmp(&a.id));

    if limit > 0 {
        records.truncate(limit);
    }

    Ok(records)
}

/// Delete a single interaction file.
pub fn delete_interaction(root: &Path, tool_name: &str, id: &str) -> Result<()> {
    let path = interaction_path(root, tool_name, id);
    std::fs::remove_file(&path)?;
    Ok(())
}

/// Delete oldest interaction files until at most `max` remain.
pub fn enforce_interaction_retention(root: &Path, tool_name: &str, max: usize) {
    let dir = interactions_dir(root, tool_name);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut files: Vec<(PathBuf, String)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "yaml")
                .unwrap_or(false)
        })
        .map(|e| {
            let id = e
                .path()
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            (e.path(), id)
        })
        .collect();

    if files.len() <= max {
        return;
    }

    // Sort oldest first (IDs are timestamp-based, so lexicographic = chronological)
    files.sort_by(|a, b| a.1.cmp(&b.1));

    let to_remove = files.len() - max;
    for (path, _) in files.into_iter().take(to_remove) {
        let _ = std::fs::remove_file(path);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(tool_name: &str, id: &str, status: &str) -> ToolInteractionRecord {
        ToolInteractionRecord {
            id: id.to_string(),
            tool_name: tool_name.to_string(),
            created_at: id.to_string(),
            completed_at: None,
            input: serde_json::json!({}),
            result: None,
            status: status.to_string(),
            tags: Vec::new(),
            notes: None,
            streaming_log: false,
        }
    }

    #[test]
    fn save_and_load_interaction() {
        let dir = tempfile::TempDir::new().unwrap();
        let record = make_record("my-tool", "20260228-120000-abc", "completed");
        save_interaction(dir.path(), &record).unwrap();
        let loaded = load_interaction(dir.path(), "my-tool", "20260228-120000-abc").unwrap();
        assert_eq!(loaded.id, record.id);
        assert_eq!(loaded.tool_name, record.tool_name);
        assert_eq!(loaded.status, "completed");
    }

    #[test]
    fn list_interactions_returns_newest_first() {
        let dir = tempfile::TempDir::new().unwrap();
        for id in [
            "20260228-110000-aaa",
            "20260228-130000-bbb",
            "20260228-120000-ccc",
        ] {
            save_interaction(dir.path(), &make_record("tool", id, "completed")).unwrap();
        }
        let list = list_interactions(dir.path(), "tool", 0).unwrap();
        assert_eq!(list.len(), 3);
        // Newest first
        assert_eq!(list[0].id, "20260228-130000-bbb");
        assert_eq!(list[2].id, "20260228-110000-aaa");
    }

    #[test]
    fn list_interactions_respects_limit() {
        let dir = tempfile::TempDir::new().unwrap();
        for i in 0..5 {
            let id = format!("20260228-{i:06}-aaa");
            save_interaction(dir.path(), &make_record("tool", &id, "completed")).unwrap();
        }
        let list = list_interactions(dir.path(), "tool", 3).unwrap();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn delete_interaction_removes_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let record = make_record("tool", "20260228-100000-del", "completed");
        save_interaction(dir.path(), &record).unwrap();
        delete_interaction(dir.path(), "tool", "20260228-100000-del").unwrap();
        let list = list_interactions(dir.path(), "tool", 0).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn enforce_retention_removes_oldest() {
        let dir = tempfile::TempDir::new().unwrap();
        for i in 0..5u32 {
            let id = format!("2026022{i}-000000-aaa");
            save_interaction(dir.path(), &make_record("tool", &id, "completed")).unwrap();
        }
        enforce_interaction_retention(dir.path(), "tool", 3);
        let list = list_interactions(dir.path(), "tool", 0).unwrap();
        assert_eq!(list.len(), 3);
    }
}
