use std::path::{Path, PathBuf};

use crate::{ClaudeAgentError, Result};

// ─── SessionStore ─────────────────────────────────────────────────────────

/// Persists Claude session IDs on disk so runs can be resumed.
///
/// Each feature slug gets its own `.session` file under
/// `<project_root>/.sdlc/sessions/`. The stored value is the bare session
/// ID string emitted by `claude --output-format stream-json` in the initial
/// `system/init` message.
///
/// # Usage
///
/// ```rust,ignore
/// let store = SessionStore::new(project_root);
///
/// // Before a run: check for an existing session to resume
/// let opts = QueryOptions {
///     resume: store.load("my-feature"),
///     ..Default::default()
/// };
///
/// // After a run: save the session ID for next time
/// let session_id = result_message.session_id();
/// store.save("my-feature", session_id)?;
///
/// // On explicit reset:
/// store.clear("my-feature")?;
/// ```
pub struct SessionStore {
    sessions_dir: PathBuf,
}

impl SessionStore {
    /// Create a `SessionStore` rooted at `project_root`.
    ///
    /// Session files live at `<project_root>/.sdlc/sessions/<slug>.session`.
    /// The directory is created lazily on the first `save`.
    pub fn new(project_root: &Path) -> Self {
        SessionStore {
            sessions_dir: project_root.join(".sdlc").join("sessions"),
        }
    }

    /// Return the stored session ID for `slug`, or `None` if none exists.
    pub fn load(&self, slug: &str) -> Option<String> {
        let id = std::fs::read_to_string(self.path(slug))
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty());
        id
    }

    /// Persist `session_id` for `slug`.
    ///
    /// Creates the sessions directory if it does not yet exist.
    pub fn save(&self, slug: &str, session_id: &str) -> Result<()> {
        std::fs::create_dir_all(&self.sessions_dir).map_err(ClaudeAgentError::Io)?;
        std::fs::write(self.path(slug), session_id).map_err(ClaudeAgentError::Io)
    }

    /// Delete the stored session for `slug` (no-op if none exists).
    pub fn clear(&self, slug: &str) -> Result<()> {
        let p = self.path(slug);
        if p.exists() {
            std::fs::remove_file(&p).map_err(ClaudeAgentError::Io)?;
        }
        Ok(())
    }

    fn path(&self, slug: &str) -> PathBuf {
        self.sessions_dir.join(format!("{slug}.session"))
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn store() -> (SessionStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = SessionStore::new(dir.path());
        (store, dir)
    }

    #[test]
    fn load_returns_none_when_no_file() {
        let (store, _dir) = store();
        assert_eq!(store.load("my-feature"), None);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (store, _dir) = store();
        store.save("my-feature", "sess-abc-123").unwrap();
        assert_eq!(store.load("my-feature"), Some("sess-abc-123".into()));
    }

    #[test]
    fn load_trims_whitespace() {
        let (store, _dir) = store();
        store.save("my-feature", "sess-abc\n").unwrap();
        assert_eq!(store.load("my-feature"), Some("sess-abc".into()));
    }

    #[test]
    fn clear_removes_session() {
        let (store, _dir) = store();
        store.save("slug", "abc").unwrap();
        store.clear("slug").unwrap();
        assert_eq!(store.load("slug"), None);
    }

    #[test]
    fn clear_is_noop_when_no_session() {
        let (store, _dir) = store();
        // Should not error
        store.clear("nonexistent").unwrap();
    }

    #[test]
    fn creates_sessions_dir_on_first_save() {
        let (store, _dir) = store();
        assert!(!store.sessions_dir.exists());
        store.save("slug", "abc").unwrap();
        assert!(store.sessions_dir.exists());
    }

    #[test]
    fn different_slugs_are_independent() {
        let (store, _dir) = store();
        store.save("feat-a", "aaa").unwrap();
        store.save("feat-b", "bbb").unwrap();
        assert_eq!(store.load("feat-a"), Some("aaa".into()));
        assert_eq!(store.load("feat-b"), Some("bbb".into()));
        store.clear("feat-a").unwrap();
        assert_eq!(store.load("feat-a"), None);
        assert_eq!(store.load("feat-b"), Some("bbb".into()));
    }
}
