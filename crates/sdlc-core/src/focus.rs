use crate::classifier::{Classification, Classifier, EvalContext};
use crate::config::Config;
use crate::error::Result;
use crate::feature::Feature;
use crate::milestone::Milestone;
use crate::rules::default_rules;
use crate::state::State;
use crate::types::ActionType;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

// ---------------------------------------------------------------------------
// FocusResult
// ---------------------------------------------------------------------------

/// The milestone context for a focused feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneSummary {
    pub slug: String,
    pub title: String,
    /// 1-based position of this feature within the milestone.
    pub position: usize,
    /// Total number of features in the milestone.
    pub total: usize,
}

/// The result of `focus()`: the highest-priority actionable directive,
/// enriched with optional milestone context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusResult {
    #[serde(flatten)]
    pub classification: Classification,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<MilestoneSummary>,
}

// ---------------------------------------------------------------------------
// focus()
// ---------------------------------------------------------------------------

fn is_actionable(action: ActionType) -> bool {
    !matches!(
        action,
        ActionType::Done | ActionType::WaitForApproval | ActionType::UnblockDependency
    )
}

/// Return the single highest-priority actionable directive.
///
/// Priority order:
/// 1. Walk `state.milestones` in order; within each milestone, walk
///    `milestone.features` in their stored order.
/// 2. Fall back to features in `state.active_features` that are not
///    assigned to any milestone, in insertion order.
///
/// Skips: archived features, `done`, `wait_for_approval`, `unblock_dependency`.
pub fn focus(root: &Path) -> Result<Option<FocusResult>> {
    let state = State::load(root)?;
    let config = Config::load(root)?;
    let classifier = Classifier::new(default_rules());

    let mut visited: HashSet<String> = HashSet::new();

    // Pass 1 — milestones in state order, features in milestone order
    for milestone_slug in &state.milestones {
        let milestone = match Milestone::load(root, milestone_slug) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let total = milestone.features.len();

        for (idx, feature_slug) in milestone.features.iter().enumerate() {
            visited.insert(feature_slug.clone());

            if let Some(result) = try_classify(
                root,
                feature_slug,
                &state,
                &config,
                &classifier,
                Some(MilestoneSummary {
                    slug: milestone.slug.clone(),
                    title: milestone.title.clone(),
                    position: idx + 1,
                    total,
                }),
            ) {
                return Ok(Some(result));
            }
        }
    }

    // Pass 2 — features not in any milestone, in state.active_features order
    for feature_slug in &state.active_features {
        if visited.contains(feature_slug) {
            continue;
        }
        if let Some(result) = try_classify(root, feature_slug, &state, &config, &classifier, None) {
            return Ok(Some(result));
        }
    }

    Ok(None)
}

fn try_classify(
    root: &Path,
    feature_slug: &str,
    state: &State,
    config: &Config,
    classifier: &Classifier,
    milestone: Option<MilestoneSummary>,
) -> Option<FocusResult> {
    let feature = Feature::load(root, feature_slug).ok()?;
    if feature.archived {
        return None;
    }
    let ctx = EvalContext {
        feature: &feature,
        state,
        config,
        root,
    };
    let classification = classifier.classify(&ctx);
    if is_actionable(classification.action) {
        Some(FocusResult {
            classification,
            milestone,
        })
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::feature::Feature;
    use crate::milestone::Milestone;
    use crate::state::State;
    use crate::types::Phase;
    use tempfile::TempDir;

    fn setup(dir: &TempDir) -> State {
        let root = dir.path();
        std::fs::create_dir_all(root.join(".sdlc/features")).unwrap();
        std::fs::create_dir_all(root.join(".sdlc/milestones")).unwrap();

        Config::new("test").save(root).unwrap();

        let state = State::new("test");
        state.save(root).unwrap();
        state
    }

    fn create_feature(dir: &TempDir, slug: &str) {
        Feature::create(dir.path(), slug, slug).unwrap();
        let mut state = State::load(dir.path()).unwrap();
        state.add_active_feature(slug);
        state.save(dir.path()).unwrap();
    }

    #[test]
    fn focus_returns_none_when_no_features() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        let result = focus(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn focus_returns_first_actionable_feature() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        create_feature(&dir, "alpha");
        create_feature(&dir, "beta");

        let result = focus(dir.path()).unwrap().unwrap();
        assert_eq!(result.classification.feature, "alpha");
        assert_eq!(result.classification.action, ActionType::CreateSpec);
        assert!(result.milestone.is_none());
    }

    #[test]
    fn focus_respects_milestone_order() {
        let dir = TempDir::new().unwrap();
        let mut state = setup(&dir);

        create_feature(&dir, "alpha");
        create_feature(&dir, "beta");

        // Create milestone with beta first, then alpha
        let mut m = Milestone::create(dir.path(), "v1", "V1").unwrap();
        m.add_feature("beta");
        m.add_feature("alpha");
        m.save(dir.path()).unwrap();

        state.add_milestone("v1");
        state.save(dir.path()).unwrap();

        let result = focus(dir.path()).unwrap().unwrap();
        // beta is first in the milestone, so it should be focused
        assert_eq!(result.classification.feature, "beta");
        let ms = result.milestone.unwrap();
        assert_eq!(ms.slug, "v1");
        assert_eq!(ms.position, 1);
        assert_eq!(ms.total, 2);
    }

    #[test]
    fn focus_skips_done_features() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // Create a feature and transition it to released (done)
        create_feature(&dir, "shipped");
        let mut f = Feature::load(dir.path(), "shipped").unwrap();
        f.phase = Phase::Released;
        f.save(dir.path()).unwrap();

        create_feature(&dir, "active");

        let result = focus(dir.path()).unwrap().unwrap();
        assert_eq!(result.classification.feature, "active");
    }

    #[test]
    fn focus_skips_archived_features() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        create_feature(&dir, "archived-one");
        let mut f = Feature::load(dir.path(), "archived-one").unwrap();
        f.archived = true;
        f.save(dir.path()).unwrap();

        create_feature(&dir, "live");

        let result = focus(dir.path()).unwrap().unwrap();
        assert_eq!(result.classification.feature, "live");
    }

    #[test]
    fn focus_falls_back_to_non_milestone_features() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // Feature in milestone — released (done)
        create_feature(&dir, "in-milestone");
        let mut f = Feature::load(dir.path(), "in-milestone").unwrap();
        f.phase = Phase::Released;
        f.save(dir.path()).unwrap();

        // Feature not in any milestone
        create_feature(&dir, "orphan");

        // Attach milestone — reload state so we don't overwrite the features added above
        let mut m = Milestone::create(dir.path(), "v1", "V1").unwrap();
        m.add_feature("in-milestone");
        m.save(dir.path()).unwrap();

        let mut state = State::load(dir.path()).unwrap();
        state.add_milestone("v1");
        state.save(dir.path()).unwrap();

        let result = focus(dir.path()).unwrap().unwrap();
        assert_eq!(result.classification.feature, "orphan");
        assert!(result.milestone.is_none());
    }
}
