use crate::artifact::Artifact;
use crate::comment::Comment;
use crate::config::Config;
use crate::error::{Result, SdlcError};
use crate::paths;
use crate::score::QualityScore;
use crate::task::Task;
use crate::types::{ArtifactStatus, ArtifactType, Phase};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// PhaseTransition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub phase: Phase,
    pub entered: DateTime<Utc>,
    pub exited: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Feature
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub slug: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub phase: Phase,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub artifacts: Vec<Artifact>,
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub next_comment_seq: u32,
    pub blockers: Vec<String>,
    pub phase_history: Vec<PhaseTransition>,
    pub dependencies: Vec<String>,
    pub archived: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scores: Vec<QualityScore>,
}

impl Feature {
    pub fn new(slug: impl Into<String>, title: impl Into<String>) -> Self {
        Self::with_description(slug, title, None)
    }

    pub fn with_description(
        slug: impl Into<String>,
        title: impl Into<String>,
        description: Option<String>,
    ) -> Self {
        let slug = slug.into();
        let title = title.into();
        let now = Utc::now();

        let artifacts = Self::default_artifacts(&slug);

        Self {
            slug,
            title,
            description,
            phase: Phase::Draft,
            created_at: now,
            updated_at: now,
            artifacts,
            tasks: Vec::new(),
            comments: Vec::new(),
            next_comment_seq: 0,
            blockers: Vec::new(),
            phase_history: vec![PhaseTransition {
                phase: Phase::Draft,
                entered: now,
                exited: None,
            }],
            dependencies: Vec::new(),
            archived: false,
            scores: Vec::new(),
        }
    }

    fn default_artifacts(slug: &str) -> Vec<Artifact> {
        let types = [
            ArtifactType::Spec,
            ArtifactType::Design,
            ArtifactType::Tasks,
            ArtifactType::QaPlan,
            ArtifactType::Review,
            ArtifactType::Audit,
            ArtifactType::QaResults,
        ];
        types
            .iter()
            .map(|&t| Artifact::new(t, format!(".sdlc/features/{}/{}", slug, t.filename())))
            .collect()
    }

    // ---------------------------------------------------------------------------
    // Persistence
    // ---------------------------------------------------------------------------

    pub fn create(root: &Path, slug: impl Into<String>, title: impl Into<String>) -> Result<Self> {
        Self::create_with_description(root, slug, title, None)
    }

    pub fn create_with_description(
        root: &Path,
        slug: impl Into<String>,
        title: impl Into<String>,
        description: Option<String>,
    ) -> Result<Self> {
        let slug = slug.into();
        paths::validate_slug(&slug)?;

        let feature_dir = paths::feature_dir(root, &slug);
        if feature_dir.exists() {
            return Err(SdlcError::FeatureExists(slug));
        }

        let feature = Self::with_description(slug, title, description);
        feature.save(root)?;
        Ok(feature)
    }

    pub fn load(root: &Path, slug: &str) -> Result<Self> {
        let manifest = paths::feature_manifest(root, slug);
        if !manifest.exists() {
            return Err(SdlcError::FeatureNotFound(slug.to_string()));
        }
        let data = std::fs::read_to_string(&manifest)?;
        let feature: Feature = serde_yaml::from_str(&data)?;
        Ok(feature)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let manifest = paths::feature_manifest(root, &self.slug);
        let data = serde_yaml::to_string(self)?;
        crate::io::atomic_write(&manifest, data.as_bytes())
    }

    pub fn list(root: &Path) -> Result<Vec<Self>> {
        let features_dir = root.join(paths::FEATURES_DIR);
        if !features_dir.exists() {
            return Ok(Vec::new());
        }

        let mut features = Vec::new();
        for entry in std::fs::read_dir(&features_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let slug = entry.file_name().to_string_lossy().into_owned();
                match Self::load(root, &slug) {
                    Ok(f) => features.push(f),
                    Err(SdlcError::FeatureNotFound(_)) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        features.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(features)
    }

    // ---------------------------------------------------------------------------
    // Phase transitions
    // ---------------------------------------------------------------------------

    pub fn can_transition_to(&self, target: Phase, cfg: &Config) -> Result<()> {
        if !cfg.phases.is_enabled(target) {
            return Err(SdlcError::InvalidTransition {
                from: self.phase.to_string(),
                to: target.to_string(),
                reason: format!("phase '{target}' is not enabled"),
            });
        }

        if target <= self.phase {
            return Err(SdlcError::InvalidTransition {
                from: self.phase.to_string(),
                to: target.to_string(),
                reason: "transitions are forward-only".to_string(),
            });
        }

        // Check required artifacts for the target phase
        let required = cfg.phases.required_for(target);
        for &artifact_type in required {
            let artifact = self.artifact(artifact_type);
            if !artifact.map(|a| a.is_satisfied()).unwrap_or(false) {
                return Err(SdlcError::MissingArtifact {
                    artifact: artifact_type.to_string(),
                    phase: target.to_string(),
                });
            }
        }

        Ok(())
    }

    pub fn transition(&mut self, target: Phase, cfg: &Config) -> Result<()> {
        self.can_transition_to(target, cfg)?;

        let now = Utc::now();
        if let Some(last) = self.phase_history.last_mut() {
            last.exited = Some(now);
        }

        self.phase = target;
        self.updated_at = now;
        self.phase_history.push(PhaseTransition {
            phase: target,
            entered: now,
            exited: None,
        });

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Artifact helpers
    // ---------------------------------------------------------------------------

    pub fn artifact(&self, artifact_type: ArtifactType) -> Option<&Artifact> {
        self.artifacts
            .iter()
            .find(|a| a.artifact_type == artifact_type)
    }

    pub fn artifact_mut(&mut self, artifact_type: ArtifactType) -> Option<&mut Artifact> {
        self.artifacts
            .iter_mut()
            .find(|a| a.artifact_type == artifact_type)
    }

    pub fn approve_artifact(
        &mut self,
        artifact_type: ArtifactType,
        by: Option<String>,
    ) -> Result<()> {
        let artifact = self
            .artifacts
            .iter_mut()
            .find(|a| a.artifact_type == artifact_type)
            .ok_or_else(|| SdlcError::ArtifactNotFound(artifact_type.to_string()))?;
        artifact.approve(by);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn reject_artifact(
        &mut self,
        artifact_type: ArtifactType,
        reason: Option<String>,
    ) -> Result<()> {
        let artifact = self
            .artifacts
            .iter_mut()
            .find(|a| a.artifact_type == artifact_type)
            .ok_or_else(|| SdlcError::ArtifactNotFound(artifact_type.to_string()))?;
        artifact.reject(reason);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn mark_artifact_draft(&mut self, artifact_type: ArtifactType) -> Result<()> {
        let artifact = self
            .artifacts
            .iter_mut()
            .find(|a| a.artifact_type == artifact_type)
            .ok_or_else(|| SdlcError::ArtifactNotFound(artifact_type.to_string()))?;
        artifact.mark_draft();
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn waive_artifact(
        &mut self,
        artifact_type: ArtifactType,
        reason: Option<String>,
    ) -> Result<()> {
        let artifact = self
            .artifacts
            .iter_mut()
            .find(|a| a.artifact_type == artifact_type)
            .ok_or_else(|| SdlcError::ArtifactNotFound(artifact_type.to_string()))?;
        artifact.waive(reason);
        self.updated_at = Utc::now();
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Metadata mutations
    // ---------------------------------------------------------------------------

    pub fn update_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = Utc::now();
    }

    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
        self.updated_at = Utc::now();
    }

    pub fn clear_description(&mut self) {
        self.description = None;
        self.updated_at = Utc::now();
    }

    // ---------------------------------------------------------------------------
    // Quality score helpers
    // ---------------------------------------------------------------------------

    /// Add or replace a quality score for a given lens.
    pub fn add_score(&mut self, score: QualityScore) {
        self.scores.retain(|s| s.lens != score.lens);
        self.scores.push(score);
        self.updated_at = Utc::now();
    }

    /// Get the current score for a given lens.
    pub fn score_for(&self, lens: &str) -> Option<&QualityScore> {
        self.scores.iter().find(|s| s.lens == lens)
    }

    /// Returns true if all scores are at or above the given threshold.
    /// Returns false if there are no scores.
    pub fn all_scores_above(&self, threshold: u32) -> bool {
        if self.scores.is_empty() {
            return false;
        }
        self.scores.iter().all(|s| s.score >= threshold)
    }

    // ---------------------------------------------------------------------------
    // Misc helpers
    // ---------------------------------------------------------------------------

    pub fn is_blocked(&self) -> bool {
        !self.blockers.is_empty()
    }

    pub fn all_artifacts_approved_for(&self, phase: Phase, cfg: &Config) -> bool {
        cfg.phases
            .required_for(phase)
            .iter()
            .all(|&t| self.artifact(t).map(|a| a.is_satisfied()).unwrap_or(false))
    }

    /// Returns artifacts that exist on disk but are still in Missing/Draft status.
    pub fn unapproved_artifacts(&self) -> Vec<&Artifact> {
        self.artifacts
            .iter()
            .filter(|a| matches!(a.status, ArtifactStatus::Draft | ArtifactStatus::NeedsFix))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_config() -> Config {
        Config::new("test")
    }

    #[test]
    fn feature_create_load() {
        let dir = TempDir::new().unwrap();
        // Need to init the .sdlc dir
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();

        let feature = Feature::create(dir.path(), "auth-login", "Auth Login").unwrap();
        assert_eq!(feature.slug, "auth-login");
        assert_eq!(feature.phase, Phase::Draft);

        let loaded = Feature::load(dir.path(), "auth-login").unwrap();
        assert_eq!(loaded.title, "Auth Login");
    }

    #[test]
    fn feature_create_duplicate_fails() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();

        Feature::create(dir.path(), "auth", "Auth").unwrap();
        assert!(Feature::create(dir.path(), "auth", "Auth Again").is_err());
    }

    #[test]
    fn transition_requires_artifacts() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();

        let mut feature = Feature::create(dir.path(), "test-feat", "Test").unwrap();
        let cfg = make_config();

        // Can't go to Specified without approved spec
        assert!(feature.transition(Phase::Specified, &cfg).is_err());

        // Approve spec → can now transition
        feature.approve_artifact(ArtifactType::Spec, None).unwrap();
        feature.transition(Phase::Specified, &cfg).unwrap();
        assert_eq!(feature.phase, Phase::Specified);
    }

    #[test]
    fn feature_description_round_trip() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();

        let feature = Feature::create_with_description(
            dir.path(),
            "desc-test",
            "Desc Test",
            Some("OAuth with Google and GitHub.".to_string()),
        )
        .unwrap();
        assert_eq!(
            feature.description.as_deref(),
            Some("OAuth with Google and GitHub.")
        );

        let loaded = Feature::load(dir.path(), "desc-test").unwrap();
        assert_eq!(
            loaded.description.as_deref(),
            Some("OAuth with Google and GitHub.")
        );
    }

    #[test]
    fn feature_without_description_compat() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();

        // Create without description (backwards compat)
        let feature = Feature::create(dir.path(), "no-desc", "No Desc").unwrap();
        assert!(feature.description.is_none());

        let loaded = Feature::load(dir.path(), "no-desc").unwrap();
        assert!(loaded.description.is_none());
    }

    #[test]
    fn feature_add_score() {
        let mut feature = Feature::new("test", "Test");
        let score = QualityScore {
            lens: "product_fit".to_string(),
            score: 85,
            deductions: vec![],
            evaluator: "review-agent".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
        };
        feature.add_score(score);
        assert_eq!(feature.scores.len(), 1);
        assert_eq!(feature.score_for("product_fit").unwrap().score, 85);
    }

    #[test]
    fn feature_add_score_replaces_existing_lens() {
        let mut feature = Feature::new("test", "Test");
        feature.add_score(QualityScore {
            lens: "product_fit".to_string(),
            score: 60,
            deductions: vec![],
            evaluator: "agent-1".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
        });
        feature.add_score(QualityScore {
            lens: "product_fit".to_string(),
            score: 90,
            deductions: vec![],
            evaluator: "agent-2".to_string(),
            timestamp: "2026-02-24T01:00:00Z".to_string(),
        });
        assert_eq!(feature.scores.len(), 1);
        assert_eq!(feature.score_for("product_fit").unwrap().score, 90);
        assert_eq!(
            feature.score_for("product_fit").unwrap().evaluator,
            "agent-2"
        );
    }

    #[test]
    fn feature_score_for_missing_lens() {
        let feature = Feature::new("test", "Test");
        assert!(feature.score_for("nonexistent").is_none());
    }

    #[test]
    fn feature_all_scores_above_empty() {
        let feature = Feature::new("test", "Test");
        assert!(!feature.all_scores_above(70));
    }

    #[test]
    fn feature_all_scores_above_mixed() {
        let mut feature = Feature::new("test", "Test");
        feature.add_score(QualityScore {
            lens: "product_fit".to_string(),
            score: 85,
            deductions: vec![],
            evaluator: "agent".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
        });
        feature.add_score(QualityScore {
            lens: "implementation".to_string(),
            score: 65,
            deductions: vec![],
            evaluator: "agent".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
        });
        assert!(!feature.all_scores_above(70));
        assert!(feature.all_scores_above(60));
    }

    #[test]
    fn feature_all_scores_above_all_pass() {
        let mut feature = Feature::new("test", "Test");
        feature.add_score(QualityScore {
            lens: "product_fit".to_string(),
            score: 80,
            deductions: vec![],
            evaluator: "agent".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
        });
        feature.add_score(QualityScore {
            lens: "implementation".to_string(),
            score: 90,
            deductions: vec![],
            evaluator: "agent".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
        });
        assert!(feature.all_scores_above(80));
    }

    #[test]
    fn feature_scores_not_serialized_when_empty() {
        let feature = Feature::new("test", "Test");
        let yaml = serde_yaml::to_string(&feature).unwrap();
        assert!(!yaml.contains("scores"));
    }

    #[test]
    fn feature_scores_roundtrip() {
        let mut feature = Feature::new("test", "Test");
        feature.add_score(QualityScore {
            lens: "product_fit".to_string(),
            score: 85,
            deductions: vec![],
            evaluator: "review-agent".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
        });
        let yaml = serde_yaml::to_string(&feature).unwrap();
        assert!(yaml.contains("scores"));
        let parsed: Feature = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.scores.len(), 1);
        assert_eq!(parsed.scores[0].lens, "product_fit");
        assert_eq!(parsed.scores[0].score, 85);
    }

    #[test]
    fn forward_only_transitions() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();

        let mut feature = Feature::create(dir.path(), "f1", "F1").unwrap();
        let cfg = make_config();
        feature.approve_artifact(ArtifactType::Spec, None).unwrap();
        feature.transition(Phase::Specified, &cfg).unwrap();

        // Can't go back
        assert!(feature.transition(Phase::Draft, &cfg).is_err());
    }
}
