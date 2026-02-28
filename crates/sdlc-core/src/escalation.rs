//! Human escalation queue — typed requests that only a human can action.
//!
//! Layout:
//!   .sdlc/escalations.yaml   — list of all escalations (open + resolved)
//!
//! IDs are sequential: E1, E2, E3, …
//! When created with a `source_feature`, a Blocker comment is automatically
//! added to that feature.  When resolved, the linked comment is removed.

use crate::comment::{add_comment, resolve_comment, CommentFlag, CommentTarget};
use crate::error::{Result, SdlcError};
use crate::feature::Feature;
use crate::io;
use crate::paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationKind {
    SecretRequest,
    Question,
    Vision,
    ManualTest,
}

impl std::fmt::Display for EscalationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EscalationKind::SecretRequest => "secret_request",
            EscalationKind::Question => "question",
            EscalationKind::Vision => "vision",
            EscalationKind::ManualTest => "manual_test",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for EscalationKind {
    type Err = SdlcError;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "secret_request" => Ok(EscalationKind::SecretRequest),
            "question" => Ok(EscalationKind::Question),
            "vision" => Ok(EscalationKind::Vision),
            "manual_test" => Ok(EscalationKind::ManualTest),
            _ => Err(SdlcError::InvalidSlug(format!(
                "unknown escalation kind '{s}': must be secret_request, question, vision, or manual_test"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationStatus {
    Open,
    Resolved,
}

impl std::fmt::Display for EscalationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EscalationStatus::Open => f.write_str("open"),
            EscalationStatus::Resolved => f.write_str("resolved"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationItem {
    pub id: String,
    pub kind: EscalationKind,
    pub title: String,
    pub context: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_feature: Option<String>,
    /// Comment ID on the source feature that gates it (None for project-level escalations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_comment_id: Option<String>,
    pub status: EscalationStatus,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

// ---------------------------------------------------------------------------
// Internal file I/O
// ---------------------------------------------------------------------------

fn load_all(root: &Path) -> Result<Vec<EscalationItem>> {
    let path = paths::escalations_path(root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_yaml::from_str(&content)?)
}

fn save_all(root: &Path, items: &[EscalationItem]) -> Result<()> {
    let path = paths::escalations_path(root);
    let content = serde_yaml::to_string(items)?;
    io::atomic_write(&path, content.as_bytes())
}

fn next_id(items: &[EscalationItem]) -> String {
    let n = items.len() + 1;
    format!("E{n}")
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a new escalation.
///
/// If `source_feature` is provided, a Blocker comment is automatically added
/// to that feature so the `wait_for_approval` gate engages.
pub fn create(
    root: &Path,
    kind: EscalationKind,
    title: impl Into<String>,
    context: impl Into<String>,
    source_feature: Option<&str>,
) -> Result<EscalationItem> {
    let title = title.into();
    let context = context.into();

    let mut items = load_all(root)?;
    let id = next_id(&items);

    // If a source feature was given, add a blocker comment to it.
    let linked_comment_id = if let Some(slug) = source_feature {
        let mut feature = Feature::load(root, slug)?;
        let body = format!("[Escalation {id}] {title}");
        let comment_id = add_comment(
            &mut feature.comments,
            &mut feature.next_comment_seq,
            body,
            Some(CommentFlag::Blocker),
            CommentTarget::Feature,
            Some("sdlc".to_string()),
        );
        feature.save(root)?;
        Some(comment_id)
    } else {
        None
    };

    let item = EscalationItem {
        id: id.clone(),
        kind,
        title,
        context,
        source_feature: source_feature.map(str::to_string),
        linked_comment_id,
        status: EscalationStatus::Open,
        created_at: Utc::now(),
        resolved_at: None,
        resolution: None,
    };

    items.push(item.clone());
    save_all(root, &items)?;

    Ok(item)
}

/// List escalations.  Pass `None` for status to get open items only.
/// Pass `Some("all")` to get everything.
pub fn list(root: &Path, status_filter: Option<&str>) -> Result<Vec<EscalationItem>> {
    let items = load_all(root)?;
    let filtered = match status_filter {
        Some("all") => items,
        Some("resolved") => items
            .into_iter()
            .filter(|e| e.status == EscalationStatus::Resolved)
            .collect(),
        _ => items
            .into_iter()
            .filter(|e| e.status == EscalationStatus::Open)
            .collect(),
    };
    Ok(filtered)
}

/// Get a single escalation by ID.
pub fn get(root: &Path, id: &str) -> Result<EscalationItem> {
    let items = load_all(root)?;
    items
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| SdlcError::EscalationNotFound(id.to_string()))
}

/// Resolve an escalation.
///
/// If it has a linked feature comment, that comment is removed so the
/// `wait_for_approval` gate disengages.
pub fn resolve(root: &Path, id: &str, resolution: impl Into<String>) -> Result<EscalationItem> {
    let resolution = resolution.into();
    let mut items = load_all(root)?;

    let pos = items
        .iter()
        .position(|e| e.id == id)
        .ok_or_else(|| SdlcError::EscalationNotFound(id.to_string()))?;

    // Remove the linked blocker comment and add a resolution comment to the source feature.
    if let Some(slug) = items[pos].source_feature.clone() {
        match Feature::load(root, &slug) {
            Ok(mut feature) => {
                // Remove the blocker comment so the wait_for_approval gate disengages.
                if let Some(comment_id) = &items[pos].linked_comment_id {
                    resolve_comment(&mut feature.comments, comment_id);
                }
                // Write the resolution back so the agent knows what happened.
                let esc_id = &items[pos].id;
                let esc_title = &items[pos].title;
                let body = format!("[Escalation {esc_id} resolved] {esc_title}\n\n{resolution}");
                add_comment(
                    &mut feature.comments,
                    &mut feature.next_comment_seq,
                    body,
                    Some(CommentFlag::Fyi),
                    CommentTarget::Feature,
                    Some("human".to_string()),
                );
                feature.save(root)?;
            }
            // Feature might have been deleted; that's fine.
            Err(SdlcError::FeatureNotFound(_)) => {}
            Err(e) => return Err(e),
        }
    }

    items[pos].status = EscalationStatus::Resolved;
    items[pos].resolved_at = Some(Utc::now());
    items[pos].resolution = Some(resolution);
    items[pos].linked_comment_id = None;

    let resolved = items[pos].clone();
    save_all(root, &items)?;

    Ok(resolved)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn init_dir() -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        // Create the .sdlc directory so atomic_write can create the file.
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        dir
    }

    #[test]
    fn create_project_level_escalation() {
        let dir = init_dir();
        let item = create(
            dir.path(),
            EscalationKind::Question,
            "Should we support crypto?",
            "Affects checkout milestone",
            None,
        )
        .unwrap();

        assert_eq!(item.id, "E1");
        assert_eq!(item.kind, EscalationKind::Question);
        assert_eq!(item.status, EscalationStatus::Open);
        assert!(item.source_feature.is_none());
        assert!(item.linked_comment_id.is_none());
    }

    #[test]
    fn sequential_ids() {
        let dir = init_dir();
        let e1 = create(dir.path(), EscalationKind::Vision, "T1", "C1", None).unwrap();
        let e2 = create(dir.path(), EscalationKind::Question, "T2", "C2", None).unwrap();
        assert_eq!(e1.id, "E1");
        assert_eq!(e2.id, "E2");
    }

    #[test]
    fn list_open_by_default() {
        let dir = init_dir();
        create(dir.path(), EscalationKind::Vision, "T1", "C1", None).unwrap();
        create(dir.path(), EscalationKind::Question, "T2", "C2", None).unwrap();

        let open = list(dir.path(), None).unwrap();
        assert_eq!(open.len(), 2);
    }

    #[test]
    fn resolve_removes_from_open() {
        let dir = init_dir();
        create(dir.path(), EscalationKind::Vision, "T1", "C1", None).unwrap();

        resolve(dir.path(), "E1", "Done").unwrap();

        let open = list(dir.path(), None).unwrap();
        assert!(open.is_empty());

        let resolved = list(dir.path(), Some("resolved")).unwrap();
        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].resolution.as_deref() == Some("Done"));
    }

    #[test]
    fn resolve_missing_escalation_returns_not_found() {
        let dir = init_dir();
        let err = resolve(dir.path(), "E99", "irrelevant").unwrap_err();
        assert!(matches!(err, SdlcError::EscalationNotFound(_)));
    }

    #[test]
    fn get_missing_returns_not_found() {
        let dir = init_dir();
        let err = get(dir.path(), "E99").unwrap_err();
        assert!(matches!(err, SdlcError::EscalationNotFound(_)));
    }

    #[test]
    fn list_all_status_filter() {
        let dir = init_dir();
        create(dir.path(), EscalationKind::Vision, "T1", "C1", None).unwrap();
        create(dir.path(), EscalationKind::Question, "T2", "C2", None).unwrap();
        resolve(dir.path(), "E1", "Done").unwrap();

        let all = list(dir.path(), Some("all")).unwrap();
        assert_eq!(all.len(), 2);
    }
}
