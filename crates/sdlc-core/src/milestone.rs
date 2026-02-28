use crate::error::{Result, SdlcError};
use crate::feature::Feature;
use crate::paths;
use crate::types::Phase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

// ---------------------------------------------------------------------------
// MilestoneStatus
// ---------------------------------------------------------------------------

/// Derived from feature phases at read time. Only `Skipped` and `Released`
/// (via `released_at`) are stored explicitly. `Active`, `Verifying`, and the
/// computed `Released` are always derived.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    Active,
    /// All features released but `released_at` not yet set — awaiting UAT sign-off.
    Verifying,
    Released,
    Skipped,
}

impl fmt::Display for MilestoneStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MilestoneStatus::Active => "active",
            MilestoneStatus::Verifying => "verifying",
            MilestoneStatus::Released => "released",
            MilestoneStatus::Skipped => "skipped",
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Narrative: what "done" looks like from a user's perspective; why this milestone matters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision: Option<String>,
    pub features: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Set when a milestone is explicitly skipped/cancelled.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "cancelled_at"
    )]
    pub skipped_at: Option<DateTime<Utc>>,
    /// Set when a milestone is explicitly marked complete (overrides computed status).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub released_at: Option<DateTime<Utc>>,
}

impl Milestone {
    pub fn new(slug: impl Into<String>, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            slug: slug.into(),
            title: title.into(),
            description: None,
            vision: None,
            features: Vec::new(),
            created_at: now,
            updated_at: now,
            skipped_at: None,
            released_at: None,
        }
    }

    /// Derive milestone status.
    ///
    /// Priority: `Skipped` > explicit `Released` (via `released_at`) > computed.
    /// When all non-archived features are released but `released_at` is not yet
    /// set, the milestone is `Verifying` — code-complete but awaiting UAT.
    /// Call `milestone.release()` (which sets `released_at`) after UAT passes
    /// to advance to `Released`.
    pub fn compute_status(&self, features: &[Feature]) -> MilestoneStatus {
        if self.skipped_at.is_some() {
            return MilestoneStatus::Skipped;
        }
        if self.released_at.is_some() {
            return MilestoneStatus::Released;
        }
        let non_archived: Vec<&Feature> = features
            .iter()
            .filter(|f| self.features.contains(&f.slug) && !f.archived)
            .collect();
        if !non_archived.is_empty() && non_archived.iter().all(|f| f.phase == Phase::Released) {
            return MilestoneStatus::Verifying;
        }
        MilestoneStatus::Active
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
        paths::validate_slug(slug)?;
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

    /// Mark the milestone as explicitly skipped/cancelled. Status becomes `Skipped`.
    pub fn skip(&mut self) {
        self.skipped_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark the milestone as explicitly complete. Status becomes `Released` regardless of features.
    pub fn release(&mut self) {
        self.released_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn update_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = Utc::now();
    }

    pub fn set_vision(&mut self, vision: impl Into<String>) {
        self.vision = Some(vision.into());
        self.updated_at = Utc::now();
    }

    /// Load the acceptance test markdown from `.sdlc/milestones/<slug>/acceptance_test.md`.
    /// Returns `None` if the file does not exist.
    pub fn load_acceptance_test(&self, root: &Path) -> Result<Option<String>> {
        let path = paths::milestone_acceptance_test_path(root, &self.slug);
        if path.exists() {
            Ok(Some(std::fs::read_to_string(&path)?))
        } else {
            Ok(None)
        }
    }

    /// Write the acceptance test to `.sdlc/milestones/<slug>/acceptance_test.md`.
    pub fn save_acceptance_test(&self, root: &Path, content: &str) -> Result<()> {
        let path = paths::milestone_acceptance_test_path(root, &self.slug);
        crate::io::atomic_write(&path, content.as_bytes())
    }

    /// Load the UAT results from `.sdlc/milestones/<slug>/uat_results.md`.
    /// Returns `None` if no run has been recorded yet.
    pub fn load_uat_results(&self, root: &Path) -> Result<Option<String>> {
        let path = paths::milestone_uat_results_path(root, &self.slug);
        if path.exists() {
            Ok(Some(std::fs::read_to_string(&path)?))
        } else {
            Ok(None)
        }
    }

    /// Write (overwrite) the UAT results to `.sdlc/milestones/<slug>/uat_results.md`.
    pub fn save_uat_results(&self, root: &Path, content: &str) -> Result<()> {
        let path = paths::milestone_uat_results_path(root, &self.slug);
        crate::io::atomic_write(&path, content.as_bytes())
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
        assert!(m.features.is_empty());
        assert_eq!(m.compute_status(&[]), MilestoneStatus::Active);

        let loaded = Milestone::load(dir.path(), "v2-launch").unwrap();
        assert_eq!(loaded.title, "v2.0 Launch");
        assert!(loaded.skipped_at.is_none());
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
    fn milestone_skip() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        assert_eq!(m.compute_status(&[]), MilestoneStatus::Active);
        m.skip();
        assert!(m.skipped_at.is_some());
        assert_eq!(m.compute_status(&[]), MilestoneStatus::Skipped);
    }

    #[test]
    fn compute_status_all_features_released_is_verifying() {
        use crate::feature::Feature;
        use crate::types::Phase;

        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.add_feature("b");

        let mut fa = Feature::new("a", "Feature A");
        fa.phase = Phase::Released;
        let mut fb = Feature::new("b", "Feature B");
        fb.phase = Phase::Released;

        // All features released but released_at not set → Verifying (awaiting UAT)
        assert_eq!(m.compute_status(&[fa, fb]), MilestoneStatus::Verifying);
    }

    #[test]
    fn compute_status_released_at_seals_as_released() {
        use crate::feature::Feature;
        use crate::types::Phase;

        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.release(); // explicitly sealed after UAT

        let mut fa = Feature::new("a", "Feature A");
        fa.phase = Phase::Released;

        assert_eq!(m.compute_status(&[fa]), MilestoneStatus::Released);
    }

    #[test]
    fn compute_status_mixed_phases() {
        use crate::feature::Feature;
        use crate::types::Phase;

        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.add_feature("b");

        let mut fa = Feature::new("a", "Feature A");
        fa.phase = Phase::Released;
        let fb = Feature::new("b", "Feature B"); // draft by default

        assert_eq!(m.compute_status(&[fa, fb]), MilestoneStatus::Active);
    }

    #[test]
    fn compute_status_skipped_overrides() {
        use crate::feature::Feature;
        use crate::types::Phase;

        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");
        m.skip();

        let mut fa = Feature::new("a", "Feature A");
        fa.phase = Phase::Released;

        // Even with all released, skipped wins
        assert_eq!(m.compute_status(&[fa]), MilestoneStatus::Skipped);
    }

    #[test]
    fn compute_status_archived_features_ignored() {
        use crate::feature::Feature;
        use crate::types::Phase;

        let dir = TempDir::new().unwrap();
        setup(&dir);

        let mut m = Milestone::create(dir.path(), "v2", "v2").unwrap();
        m.add_feature("a");

        let mut fa = Feature::new("a", "Feature A");
        fa.phase = Phase::Implementation;
        fa.archived = true;

        // No non-archived features → still Active
        assert_eq!(m.compute_status(&[fa]), MilestoneStatus::Active);
    }
}
