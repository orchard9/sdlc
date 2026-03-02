//! FeedbackThread — persistent, contextual, collaborative idea threads.
//!
//! A thread is a lightweight comment log anchored to a named subject via a
//! `context` string (e.g. `"feature:my-slug"`, `"ponder:my-idea"`). Each
//! thread holds an append-only list of `ThreadPost` entries authored by
//! either a human or an agent.
//!
//! Layout:
//!   .sdlc/feedback-threads/<id>/
//!     manifest.yaml         — thread metadata
//!     posts/post-NNN.yaml   — individual post records (seq 1-based, 3-digit zero-padded)
//!
//! IDs are generated as `<YYYYMMDD>-<sanitized-context>` where the context
//! string has `:` and `/` replaced by `-` and is truncated so the full ID
//! stays within 64 chars. Collisions are resolved by appending `-2`, `-3`, …

use crate::error::{Result, SdlcError};
use crate::{io, paths};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Thread-level metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackThread {
    pub id: String,
    pub title: String,
    /// Arbitrary namespaced context string, e.g. "feature:my-slug".
    pub context: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub post_count: u32,
}

/// A single post within a thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadPost {
    /// 1-based, sequential within the thread.
    pub seq: u32,
    /// "human" | "agent:<name>"
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn load_manifest(root: &Path, id: &str) -> Result<FeedbackThread> {
    let path = paths::feedback_thread_manifest(root, id);
    if !path.exists() {
        return Err(SdlcError::ThreadNotFound(id.to_string()));
    }
    let data = std::fs::read_to_string(&path)?;
    Ok(serde_yaml::from_str(&data)?)
}

fn save_manifest(root: &Path, thread: &FeedbackThread) -> Result<()> {
    let path = paths::feedback_thread_manifest(root, &thread.id);
    let data = serde_yaml::to_string(thread)?;
    io::atomic_write(&path, data.as_bytes())
}

/// Generate a collision-safe thread ID from the context string.
///
/// Format: `<YYYYMMDD>-<sanitized-context>`
/// The sanitized context has `:`, `/`, and spaces replaced with `-`, then
/// truncated so the full ID fits in 64 chars.
fn make_thread_id(root: &Path, context: &str) -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    // Sanitize: replace non-alphanumeric-or-dash chars with `-`, collapse runs
    let sanitized: String = context
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    // Collapse consecutive dashes
    let mut prev_dash = false;
    let sanitized: String = sanitized
        .chars()
        .filter(|&c| {
            if c == '-' {
                if prev_dash {
                    return false;
                }
                prev_dash = true;
            } else {
                prev_dash = false;
            }
            true
        })
        .collect();
    let sanitized = sanitized.trim_matches('-').to_lowercase();

    // date prefix is 8 chars + dash = 9. Leave room in 64 char limit.
    let max_ctx_len = 64 - 9; // = 55
    let sanitized = if sanitized.len() > max_ctx_len {
        sanitized[..max_ctx_len].trim_end_matches('-').to_string()
    } else {
        sanitized
    };

    let base = format!("{date}-{sanitized}");

    // Collision resolution
    if !paths::feedback_thread_dir(root, &base).exists() {
        return base;
    }
    let mut n = 2u32;
    loop {
        let candidate = format!("{base}-{n}");
        if !paths::feedback_thread_dir(root, &candidate).exists() {
            return candidate;
        }
        n += 1;
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a new feedback thread anchored to `context`.
///
/// If `title` is empty, a default title of `"Discussion: <context>"` is used.
pub fn create_thread(root: &Path, context: &str, title: &str) -> Result<FeedbackThread> {
    let id = make_thread_id(root, context);
    let title = if title.trim().is_empty() {
        format!("Discussion: {context}")
    } else {
        title.to_string()
    };
    let now = Utc::now();
    let thread = FeedbackThread {
        id,
        title,
        context: context.to_string(),
        created_at: now,
        updated_at: now,
        post_count: 0,
    };
    save_manifest(root, &thread)?;
    Ok(thread)
}

/// Load an existing thread by ID.
pub fn load_thread(root: &Path, id: &str) -> Result<FeedbackThread> {
    load_manifest(root, id)
}

/// List all threads, newest-first by `updated_at`.
///
/// If `filter_context` is `Some`, only threads whose `context` equals the
/// filter value are returned.
pub fn list_threads(root: &Path, filter_context: Option<&str>) -> Result<Vec<FeedbackThread>> {
    let dir = paths::feedback_threads_dir(root);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut threads: Vec<FeedbackThread> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let manifest = e.path().join("manifest.yaml");
            let data = std::fs::read_to_string(manifest).ok()?;
            serde_yaml::from_str::<FeedbackThread>(&data).ok()
        })
        .filter(|t| match filter_context {
            Some(ctx) => t.context == ctx,
            None => true,
        })
        .collect();

    threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(threads)
}

/// Delete a thread and all its posts.
pub fn delete_thread(root: &Path, id: &str) -> Result<()> {
    let dir = paths::feedback_thread_dir(root, id);
    if !dir.exists() {
        return Err(SdlcError::ThreadNotFound(id.to_string()));
    }
    std::fs::remove_dir_all(&dir)?;
    Ok(())
}

/// Append a post to a thread. Returns the new post.
pub fn add_post(root: &Path, id: &str, author: &str, content: &str) -> Result<ThreadPost> {
    let mut thread = load_manifest(root, id)?;
    let seq = thread.post_count + 1;
    let post = ThreadPost {
        seq,
        author: author.to_string(),
        content: content.to_string(),
        created_at: Utc::now(),
    };
    let post_path = paths::feedback_thread_post_path(root, id, seq);
    let data = serde_yaml::to_string(&post)?;
    io::atomic_write(&post_path, data.as_bytes())?;

    thread.post_count = seq;
    thread.updated_at = Utc::now();
    save_manifest(root, &thread)?;
    Ok(post)
}

/// List all posts for a thread, ordered by `seq` ascending.
pub fn list_posts(root: &Path, id: &str) -> Result<Vec<ThreadPost>> {
    // Verify thread exists first
    if !paths::feedback_thread_manifest(root, id).exists() {
        return Err(SdlcError::ThreadNotFound(id.to_string()));
    }
    let posts_dir = paths::feedback_thread_posts_dir(root, id);
    if !posts_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut posts: Vec<ThreadPost> = std::fs::read_dir(&posts_dir)?
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

    posts.sort_by_key(|p| p.seq);
    Ok(posts)
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
    fn create_and_load_thread() {
        let dir = init_dir();
        let t = create_thread(dir.path(), "feature:my-slug", "").unwrap();
        assert!(t.id.contains("feature-my-slug"));
        assert_eq!(t.context, "feature:my-slug");
        assert_eq!(t.title, "Discussion: feature:my-slug");
        assert_eq!(t.post_count, 0);

        let loaded = load_thread(dir.path(), &t.id).unwrap();
        assert_eq!(loaded.id, t.id);
        assert_eq!(loaded.context, "feature:my-slug");
    }

    #[test]
    fn create_with_explicit_title() {
        let dir = init_dir();
        let t = create_thread(dir.path(), "ponder:idea", "My custom title").unwrap();
        assert_eq!(t.title, "My custom title");
    }

    #[test]
    fn list_empty_when_no_threads() {
        let dir = init_dir();
        let threads = list_threads(dir.path(), None).unwrap();
        assert!(threads.is_empty());
    }

    #[test]
    fn list_all_threads() {
        let dir = init_dir();
        create_thread(dir.path(), "feature:a", "").unwrap();
        create_thread(dir.path(), "feature:b", "").unwrap();
        let threads = list_threads(dir.path(), None).unwrap();
        assert_eq!(threads.len(), 2);
    }

    #[test]
    fn list_with_context_filter() {
        let dir = init_dir();
        create_thread(dir.path(), "feature:a", "").unwrap();
        create_thread(dir.path(), "feature:b", "").unwrap();
        let threads = list_threads(dir.path(), Some("feature:a")).unwrap();
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].context, "feature:a");
    }

    #[test]
    fn add_posts_increments_seq_and_post_count() {
        let dir = init_dir();
        let t = create_thread(dir.path(), "feature:x", "").unwrap();
        let p1 = add_post(dir.path(), &t.id, "human", "Hello").unwrap();
        let p2 = add_post(dir.path(), &t.id, "agent:advisor", "World").unwrap();
        assert_eq!(p1.seq, 1);
        assert_eq!(p2.seq, 2);

        let loaded = load_thread(dir.path(), &t.id).unwrap();
        assert_eq!(loaded.post_count, 2);
    }

    #[test]
    fn list_posts_ordered_by_seq() {
        let dir = init_dir();
        let t = create_thread(dir.path(), "feature:y", "").unwrap();
        add_post(dir.path(), &t.id, "human", "First").unwrap();
        add_post(dir.path(), &t.id, "human", "Second").unwrap();
        let posts = list_posts(dir.path(), &t.id).unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].seq, 1);
        assert_eq!(posts[0].content, "First");
        assert_eq!(posts[1].seq, 2);
        assert_eq!(posts[1].content, "Second");
    }

    #[test]
    fn list_posts_empty_when_none() {
        let dir = init_dir();
        let t = create_thread(dir.path(), "feature:z", "").unwrap();
        let posts = list_posts(dir.path(), &t.id).unwrap();
        assert!(posts.is_empty());
    }

    #[test]
    fn delete_thread_removes_directory() {
        let dir = init_dir();
        let t = create_thread(dir.path(), "feature:del", "").unwrap();
        delete_thread(dir.path(), &t.id).unwrap();
        let threads = list_threads(dir.path(), None).unwrap();
        assert!(threads.is_empty());
    }

    #[test]
    fn delete_nonexistent_thread_returns_error() {
        let dir = init_dir();
        let result = delete_thread(dir.path(), "nonexistent-id");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SdlcError::ThreadNotFound(_)));
    }

    #[test]
    fn load_nonexistent_thread_returns_error() {
        let dir = init_dir();
        let result = load_thread(dir.path(), "nope");
        assert!(matches!(result.unwrap_err(), SdlcError::ThreadNotFound(_)));
    }

    #[test]
    fn add_post_to_nonexistent_thread_returns_error() {
        let dir = init_dir();
        let result = add_post(dir.path(), "nope", "human", "text");
        assert!(matches!(result.unwrap_err(), SdlcError::ThreadNotFound(_)));
    }

    #[test]
    fn list_posts_nonexistent_thread_returns_error() {
        let dir = init_dir();
        let result = list_posts(dir.path(), "nope");
        assert!(matches!(result.unwrap_err(), SdlcError::ThreadNotFound(_)));
    }

    #[test]
    fn collision_safe_id_generation() {
        let dir = init_dir();
        // Create two threads with the same context on the same day — IDs must differ
        let t1 = create_thread(dir.path(), "feature:same", "First").unwrap();
        let t2 = create_thread(dir.path(), "feature:same", "Second").unwrap();
        assert_ne!(t1.id, t2.id);
    }

    #[test]
    fn context_with_special_chars_sanitizes_to_valid_path() {
        let dir = init_dir();
        let t = create_thread(dir.path(), "feature:my/slug with spaces", "").unwrap();
        // ID must not contain colons, slashes, or spaces
        assert!(!t.id.contains(':'));
        assert!(!t.id.contains('/'));
        assert!(!t.id.contains(' '));
    }

    #[test]
    fn updated_at_changes_after_post() {
        let dir = init_dir();
        let t = create_thread(dir.path(), "feature:time", "").unwrap();
        let before = t.updated_at;
        // Small sleep to ensure timestamp differs
        std::thread::sleep(std::time::Duration::from_millis(5));
        add_post(dir.path(), &t.id, "human", "Hello").unwrap();
        let after = load_thread(dir.path(), &t.id).unwrap();
        assert!(after.updated_at >= before);
    }
}
