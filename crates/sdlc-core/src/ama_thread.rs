//! AMA thread persistence.
//!
//! Each thread is a conversation of question+answer turns stored in
//! `.sdlc/tool-interactions/ama/threads/<id>/`.
//! - `manifest.yaml` — thread metadata
//! - `turns/turn-NNN.yaml` — individual turn records

use crate::{error::Result, io};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Thread-level metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AmaThread {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub turn_count: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    /// "feature:<slug>" | "ponder:<slug>" — set when thread is committed.
    pub committed_to: Option<String>,
}

/// A single turn within a thread.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AmaTurn {
    pub turn_index: u32,
    pub question: String,
    #[serde(default)]
    pub sources: Vec<serde_json::Value>,
    /// None until the agent run completes and the frontend POCTs the synthesis.
    pub synthesis: Option<String>,
    pub run_id: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

pub fn threads_dir(root: &Path) -> PathBuf {
    root.join(".sdlc")
        .join("tool-interactions")
        .join("ama")
        .join("threads")
}

pub fn thread_dir(root: &Path, id: &str) -> PathBuf {
    threads_dir(root).join(id)
}

fn thread_manifest(root: &Path, id: &str) -> PathBuf {
    thread_dir(root, id).join("manifest.yaml")
}

fn turns_dir(root: &Path, id: &str) -> PathBuf {
    thread_dir(root, id).join("turns")
}

fn turn_path(root: &Path, thread_id: &str, turn_index: u32) -> PathBuf {
    turns_dir(root, thread_id).join(format!("turn-{turn_index:03}.yaml"))
}

// ---------------------------------------------------------------------------
// Thread CRUD
// ---------------------------------------------------------------------------

/// Create a new thread with the given id and title.
pub fn create_thread(root: &Path, id: &str, title: &str) -> Result<AmaThread> {
    let now = chrono::Utc::now().to_rfc3339();
    let thread = AmaThread {
        id: id.to_string(),
        title: title.to_string(),
        created_at: now.clone(),
        updated_at: now,
        turn_count: 0,
        tags: Vec::new(),
        committed_to: None,
    };
    save_thread(root, &thread)?;
    Ok(thread)
}

/// Load a thread's manifest.
pub fn load_thread(root: &Path, id: &str) -> Result<AmaThread> {
    let path = thread_manifest(root, id);
    let data = std::fs::read_to_string(&path)?;
    let thread = serde_yaml::from_str(&data)?;
    Ok(thread)
}

/// Persist a thread manifest.
pub fn save_thread(root: &Path, thread: &AmaThread) -> Result<()> {
    let path = thread_manifest(root, &thread.id);
    let data = serde_yaml::to_string(thread)?;
    io::atomic_write(&path, data.as_bytes())
}

/// List all threads, newest first.
pub fn list_threads(root: &Path, limit: usize) -> Result<Vec<AmaThread>> {
    let dir = threads_dir(root);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut threads: Vec<AmaThread> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let manifest = e.path().join("manifest.yaml");
            let data = std::fs::read_to_string(manifest).ok()?;
            serde_yaml::from_str(&data).ok()
        })
        .collect();

    threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    if limit > 0 {
        threads.truncate(limit);
    }

    Ok(threads)
}

/// Delete a thread and all its turns.
pub fn delete_thread(root: &Path, id: &str) -> Result<()> {
    let dir = thread_dir(root, id);
    std::fs::remove_dir_all(&dir)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Turn CRUD
// ---------------------------------------------------------------------------

/// Persist a turn record.
pub fn save_turn(root: &Path, thread_id: &str, turn: &AmaTurn) -> Result<()> {
    let path = turn_path(root, thread_id, turn.turn_index);
    let data = serde_yaml::to_string(turn)?;
    io::atomic_write(&path, data.as_bytes())
}

/// Load a turn by index.
pub fn load_turn(root: &Path, thread_id: &str, turn_index: u32) -> Result<AmaTurn> {
    let path = turn_path(root, thread_id, turn_index);
    let data = std::fs::read_to_string(&path)?;
    let turn = serde_yaml::from_str(&data)?;
    Ok(turn)
}

/// List all turns for a thread, ordered by turn_index ascending.
pub fn list_turns(root: &Path, thread_id: &str) -> Result<Vec<AmaTurn>> {
    let dir = turns_dir(root, thread_id);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut turns: Vec<AmaTurn> = std::fs::read_dir(&dir)?
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

    turns.sort_by_key(|t| t.turn_index);
    Ok(turns)
}

/// Update the synthesis text for a turn.
pub fn update_turn_synthesis(
    root: &Path,
    thread_id: &str,
    turn_index: u32,
    synthesis: &str,
) -> Result<()> {
    let mut turn = load_turn(root, thread_id, turn_index)?;
    turn.synthesis = Some(synthesis.to_string());
    turn.completed_at = Some(chrono::Utc::now().to_rfc3339());
    save_turn(root, thread_id, &turn)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_load_thread() {
        let dir = tempfile::TempDir::new().unwrap();
        let t = create_thread(dir.path(), "20260228-thread-abc", "My AMA Thread").unwrap();
        assert_eq!(t.id, "20260228-thread-abc");
        assert_eq!(t.turn_count, 0);

        let loaded = load_thread(dir.path(), "20260228-thread-abc").unwrap();
        assert_eq!(loaded.title, "My AMA Thread");
    }

    #[test]
    fn save_and_load_turn() {
        let dir = tempfile::TempDir::new().unwrap();
        create_thread(dir.path(), "t1", "Thread 1").unwrap();

        let turn = AmaTurn {
            turn_index: 0,
            question: "What is the auth flow?".to_string(),
            sources: Vec::new(),
            synthesis: None,
            run_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        };
        save_turn(dir.path(), "t1", &turn).unwrap();
        let loaded = load_turn(dir.path(), "t1", 0).unwrap();
        assert_eq!(loaded.question, "What is the auth flow?");
    }

    #[test]
    fn update_turn_synthesis_sets_text() {
        let dir = tempfile::TempDir::new().unwrap();
        create_thread(dir.path(), "t2", "Thread 2").unwrap();

        let turn = AmaTurn {
            turn_index: 0,
            question: "Question?".to_string(),
            sources: Vec::new(),
            synthesis: None,
            run_id: Some("run-abc".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        };
        save_turn(dir.path(), "t2", &turn).unwrap();
        update_turn_synthesis(dir.path(), "t2", 0, "The answer is...").unwrap();

        let loaded = load_turn(dir.path(), "t2", 0).unwrap();
        assert_eq!(loaded.synthesis.as_deref(), Some("The answer is..."));
        assert!(loaded.completed_at.is_some());
    }

    #[test]
    fn list_threads_newest_first() {
        let dir = tempfile::TempDir::new().unwrap();
        create_thread(dir.path(), "id-a", "Alpha").unwrap();

        // Update updated_at manually by saving with a later timestamp
        let mut t2 = create_thread(dir.path(), "id-b", "Beta").unwrap();
        t2.updated_at = "2099-01-01T00:00:00Z".to_string();
        save_thread(dir.path(), &t2).unwrap();

        let threads = list_threads(dir.path(), 0).unwrap();
        assert_eq!(threads[0].id, "id-b");
    }

    #[test]
    fn delete_thread_removes_directory() {
        let dir = tempfile::TempDir::new().unwrap();
        create_thread(dir.path(), "del-me", "Deletable").unwrap();
        delete_thread(dir.path(), "del-me").unwrap();
        let threads = list_threads(dir.path(), 0).unwrap();
        assert!(threads.is_empty());
    }
}
