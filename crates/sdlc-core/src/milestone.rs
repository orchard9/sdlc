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
