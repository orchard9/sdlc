//! Project-level backlog — a parking lot for out-of-scope concerns discovered during
//! autonomous agent runs.
//!
//! Agents call `sdlc backlog add` at the moment of discovery (not deferred to session
//! end) to capture cross-feature concerns with no other natural home. Items are
//! classified by kind, can be parked with a reason, or promoted to features.
//!
//! Storage: `.sdlc/backlog.yaml` — flat list, append-only IDs.
//! IDs: B1, B2, B3... — sequential, never recycled.

use crate::{error::Result, io, paths, SdlcError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Classification of a backlog item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacklogKind {
    /// Bug risk, race condition, security concern, correctness issue.
    Concern,
    /// Future enhancement or refactor opportunity.
    Idea,
    /// Known technical debt or structural accretion.
    Debt,
}

impl std::fmt::Display for BacklogKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BacklogKind::Concern => f.write_str("concern"),
            BacklogKind::Idea => f.write_str("idea"),
            BacklogKind::Debt => f.write_str("debt"),
        }
    }
}

/// Lifecycle status of a backlog item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacklogStatus {
    /// Newly captured; awaiting triage.
    Open,
    /// Set aside with a reason; not urgent.
    Parked,
    /// Promoted to a tracked feature.
    Promoted,
}

impl std::fmt::Display for BacklogStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BacklogStatus::Open => f.write_str("open"),
            BacklogStatus::Parked => f.write_str("parked"),
            BacklogStatus::Promoted => f.write_str("promoted"),
        }
    }
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single captured concern, idea, or debt item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogItem {
    /// Sequential B-prefixed ID: B1, B2, B3…  Never recycled.
    pub id: String,

    /// Required. A complete sentence describing the concern with a component reference.
    /// Example: "AuthMiddleware in auth.rs: token validation has a race under concurrent requests."
    pub title: String,

    /// Classification: concern, idea, or debt.
    pub kind: BacklogKind,

    /// Lifecycle status.
    pub status: BacklogStatus,

    /// Optional multi-line context explaining the concern in detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional short grounding reference: file path, function name, or failing test.
    /// Distinguishes execution-grounded concerns from speculative ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,

    /// Slug of the feature that was running when this item was discovered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_feature: Option<String>,

    /// Required when status is Parked. Explains why the item was de-prioritized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub park_reason: Option<String>,

    /// Slug of the feature this item was promoted to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_to: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The top-level store; wraps the flat list in backlog.yaml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BacklogStore {
    #[serde(default)]
    pub items: Vec<BacklogItem>,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl BacklogStore {
    /// Load `.sdlc/backlog.yaml`. Returns an empty default if the file is absent.
    pub fn load(root: &Path) -> Result<Self> {
        let path = paths::backlog_path(root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(&path)?;
        let store: Self = serde_yaml::from_str(&data)?;
        Ok(store)
    }

    /// Atomically write `.sdlc/backlog.yaml`.
    pub fn save(&self, root: &Path) -> Result<()> {
        let path = paths::backlog_path(root);
        let data = serde_yaml::to_string(self)?;
        io::atomic_write(&path, data.as_bytes())
    }

    /// Add a new item. Returns the created item with its assigned ID.
    pub fn add(
        root: &Path,
        title: String,
        kind: BacklogKind,
        description: Option<String>,
        evidence: Option<String>,
        source_feature: Option<String>,
    ) -> Result<BacklogItem> {
        let mut store = Self::load(root)?;
        let now = Utc::now();
        let id = Self::next_id(&store.items);
        let item = BacklogItem {
            id,
            title,
            kind,
            status: BacklogStatus::Open,
            description,
            evidence,
            source_feature,
            park_reason: None,
            promoted_to: None,
            created_at: now,
            updated_at: now,
        };
        store.items.push(item.clone());
        store.save(root)?;
        Ok(item)
    }

    /// List items, with optional status and source_feature filters (AND-composed).
    /// Results are sorted by `created_at` ascending.
    pub fn list(
        root: &Path,
        status_filter: Option<BacklogStatus>,
        source_feature: Option<&str>,
    ) -> Result<Vec<BacklogItem>> {
        let store = Self::load(root)?;
        let items = store
            .items
            .into_iter()
            .filter(|item| {
                let status_ok = status_filter.map(|s| item.status == s).unwrap_or(true);
                let feature_ok = source_feature
                    .map(|f| item.source_feature.as_deref() == Some(f))
                    .unwrap_or(true);
                status_ok && feature_ok
            })
            .collect();
        Ok(items)
    }

    /// Get a single item by ID. Returns `BacklogItemNotFound` if absent.
    pub fn get(root: &Path, id: &str) -> Result<BacklogItem> {
        let store = Self::load(root)?;
        store
            .items
            .into_iter()
            .find(|item| item.id == id)
            .ok_or_else(|| SdlcError::BacklogItemNotFound(id.to_string()))
    }

    /// Park an item. Requires a non-empty `park_reason`.
    /// Returns an error if the item is already Promoted.
    pub fn park(root: &Path, id: &str, park_reason: String) -> Result<BacklogItem> {
        if park_reason.trim().is_empty() {
            return Err(SdlcError::InvalidTransition {
                from: "open".to_string(),
                to: "parked".to_string(),
                reason:
                    "park_reason must not be empty — explain why this item is being de-prioritized"
                        .to_string(),
            });
        }
        let mut store = Self::load(root)?;
        let item = store
            .items
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| SdlcError::BacklogItemNotFound(id.to_string()))?;
        if item.status == BacklogStatus::Promoted {
            return Err(SdlcError::InvalidTransition {
                from: "promoted".to_string(),
                to: "parked".to_string(),
                reason: "cannot park a promoted item; it has already been actioned".to_string(),
            });
        }
        item.status = BacklogStatus::Parked;
        item.park_reason = Some(park_reason);
        item.updated_at = Utc::now();
        let updated = item.clone();
        store.save(root)?;
        Ok(updated)
    }

    /// Record that an item was promoted to a feature.
    /// The feature must be created by the caller (CLI layer) before calling this.
    /// Parked items can be promoted (intent overrides prior parking).
    pub fn mark_promoted(root: &Path, id: &str, feature_slug: &str) -> Result<BacklogItem> {
        let mut store = Self::load(root)?;
        let item = store
            .items
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| SdlcError::BacklogItemNotFound(id.to_string()))?;
        if item.status == BacklogStatus::Promoted {
            return Err(SdlcError::InvalidTransition {
                from: "promoted".to_string(),
                to: "promoted".to_string(),
                reason: format!(
                    "item {} is already promoted to '{}'",
                    id,
                    item.promoted_to.as_deref().unwrap_or("unknown")
                ),
            });
        }
        item.status = BacklogStatus::Promoted;
        item.promoted_to = Some(feature_slug.to_string());
        item.updated_at = Utc::now();
        let updated = item.clone();
        store.save(root)?;
        Ok(updated)
    }

    /// Generate the next sequential B-prefixed ID.
    fn next_id(items: &[BacklogItem]) -> String {
        let max = items
            .iter()
            .filter_map(|i| i.id.strip_prefix('B').and_then(|n| n.parse::<u32>().ok()))
            .max()
            .unwrap_or(0);
        format!("B{}", max + 1)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn sdlc_dir(dir: &TempDir) -> std::path::PathBuf {
        let sdlc = dir.path().join(".sdlc");
        std::fs::create_dir_all(&sdlc).unwrap();
        dir.path().to_path_buf()
    }

    #[test]
    fn add_creates_item_with_b1_id() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        let item = BacklogStore::add(
            &root,
            "auth.rs: token race under concurrent requests".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(item.id, "B1");
        assert_eq!(item.status, BacklogStatus::Open);
        assert_eq!(item.kind, BacklogKind::Concern);
    }

    #[test]
    fn add_sequential_ids() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        let first = BacklogStore::add(
            &root,
            "First concern".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        let second = BacklogStore::add(
            &root,
            "Second concern".to_string(),
            BacklogKind::Idea,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(first.id, "B1");
        assert_eq!(second.id, "B2");
    }

    #[test]
    fn add_persists_all_fields() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        let item = BacklogStore::add(
            &root,
            "redb.rs: compaction not configured".to_string(),
            BacklogKind::Debt,
            Some("may grow unbounded under heavy write load".to_string()),
            Some("crates/sdlc-core/src/orchestrator.rs:42".to_string()),
            Some("run-events-api".to_string()),
        )
        .unwrap();
        assert_eq!(item.kind, BacklogKind::Debt);
        assert_eq!(
            item.description.as_deref(),
            Some("may grow unbounded under heavy write load")
        );
        assert_eq!(
            item.evidence.as_deref(),
            Some("crates/sdlc-core/src/orchestrator.rs:42")
        );
        assert_eq!(item.source_feature.as_deref(), Some("run-events-api"));
    }

    #[test]
    fn list_unfiltered_returns_all() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        BacklogStore::add(&root, "B".to_string(), BacklogKind::Idea, None, None, None).unwrap();
        let items = BacklogStore::list(&root, None, None).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn list_open_status_filter() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        BacklogStore::add(&root, "B".to_string(), BacklogKind::Idea, None, None, None).unwrap();
        BacklogStore::park(&root, "B1", "not urgent".to_string()).unwrap();
        let open = BacklogStore::list(&root, Some(BacklogStatus::Open), None).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, "B2");
    }

    #[test]
    fn list_by_source_feature() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            Some("feature-x".to_string()),
        )
        .unwrap();
        BacklogStore::add(
            &root,
            "B".to_string(),
            BacklogKind::Concern,
            None,
            None,
            Some("feature-y".to_string()),
        )
        .unwrap();
        let items = BacklogStore::list(&root, None, Some("feature-x")).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].source_feature.as_deref(), Some("feature-x"));
    }

    #[test]
    fn list_combined_filters() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        // open + feature-x
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            Some("feature-x".to_string()),
        )
        .unwrap();
        // parked + feature-x
        BacklogStore::add(
            &root,
            "B".to_string(),
            BacklogKind::Debt,
            None,
            None,
            Some("feature-x".to_string()),
        )
        .unwrap();
        BacklogStore::park(&root, "B2", "low priority".to_string()).unwrap();
        // open + feature-y
        BacklogStore::add(
            &root,
            "C".to_string(),
            BacklogKind::Idea,
            None,
            None,
            Some("feature-y".to_string()),
        )
        .unwrap();
        let items =
            BacklogStore::list(&root, Some(BacklogStatus::Open), Some("feature-x")).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "B1");
    }

    #[test]
    fn get_existing_returns_item() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        let item = BacklogStore::get(&root, "B1").unwrap();
        assert_eq!(item.id, "B1");
    }

    #[test]
    fn get_missing_id_errors() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        let err = BacklogStore::get(&root, "B99").unwrap_err();
        assert!(matches!(err, SdlcError::BacklogItemNotFound(ref id) if id == "B99"));
    }

    #[test]
    fn park_sets_status_and_reason() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        let parked = BacklogStore::park(&root, "B1", "revisit after v14".to_string()).unwrap();
        assert_eq!(parked.status, BacklogStatus::Parked);
        assert_eq!(parked.park_reason.as_deref(), Some("revisit after v14"));
        // Verify persisted
        let loaded = BacklogStore::get(&root, "B1").unwrap();
        assert_eq!(loaded.status, BacklogStatus::Parked);
    }

    #[test]
    fn park_requires_nonempty_reason() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        let err = BacklogStore::park(&root, "B1", "".to_string()).unwrap_err();
        assert!(matches!(err, SdlcError::InvalidTransition { .. }));
    }

    #[test]
    fn park_whitespace_only_reason_errors() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        let err = BacklogStore::park(&root, "B1", "   ".to_string()).unwrap_err();
        assert!(matches!(err, SdlcError::InvalidTransition { .. }));
    }

    #[test]
    fn park_promoted_item_errors() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        BacklogStore::mark_promoted(&root, "B1", "my-feature").unwrap();
        let err = BacklogStore::park(&root, "B1", "too late".to_string()).unwrap_err();
        assert!(matches!(err, SdlcError::InvalidTransition { .. }));
    }

    #[test]
    fn mark_promoted_sets_slug() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        let promoted = BacklogStore::mark_promoted(&root, "B1", "auth-race-fix").unwrap();
        assert_eq!(promoted.status, BacklogStatus::Promoted);
        assert_eq!(promoted.promoted_to.as_deref(), Some("auth-race-fix"));
    }

    #[test]
    fn mark_promoted_from_parked_ok() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        BacklogStore::park(&root, "B1", "parked initially".to_string()).unwrap();
        let promoted = BacklogStore::mark_promoted(&root, "B1", "auth-race-fix").unwrap();
        assert_eq!(promoted.status, BacklogStatus::Promoted);
    }

    #[test]
    fn mark_promoted_already_promoted_errors() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "A".to_string(),
            BacklogKind::Concern,
            None,
            None,
            None,
        )
        .unwrap();
        BacklogStore::mark_promoted(&root, "B1", "feature-a").unwrap();
        let err = BacklogStore::mark_promoted(&root, "B1", "feature-b").unwrap_err();
        assert!(matches!(err, SdlcError::InvalidTransition { .. }));
    }

    #[test]
    fn round_trip_serialization() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        BacklogStore::add(
            &root,
            "Round-trip test".to_string(),
            BacklogKind::Idea,
            Some("detail".to_string()),
            Some("src/lib.rs:10".to_string()),
            Some("some-feature".to_string()),
        )
        .unwrap();
        let store = BacklogStore::load(&root).unwrap();
        assert_eq!(store.items.len(), 1);
        let item = &store.items[0];
        assert_eq!(item.id, "B1");
        assert_eq!(item.kind, BacklogKind::Idea);
        assert_eq!(item.description.as_deref(), Some("detail"));
        assert_eq!(item.evidence.as_deref(), Some("src/lib.rs:10"));
        assert_eq!(item.source_feature.as_deref(), Some("some-feature"));
        assert!(item.park_reason.is_none());
        assert!(item.promoted_to.is_none());
    }

    #[test]
    fn load_absent_file_returns_empty() {
        let dir = tmp();
        let root = sdlc_dir(&dir);
        let store = BacklogStore::load(&root).unwrap();
        assert!(store.items.is_empty());
    }
}
