use crate::error::{Result, SdlcError};
use crate::paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

// ---------------------------------------------------------------------------
// MilestoneStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    Active,
    Complete,
    Cancelled,
}

impl fmt::Display for MilestoneStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MilestoneStatus::Active => "active",
            MilestoneStatus::Complete => "complete",
            MilestoneStatus::Cancelled => "cancelled",
        };
        f.write_str(s)
    }
}

// ---------------------------------------------------------------------------
// Milestone
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub features: Vec<String>,
    pub status: MilestoneStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

impl Milestone {
    pub fn new(slug: impl Into<String>, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            slug: slug.into(),
            title: title.into(),
            description: None,
            features: Vec::new(),
            status: MilestoneStatus::Active,
            created_at: now,
            updated_at: now,
            completed_at: None,
            cancelled_at: None,
        }
    }

    // ---------------------------------------------------------------------------
    // Persistence
    // ---------------------------------------------------------------------------

    pub fn create(root: &Path, slug: impl Into<String>, title: impl Into<String>) -> Result<Self> {
        let slug = slug.into();
        paths::validate_slug(&slug)?;

        let dir = paths::milestone_dir(root, &slug);
        if dir.exists() {
            return Err(SdlcError::MilestoneExists(slug));
        }

        let milestone = Self::new(slug, title);
        milestone.save(root)?;
        Ok(milestone)
    }

    pub fn load(root: &Path, slug: &str) -> Result<Self> {
        let manifest = paths::milestone_manifest(root, slug);
        if !manifest.exists() {
            return Err(SdlcError::MilestoneNotFound(slug.to_string()));
        }
        let data = std::fs::read_to_string(&manifest)?;
        let milestone: Milestone = serde_yaml::from_str(&data)?;
        Ok(milestone)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let manifest = paths::milestone_manifest(root, &self.slug);
        let data = serde_yaml::to_string(self)?;
        crate::io::atomic_write(&manifest, data.as_bytes())
    }

    pub fn list(root: &Path) -> Result<Vec<Self>> {
        let milestones_dir = root.join(paths::MILESTONES_DIR);
        if !milestones_dir.exists() {
            return Ok(Vec::new());
        }

        let mut milestones = Vec::new();
        for entry in std::fs::read_dir(&milestones_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let slug = entry.file_name().to_string_lossy().into_owned();
                match Self::load(root, &slug) {
                    Ok(m) => milestones.push(m),
                    Err(SdlcError::MilestoneNotFound(_)) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        milestones.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(milestones)
    }

    // ---------------------------------------------------------------------------
    // Mutations
    // ---------------------------------------------------------------------------

    /// Add a feature slug. Returns `false` if already present (idempotent).
    pub fn add_feature(&mut self, feature_slug: &str) -> bool {
        if self.features.contains(&feature_slug.to_string()) {
            return false;
        }
        self.features.push(feature_slug.to_string());
        self.updated_at = Utc::now();
        true
    }

    /// Insert a feature slug at `pos` (0-based, clamped). Returns `false` if already present.
    pub fn add_feature_at(&mut self, feature_slug: &str, pos: usize) -> bool {
        if self.features.contains(&feature_slug.to_string()) {
            return false;
        }
        let insert_at = pos.min(self.features.len());
        self.features.insert(insert_at, feature_slug.to_string());
        self.updated_at = Utc::now();
        true
    }

    /// Remove a feature slug. Returns `false` if not present.
    pub fn remove_feature(&mut self, feature_slug: &str) -> bool {
        let before = self.features.len();
        self.features.retain(|s| s != feature_slug);
        if self.features.len() < before {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn complete(&mut self) {
        self.status = MilestoneStatus::Complete;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn cancel(&mut self) {
        self.status = MilestoneStatus::Cancelled;
        self.cancelled_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn update_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = Utc::now();
    }

    /// Replace the feature order with `ordered`. Every slug currently in
    /// `self.features` must appear exactly once in `ordered`.
    pub fn reorder_features(&mut self, ordered: &[&str]) -> Result<()> {
        // Check for duplicates in the input list
        let mut seen = std::collections::HashSet::new();
        for &s in ordered {
            if !seen.insert(s) {
                return Err(SdlcError::InvalidFeatureOrder(format!(
                    "duplicate slug in order list: '{s}'"
                )));
            }
        }

        // Build a set of existing features for O(n) lookups
        let existing: std::collections::HashSet<&str> =
            self.features.iter().map(|s| s.as_str()).collect();

        // Check for slugs in ordered that are not in self.features
        for &s in ordered {
            if !existing.contains(s) {
                return Err(SdlcError::InvalidFeatureOrder(format!(
                    "'{s}' is not in this milestone"
                )));
            }
        }

        // Check for slugs in self.features that are missing from ordered
        for f in &self.features {
            if !seen.contains(f.as_str()) {
                return Err(SdlcError::InvalidFeatureOrder(format!(
                    "missing slug in order list: '{f}'"
                )));
            }
        }

        self.features = ordered.iter().map(|s| s.to_string()).collect();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Move `slug` to `to_index` (0-based). Clamps to valid range.
    pub fn move_feature(&mut self, slug: &str, to_index: usize) -> Result<()> {
        let from = self
            .features
            .iter()
            .position(|s| s == slug)
            .ok_or_else(|| SdlcError::FeatureNotFound(slug.to_string()))?;

        let last = self.features.len() - 1;
        let to = to_index.min(last);

        let item = self.features.remove(from);
        self.features.insert(to, item);
        self.updated_at = Utc::now();
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup(dir: &TempDir) {
        std::fs::create_dir_all(dir.path().join(".sdlc/milestones")).unwrap();
    }

    #[test]
    fn milestone_create_load() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let m = Milestone::create(dir.path(), "v2-launch", "v2.0 Launch").unwrap();
        assert_eq!(m.slug, "v2-launch");
        assert_eq!(m.status, MilestoneStatus::Active);
        assert!(m.features.is_empty());

        let loaded = Milestone::load(dir.path(), "v2-launch").unwrap();
        assert_eq!(loaded.title, "v2.0 Launch");
        assert!(loaded.completed_at.is_none());
    }

    #[test]
    fn milestone_duplicate_fails() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        Milestone::create(dir.path(), "v2", "v2").unwrap();
        assert!(matches!(
            Milestone::create(dir.path(), "v2", "v2 again"),
            Err(SdlcError::MilestoneExists(_))
        ));
    }

    #[test]
    fn milestone_add_remove_feature() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();

        assert!(m.add_feature("auth"));
        assert!(!m.add_feature("auth")); // idempotent
        assert_eq!(m.features.len(), 1);

        assert!(m.remove_feature("auth"));
        assert!(!m.remove_feature("auth")); // already gone
        assert!(m.features.is_empty());
    }

    #[test]
    fn reorder_basic() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.add_feature("b");
        m.add_feature("c");

        m.reorder_features(&["c", "a", "b"]).unwrap();
        assert_eq!(m.features, vec!["c", "a", "b"]);
    }

    #[test]
    fn reorder_rejects_missing_slug() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.add_feature("b");

        let err = m.reorder_features(&["a"]).unwrap_err();
        assert!(err.to_string().contains("missing slug in order list: 'b'"));
    }

    #[test]
    fn reorder_rejects_extra_slug() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");

        let err = m.reorder_features(&["a", "ghost"]).unwrap_err();
        assert!(err.to_string().contains("'ghost' is not in this milestone"));
    }

    #[test]
    fn reorder_rejects_duplicate() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.add_feature("b");

        let err = m.reorder_features(&["a", "a"]).unwrap_err();
        assert!(err
            .to_string()
            .contains("duplicate slug in order list: 'a'"));
    }

    #[test]
    fn reorder_yaml_round_trip() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("x");
        m.add_feature("y");
        m.add_feature("z");
        m.reorder_features(&["z", "x", "y"]).unwrap();
        m.save(dir.path()).unwrap();

        let loaded = Milestone::load(dir.path(), "v2").unwrap();
        assert_eq!(loaded.features, vec!["z", "x", "y"]);
    }

    #[test]
    fn move_feature_forward() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.add_feature("b");
        m.add_feature("c");

        m.move_feature("a", 2).unwrap();
        assert_eq!(m.features, vec!["b", "c", "a"]);
    }

    #[test]
    fn move_feature_backward() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.add_feature("b");
        m.add_feature("c");

        m.move_feature("c", 0).unwrap();
        assert_eq!(m.features, vec!["c", "a", "b"]);
    }

    #[test]
    fn move_feature_clamps_to_last() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.add_feature("b");
        m.add_feature("c");

        // to_index beyond end → clamp to 2
        m.move_feature("a", 99).unwrap();
        assert_eq!(m.features, vec!["b", "c", "a"]);
    }

    #[test]
    fn move_feature_not_found() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");

        assert!(matches!(
            m.move_feature("ghost", 0),
            Err(SdlcError::FeatureNotFound(_))
        ));
    }

    #[test]
    fn milestone_complete_cancel() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.complete();
        assert_eq!(m.status, MilestoneStatus::Complete);
        assert!(m.completed_at.is_some());

        let mut m2 = Milestone::create(dir.path(), "v3", "v3").unwrap();
        m2.cancel();
        assert_eq!(m2.status, MilestoneStatus::Cancelled);
        assert!(m2.cancelled_at.is_some());
    }
}
