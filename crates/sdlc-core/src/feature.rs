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
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Deserialize `artifacts` accepting both a YAML sequence (`[]`) and an empty
/// YAML map (`{}`). Older manifest files were written with `artifacts: {}` by
/// AI agents that serialized an empty collection as a map instead of a list.
fn deserialize_artifacts<'de, D>(deserializer: D) -> std::result::Result<Vec<Artifact>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, MapAccess, SeqAccess, Visitor};
    use std::fmt;

    struct ArtifactsVisitor;

    impl<'de> Visitor<'de> for ArtifactsVisitor {
        type Value = Vec<Artifact>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "a sequence of artifacts or an empty map")
        }

        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut vec = Vec::new();
            while let Some(item) = seq.next_element::<Artifact>()? {
                vec.push(item);
            }
            Ok(vec)
        }

        fn visit_map<A: MapAccess<'de>>(
            self,
            mut map: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            // Accept only an empty map (artifacts: {}) — treat as empty vec.
            if map
                .next_entry::<serde::de::IgnoredAny, serde::de::IgnoredAny>()?
                .is_some()
            {
                return Err(A::Error::custom(
                    "artifacts map must be empty if present as a map",
                ));
            }
            Ok(Vec::new())
        }
    }

    deserializer.deserialize_any(ArtifactsVisitor)
}

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
    #[serde(deserialize_with = "deserialize_artifacts")]
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
    /// Schema version for this manifest. Migration on load brings older files
    /// to [`crate::migrations::FEATURE_SCHEMA_VERSION`]. New files are always
    /// stamped with the current version by the constructor.
    #[serde(default)]
    pub schema_version: u32,
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
            schema_version: crate::migrations::FEATURE_SCHEMA_VERSION,
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
        paths::validate_slug(slug)?;
        let manifest = paths::feature_manifest(root, slug);
        if !manifest.exists() {
            return Err(SdlcError::FeatureNotFound(slug.to_string()));
        }

        let path_display = manifest.display().to_string();
        let data = std::fs::read_to_string(&manifest)?;

        // Phase 1: parse raw YAML — catches syntax errors with path context.
        let mut value: serde_yaml::Value =
            serde_yaml::from_str(&data).map_err(|e| SdlcError::ManifestParseFailed {
                path: path_display.clone(),
                message: e.to_string(),
            })?;

        // Phase 2: migrate to current schema version.
        let migrated = crate::migrations::migrate_feature(&mut value).map_err(|msg| {
            SdlcError::ManifestIncompatible {
                path: path_display.clone(),
                entity: "Feature".to_string(),
                message: msg,
                fix_hint: "Run `sdlc doctor --fix` to attempt auto-repair.".to_string(),
            }
        })?;

        // Phase 3: deserialize into the typed struct. Any remaining mismatch
        // (e.g. wrong type for a field) surfaces here with an actionable hint.
        let feature: Feature =
            serde_yaml::from_value(value).map_err(|e| SdlcError::ManifestIncompatible {
                path: path_display.clone(),
                entity: "Feature".to_string(),
                message: e.to_string(),
                fix_hint: crate::migrations::feature_fix_hint(&e),
            })?;

        // Phase 4: self-heal — rewrite the file if migration upgraded it.
        if migrated {
            let _ = feature.save(root); // best-effort; load still succeeds on save failure
        }

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
    // Dependency cycle detection
    // ---------------------------------------------------------------------------

    /// Validate that setting `new_deps` for `slug` does not introduce a dependency cycle.
    ///
    /// `all_features` maps each feature slug to its current dependencies. This function
    /// builds a proposed graph — replacing `slug`'s deps with `new_deps` — and runs a
    /// DFS from `slug` to check reachability back to `slug`.
    ///
    /// Returns `Ok(())` if the graph is acyclic, or `Err(SdlcError::DependencyCycle(…))`
    /// with a human-readable path if a cycle is found.
    ///
    /// Empty `new_deps` is always valid; no cycle is possible.
    pub fn validate_no_dep_cycle(
        slug: &str,
        new_deps: &[String],
        all_features: &HashMap<String, Vec<String>>,
    ) -> Result<()> {
        if new_deps.is_empty() {
            return Ok(());
        }

        // Build the proposed graph: same as all_features but with slug's deps replaced.
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        for (k, deps) in all_features {
            let entry: Vec<&str> = deps.iter().map(|s| s.as_str()).collect();
            graph.insert(k.as_str(), entry);
        }
        graph.insert(slug, new_deps.iter().map(|s| s.as_str()).collect());

        // DFS from slug looking for a path back to slug.
        // Returns the cycle path (slug → … → slug) if found, or None.
        fn dfs<'a>(
            start: &'a str,
            current: &'a str,
            graph: &'a HashMap<&'a str, Vec<&'a str>>,
            visited: &mut HashSet<&'a str>,
            path: &mut Vec<&'a str>,
        ) -> Option<Vec<String>> {
            if let Some(neighbors) = graph.get(current) {
                for &neighbor in neighbors {
                    if neighbor == start {
                        // Cycle found — build the path string
                        let mut cycle = path.iter().map(|s| format!("'{s}'")).collect::<Vec<_>>();
                        cycle.push(format!("'{start}'"));
                        return Some(cycle);
                    }
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor);
                        path.push(neighbor);
                        if let Some(cycle) = dfs(start, neighbor, graph, visited, path) {
                            return Some(cycle);
                        }
                        path.pop();
                        visited.remove(neighbor);
                    }
                }
            }
            None
        }

        let mut visited: HashSet<&str> = HashSet::new();
        visited.insert(slug);
        let mut path: Vec<&str> = vec![slug];

        if let Some(cycle) = dfs(slug, slug, &graph, &mut visited, &mut path) {
            return Err(SdlcError::DependencyCycle(cycle.join(" → ")));
        }

        Ok(())
    }

    /// Build a slug → deps map for all features currently on disk.
    pub fn dep_graph(root: &Path) -> Result<HashMap<String, Vec<String>>> {
        let features = Self::list(root)?;
        Ok(features
            .into_iter()
            .map(|f| (f.slug, f.dependencies))
            .collect())
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

    /// Remove the blocker at the given index. Returns an error if `idx` is out of range.
    pub fn remove_blocker(&mut self, idx: usize) -> Result<()> {
        if idx >= self.blockers.len() {
            return Err(SdlcError::InvalidPhase(format!(
                "blocker index {} out of range (len={})",
                idx,
                self.blockers.len()
            )));
        }
        self.blockers.remove(idx);
        self.updated_at = Utc::now();
        Ok(())
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
    fn remove_blocker_removes_correct_element() {
        let mut feature = Feature::new("test", "Test");
        feature.blockers = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        feature.remove_blocker(1).unwrap();
        assert_eq!(feature.blockers, vec!["a", "c"]);
    }

    #[test]
    fn remove_blocker_out_of_range_returns_err() {
        let mut feature = Feature::new("test", "Test");
        feature.blockers = vec!["a".to_string()];
        assert!(feature.remove_blocker(1).is_err());
    }

    #[test]
    fn remove_blocker_empty_list_returns_err() {
        let mut feature = Feature::new("test", "Test");
        assert!(feature.remove_blocker(0).is_err());
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

    // ---------------------------------------------------------------------------
    // Dependency cycle detection tests
    // ---------------------------------------------------------------------------

    fn make_graph(pairs: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, vs)| (k.to_string(), vs.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    #[test]
    fn no_cycle_empty_deps() {
        let graph = make_graph(&[]);
        // Empty new_deps is always valid
        assert!(Feature::validate_no_dep_cycle("a", &[], &graph).is_ok());
    }

    #[test]
    fn no_cycle_simple_chain() {
        // a → b → c — no cycle
        let graph = make_graph(&[("b", &["c"]), ("c", &[])]);
        assert!(Feature::validate_no_dep_cycle("a", &["b".to_string()], &graph).is_ok());
    }

    #[test]
    fn cycle_direct_two_nodes() {
        // a depends on b; b already depends on a → cycle
        let graph = make_graph(&[("b", &["a"])]);
        let result = Feature::validate_no_dep_cycle("a", &["b".to_string()], &graph);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("cycle"), "expected 'cycle' in: {msg}");
        assert!(msg.contains("'a'"), "expected feature 'a' in: {msg}");
        assert!(msg.contains("'b'"), "expected feature 'b' in: {msg}");
    }

    #[test]
    fn cycle_longer_chain() {
        // a → b → c → a is a cycle; we're setting a's deps to [b]
        let graph = make_graph(&[("b", &["c"]), ("c", &["a"])]);
        let result = Feature::validate_no_dep_cycle("a", &["b".to_string()], &graph);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("cycle"), "expected 'cycle' in: {msg}");
    }

    #[test]
    fn no_cycle_diamond() {
        // a → b, a → c, both b and c depend on d — no cycle
        let graph = make_graph(&[("b", &["d"]), ("c", &["d"]), ("d", &[])]);
        let result =
            Feature::validate_no_dep_cycle("a", &["b".to_string(), "c".to_string()], &graph);
        assert!(result.is_ok());
    }

    #[test]
    fn cycle_on_disk_integration() {
        // Full integration: create two features on disk, set a→b, then try b→a.
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();

        let mut feat_a = Feature::create(dir.path(), "feat-a", "A").unwrap();
        Feature::create(dir.path(), "feat-b", "B").unwrap();

        // Set feat-a to depend on feat-b (no cycle yet)
        feat_a.dependencies = vec!["feat-b".to_string()];
        feat_a.save(dir.path()).unwrap();

        // Now try to set feat-b → feat-a — should detect cycle
        let graph = Feature::dep_graph(dir.path()).unwrap();
        let result = Feature::validate_no_dep_cycle("feat-b", &["feat-a".to_string()], &graph);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("cycle"),
            "expected 'cycle' in error message: {msg}"
        );
        assert!(msg.contains("'feat-a'"), "expected 'feat-a' in: {msg}");
        assert!(msg.contains("'feat-b'"), "expected 'feat-b' in: {msg}");
    }

    #[test]
    fn self_dep_is_caught_before_cycle_check() {
        // Self-dependency is already handled by the CLI guard (dep == slug),
        // but validate_no_dep_cycle also catches it via the DFS.
        let graph = make_graph(&[]);
        let result = Feature::validate_no_dep_cycle("a", &["a".to_string()], &graph);
        assert!(result.is_err());
    }
}
