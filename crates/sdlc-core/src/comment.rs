use crate::types::ArtifactType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// CommentFlag
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentFlag {
    Blocker,
    Question,
    Decision,
    Fyi,
}

impl fmt::Display for CommentFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CommentFlag::Blocker => "blocker",
            CommentFlag::Question => "question",
            CommentFlag::Decision => "decision",
            CommentFlag::Fyi => "fyi",
        };
        f.write_str(s)
    }
}

// ---------------------------------------------------------------------------
// CommentTarget
// ---------------------------------------------------------------------------

/// What entity a comment is attached to within a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CommentTarget {
    Feature,
    Task { task_id: String },
    Artifact { artifact_type: ArtifactType },
}

impl fmt::Display for CommentTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommentTarget::Feature => f.write_str("feature"),
            CommentTarget::Task { task_id } => write!(f, "task:{}", task_id),
            CommentTarget::Artifact { artifact_type } => {
                write!(f, "artifact:{}", artifact_type)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Comment
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author: Option<String>,
    pub body: String,
    pub flag: Option<CommentFlag>,
    pub target: CommentTarget,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------

/// Append a comment to the list and return its auto-generated ID.
///
/// `seq` is a monotonic counter stored on the owning `Feature`. Incrementing it
/// before generating the ID ensures IDs are unique even after comments are resolved
/// (removed), which would otherwise cause a length-based scheme to produce duplicates.
pub fn add_comment(
    comments: &mut Vec<Comment>,
    seq: &mut u32,
    body: impl Into<String>,
    flag: Option<CommentFlag>,
    target: CommentTarget,
    author: Option<String>,
) -> String {
    *seq += 1;
    let id = format!("C{}", *seq);
    comments.push(Comment {
        id: id.clone(),
        author,
        body: body.into(),
        flag,
        target,
        created_at: Utc::now(),
    });
    id
}

/// Remove a comment by ID. Returns `true` if found and removed, `false` if not found.
pub fn resolve_comment(comments: &mut Vec<Comment>, id: &str) -> bool {
    if let Some(pos) = comments.iter().position(|c| c.id == id) {
        comments.remove(pos);
        true
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_comment_increments_id() {
        let mut comments: Vec<Comment> = Vec::new();
        let mut seq: u32 = 0;
        let id1 = add_comment(&mut comments, &mut seq, "first", None, CommentTarget::Feature, None);
        let id2 = add_comment(
            &mut comments,
            &mut seq,
            "second",
            Some(CommentFlag::Blocker),
            CommentTarget::Task { task_id: "T1".to_string() },
            Some("alice".to_string()),
        );
        assert_eq!(id1, "C1");
        assert_eq!(id2, "C2");
        assert_eq!(comments[1].flag, Some(CommentFlag::Blocker));
        assert_eq!(comments[1].author.as_deref(), Some("alice"));
    }

    #[test]
    fn comment_flag_display() {
        assert_eq!(CommentFlag::Blocker.to_string(), "blocker");
        assert_eq!(CommentFlag::Fyi.to_string(), "fyi");
    }

    #[test]
    fn comment_target_display() {
        assert_eq!(CommentTarget::Feature.to_string(), "feature");
        assert_eq!(
            CommentTarget::Task { task_id: "T2".to_string() }.to_string(),
            "task:T2"
        );
    }

    #[test]
    fn resolve_comment_removes_by_id() {
        let mut comments: Vec<Comment> = Vec::new();
        let mut seq: u32 = 0;
        add_comment(
            &mut comments,
            &mut seq,
            "first",
            Some(CommentFlag::Blocker),
            CommentTarget::Feature,
            None,
        );
        add_comment(&mut comments, &mut seq, "second", None, CommentTarget::Feature, None);

        let removed = resolve_comment(&mut comments, "C1");
        assert!(removed);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].id, "C2");

        // Non-existent ID returns false, list unchanged
        let not_found = resolve_comment(&mut comments, "C99");
        assert!(!not_found);
        assert_eq!(comments.len(), 1);
    }

    #[test]
    fn no_id_collision_after_resolve() {
        let mut comments: Vec<Comment> = Vec::new();
        let mut seq: u32 = 0;
        add_comment(&mut comments, &mut seq, "first", None, CommentTarget::Feature, None);  // C1
        add_comment(&mut comments, &mut seq, "second", None, CommentTarget::Feature, None); // C2
        resolve_comment(&mut comments, "C1"); // C2 now the only comment
        let id3 = add_comment(&mut comments, &mut seq, "third", None, CommentTarget::Feature, None);
        assert_eq!(id3, "C3", "ID must not collide with existing C2");
        assert_eq!(comments.len(), 2);
        // Verify C2 is the original "second", not "third"
        assert_eq!(comments[0].body, "second");
        assert_eq!(comments[1].body, "third");
        assert_eq!(comments[1].id, "C3");
    }
}
