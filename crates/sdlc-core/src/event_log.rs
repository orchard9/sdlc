//! Append-only changelog event log.
//!
//! Events are written to `.sdlc/changelog.yaml` as a YAML sequence.
//! `query_events()` reads the file, optionally filters by `since` timestamp,
//! and caps results at `limit`.
//!
//! The file format is a flat YAML list:
//! ```yaml
//! - id: "ev-0001"
//!   kind: feature_merged
//!   slug: my-feature
//!   timestamp: "2026-03-02T23:00:00Z"
//!   metadata: {}
//! ```

use crate::{error::Result, io};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// EventKind
// ---------------------------------------------------------------------------

/// The kind of changelog event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    FeatureMerged,
    RunFailed,
    MilestoneWaveCompleted,
    FeaturePhaseAdvanced,
    ReviewApproved,
    AuditApproved,
    QaApproved,
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventKind::FeatureMerged => write!(f, "feature_merged"),
            EventKind::RunFailed => write!(f, "run_failed"),
            EventKind::MilestoneWaveCompleted => write!(f, "milestone_wave_completed"),
            EventKind::FeaturePhaseAdvanced => write!(f, "feature_phase_advanced"),
            EventKind::ReviewApproved => write!(f, "review_approved"),
            EventKind::AuditApproved => write!(f, "audit_approved"),
            EventKind::QaApproved => write!(f, "qa_approved"),
        }
    }
}

// ---------------------------------------------------------------------------
// ChangeEvent
// ---------------------------------------------------------------------------

/// A single changelog event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    /// Sequential ID: ev-0001, ev-0002, …
    pub id: String,

    /// The kind of event.
    pub kind: EventKind,

    /// Feature or milestone slug associated with this event.
    /// Optional — some events (e.g. milestone_wave_completed) carry a slug
    /// at the milestone level rather than a feature level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// UTC timestamp of the event.
    pub timestamp: DateTime<Utc>,

    /// Arbitrary key-value metadata for the event.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn changelog_path(root: &Path) -> std::path::PathBuf {
    root.join(".sdlc").join("changelog.yaml")
}

fn load_events(root: &Path) -> Result<Vec<ChangeEvent>> {
    let path = changelog_path(root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let events: Vec<ChangeEvent> = serde_yaml::from_str(&content)?;
    Ok(events)
}

fn next_id(events: &[ChangeEvent]) -> String {
    let n = events.len() + 1;
    format!("ev-{n:04}")
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Append a new event to `.sdlc/changelog.yaml`.
///
/// This function performs a read-then-write (not a true append) to maintain
/// valid YAML sequence format. The file is always replaced atomically.
pub fn append_event(
    root: &Path,
    kind: EventKind,
    slug: Option<String>,
    metadata: serde_json::Value,
) -> Result<ChangeEvent> {
    let mut events = load_events(root)?;
    let event = ChangeEvent {
        id: next_id(&events),
        kind,
        slug,
        timestamp: Utc::now(),
        metadata,
    };
    events.push(event.clone());
    let yaml = serde_yaml::to_string(&events)?;
    io::atomic_write(&changelog_path(root), yaml.as_bytes())?;
    Ok(event)
}

/// Query changelog events, optionally filtered by `since` and capped at `limit`.
///
/// Returns events in chronological order (ascending by timestamp).
/// When `changelog.yaml` does not exist, returns an empty vec.
pub fn query_events(
    root: &Path,
    since: Option<DateTime<Utc>>,
    limit: usize,
) -> Result<Vec<ChangeEvent>> {
    let events = load_events(root)?;
    let filtered: Vec<ChangeEvent> = events
        .into_iter()
        .filter(|e| since.is_none_or(|s| e.timestamp >= s))
        .take(limit)
        .collect();
    Ok(filtered)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_root() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        dir
    }

    #[test]
    fn empty_when_no_file() {
        let dir = make_root();
        let events = query_events(dir.path(), None, 100).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn append_and_query_round_trip() {
        let dir = make_root();
        append_event(
            dir.path(),
            EventKind::FeatureMerged,
            Some("my-feature".into()),
            serde_json::json!({}),
        )
        .unwrap();
        let events = query_events(dir.path(), None, 100).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::FeatureMerged);
        assert_eq!(events[0].slug.as_deref(), Some("my-feature"));
    }

    #[test]
    fn limit_caps_results() {
        let dir = make_root();
        for _ in 0..5 {
            append_event(
                dir.path(),
                EventKind::RunFailed,
                None,
                serde_json::json!({}),
            )
            .unwrap();
        }
        let events = query_events(dir.path(), None, 3).unwrap();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn since_filter_excludes_old_events() {
        let dir = make_root();
        // Append one event, then set `since` to after it
        append_event(
            dir.path(),
            EventKind::FeatureMerged,
            Some("old".into()),
            serde_json::json!({}),
        )
        .unwrap();
        let future: DateTime<Utc> = "2099-01-01T00:00:00Z".parse().unwrap();
        let events = query_events(dir.path(), Some(future), 100).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn since_filter_includes_matching_events() {
        let dir = make_root();
        let past: DateTime<Utc> = "2000-01-01T00:00:00Z".parse().unwrap();
        append_event(
            dir.path(),
            EventKind::ReviewApproved,
            Some("feat".into()),
            serde_json::json!({}),
        )
        .unwrap();
        let events = query_events(dir.path(), Some(past), 100).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn ids_are_sequential() {
        let dir = make_root();
        let e1 = append_event(
            dir.path(),
            EventKind::FeatureMerged,
            None,
            serde_json::json!({}),
        )
        .unwrap();
        let e2 = append_event(
            dir.path(),
            EventKind::RunFailed,
            None,
            serde_json::json!({}),
        )
        .unwrap();
        assert_eq!(e1.id, "ev-0001");
        assert_eq!(e2.id, "ev-0002");
    }
}
