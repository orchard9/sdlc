use crate::classifier::{Classifier, EvalContext};
use crate::config::Config;
use crate::feature::Feature;
use crate::milestone::{Milestone, MilestoneStatus};
use crate::state::State;
use crate::types::ActionType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const MAX_PARALLEL: usize = 4;
const MAX_UAT: usize = 1;

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelWorkItem {
    pub milestone_slug: String,
    pub milestone_title: String,
    #[serde(flatten)]
    pub kind: WorkItemKind,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkItemKind {
    Feature { slug: String, next_action: String },
    Uat,
}

// ---------------------------------------------------------------------------
// Core selection logic — single source of truth for dashboard + dev-driver
// ---------------------------------------------------------------------------

/// Select up to MAX_PARALLEL work items from current milestones.
///
/// Rules (identical to the dashboard's MilestoneDigestRow logic):
/// 1. Filter milestones: skip Released and Skipped.
/// 2. Preserve created_at order (Milestone::list already sorts ascending).
/// 3. Per milestone:
///    - Verifying (all features released, awaiting UAT) → UAT slot.
///    - Otherwise → first non-done, non-archived, non-skipped feature.
/// 4. Cap: 4 total, 1 UAT.
///
/// `milestones` must be the output of `Milestone::list` (sorted by created_at).
/// `next_actions` maps feature slug → ActionType (caller runs the classifier once).
pub fn select_parallel_work(
    milestones: &[Milestone],
    features: &[Feature],
    next_actions: &HashMap<String, ActionType>,
) -> Vec<ParallelWorkItem> {
    let feature_map: HashMap<&str, &Feature> =
        features.iter().map(|f| (f.slug.as_str(), f)).collect();

    let mut items: Vec<ParallelWorkItem> = Vec::new();
    let mut uat_count = 0;

    for milestone in milestones {
        if items.len() >= MAX_PARALLEL {
            break;
        }

        let status = milestone.compute_status(features);
        match status {
            MilestoneStatus::Released | MilestoneStatus::Skipped => continue,
            MilestoneStatus::Verifying => {
                if uat_count < MAX_UAT {
                    items.push(ParallelWorkItem {
                        milestone_slug: milestone.slug.clone(),
                        milestone_title: milestone.title.clone(),
                        kind: WorkItemKind::Uat,
                        command: format!("/sdlc-milestone-uat {}", milestone.slug),
                    });
                    uat_count += 1;
                }
                continue;
            }
            MilestoneStatus::Active => {}
        }

        // Active milestone: find first actionable feature in milestone's feature order.
        // This mirrors the dashboard's Array.find() in MilestoneDigestRow.tsx.
        let next = milestone.features.iter().find_map(|slug| {
            let f = feature_map.get(slug.as_str())?;
            if f.archived {
                return None;
            }
            let action = next_actions.get(slug.as_str())?;
            if *action == ActionType::Done {
                return None;
            }
            // Respect skip:autonomous — same tag the old dev-driver checked via file read
            if f.tasks
                .iter()
                .any(|t| t.title.to_lowercase().contains("skip:autonomous"))
            {
                return None;
            }
            Some((f, *action))
        });

        if let Some((f, action)) = next {
            items.push(ParallelWorkItem {
                milestone_slug: milestone.slug.clone(),
                milestone_title: milestone.title.clone(),
                kind: WorkItemKind::Feature {
                    slug: f.slug.clone(),
                    next_action: action.as_str().to_string(),
                },
                command: format!("/sdlc-run {}", f.slug),
            });
        }
    }

    items
}

/// Convenience: load everything from disk and run the classifier in one call.
/// Used by the CLI command; the server calls `select_parallel_work` directly
/// after its own classifier pass to avoid double-classifying.
pub fn select_parallel_work_from_root(root: &Path) -> crate::Result<Vec<ParallelWorkItem>> {
    let state = State::load(root)?;
    let features = Feature::list(root)?;
    let milestones = Milestone::list(root)?;
    let config = Config::load(root)?;
    let classifier = Classifier::new(crate::rules::default_rules());

    let mut next_actions: HashMap<String, ActionType> = HashMap::new();
    for f in &features {
        let ctx = EvalContext {
            feature: f,
            state: &state,
            config: &config,
            root,
        };
        let classification = classifier.classify(&ctx);
        next_actions.insert(f.slug.clone(), classification.action);
    }

    Ok(select_parallel_work(&milestones, &features, &next_actions))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::Feature;
    use crate::milestone::Milestone;
    use crate::task::Task;
    use crate::types::{ActionType, Phase};
    use std::collections::HashMap;

    fn make_milestone(slug: &str, features: Vec<String>) -> Milestone {
        let mut m = Milestone::new(slug, format!("Milestone {slug}"));
        m.features = features;
        m
    }

    fn make_feature(slug: &str, phase: Phase, archived: bool) -> Feature {
        let mut f = Feature::new(slug, format!("Feature {slug}"));
        f.phase = phase;
        f.archived = archived;
        f
    }

    fn make_feature_with_skip(slug: &str) -> Feature {
        let mut f = Feature::new(slug, format!("Feature {slug}"));
        f.phase = Phase::Implementation;
        f.tasks = vec![Task::new("t1", "skip:autonomous: needs human review")];
        f
    }

    fn actions(pairs: &[(&str, ActionType)]) -> HashMap<String, ActionType> {
        pairs.iter().map(|(s, a)| (s.to_string(), *a)).collect()
    }

    #[test]
    fn picks_first_non_done_per_milestone() {
        let m = make_milestone("m1", vec!["done-f".to_string(), "active-f".to_string()]);
        let features = vec![
            make_feature("done-f", Phase::Released, false),
            make_feature("active-f", Phase::Implementation, false),
        ];
        let next_actions = actions(&[
            ("done-f", ActionType::Done),
            ("active-f", ActionType::ImplementTask),
        ]);

        let items = select_parallel_work(&[m], &features, &next_actions);
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0].kind, WorkItemKind::Feature { slug, .. } if slug == "active-f"));
    }

    #[test]
    fn caps_at_four_total() {
        let milestones: Vec<Milestone> = (0..6)
            .map(|i| make_milestone(&format!("m{i}"), vec![format!("f{i}")]))
            .collect();
        let features: Vec<Feature> = (0..6)
            .map(|i| make_feature(&format!("f{i}"), Phase::Implementation, false))
            .collect();
        let next_actions: HashMap<String, ActionType> = (0..6)
            .map(|i| (format!("f{i}"), ActionType::ImplementTask))
            .collect();

        let items = select_parallel_work(&milestones, &features, &next_actions);
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn uat_slot_for_verifying_milestone() {
        let m = make_milestone("m1", vec!["f1".to_string()]);
        // All features released → Verifying
        let f = Feature {
            phase: Phase::Released,
            ..make_feature("f1", Phase::Released, false)
        };
        let next_actions = actions(&[("f1", ActionType::Done)]);

        let items = select_parallel_work(std::slice::from_ref(&m), &[f], &next_actions);
        assert_eq!(items.len(), 1);
        assert!(matches!(items[0].kind, WorkItemKind::Uat));
        assert_eq!(items[0].command, "/sdlc-milestone-uat m1");
    }

    #[test]
    fn caps_uat_at_one() {
        // Two verifying milestones — only one UAT slot allowed
        let features: Vec<Feature> = (0..2)
            .map(|i| make_feature(&format!("f{i}"), Phase::Released, false))
            .collect();
        let milestones: Vec<Milestone> = (0..2)
            .map(|i| make_milestone(&format!("m{i}"), vec![format!("f{i}")]))
            .collect();
        let next_actions: HashMap<String, ActionType> = (0..2)
            .map(|i| (format!("f{i}"), ActionType::Done))
            .collect();

        let items = select_parallel_work(&milestones, &features, &next_actions);
        let uat_count = items
            .iter()
            .filter(|i| matches!(i.kind, WorkItemKind::Uat))
            .count();
        assert_eq!(uat_count, 1);
    }

    #[test]
    fn skips_skip_autonomous_features() {
        let m = make_milestone("m1", vec!["skip-f".to_string(), "ok-f".to_string()]);
        let features = vec![
            make_feature_with_skip("skip-f"),
            make_feature("ok-f", Phase::Implementation, false),
        ];
        let next_actions = actions(&[
            ("skip-f", ActionType::ImplementTask),
            ("ok-f", ActionType::ImplementTask),
        ]);

        let items = select_parallel_work(&[m], &features, &next_actions);
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0].kind, WorkItemKind::Feature { slug, .. } if slug == "ok-f"));
    }

    #[test]
    fn skips_archived_features() {
        let m = make_milestone("m1", vec!["arch-f".to_string(), "live-f".to_string()]);
        let features = vec![
            make_feature("arch-f", Phase::Implementation, true), // archived
            make_feature("live-f", Phase::Implementation, false),
        ];
        let next_actions = actions(&[
            ("arch-f", ActionType::ImplementTask),
            ("live-f", ActionType::ImplementTask),
        ]);

        let items = select_parallel_work(&[m], &features, &next_actions);
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0].kind, WorkItemKind::Feature { slug, .. } if slug == "live-f"));
    }

    #[test]
    fn empty_when_all_done() {
        // A feature in Merge phase with Done action — milestone is Active but no slot produced.
        let m = make_milestone("m1", vec!["done-f".to_string()]);
        let features = vec![make_feature("done-f", Phase::Merge, false)];
        let next_actions = actions(&[("done-f", ActionType::Done)]);

        let items = select_parallel_work(&[m], &features, &next_actions);
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn feature_in_milestone_but_absent_from_next_actions_is_skipped() {
        // Feature exists in milestone.features but has no entry in next_actions.
        // The find_map returns None for that slug → treated as no action → skipped.
        let m = make_milestone("m1", vec!["ghost-f".to_string(), "real-f".to_string()]);
        let features = vec![
            make_feature("ghost-f", Phase::Implementation, false),
            make_feature("real-f", Phase::Implementation, false),
        ];
        // ghost-f intentionally omitted from next_actions
        let next_actions = actions(&[("real-f", ActionType::ImplementTask)]);

        let items = select_parallel_work(&[m], &features, &next_actions);
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0].kind, WorkItemKind::Feature { slug, .. } if slug == "real-f"));
    }

    #[test]
    fn verifying_milestone_with_zero_features_does_not_panic() {
        // A milestone with an empty feature list — compute_status yields Active
        // (the Verifying branch requires at least one non-archived feature).
        // No features means no actionable slot, but the function must not panic.
        let m = make_milestone("m1", vec![]);
        let features: Vec<Feature> = vec![];
        let next_actions: HashMap<String, ActionType> = HashMap::new();

        let items = select_parallel_work(&[m], &features, &next_actions);
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn verifying_first_then_active_second_both_appear() {
        // First milestone is Verifying → UAT slot.
        // Second milestone is Active → feature slot.
        // Both should appear in the result.
        let verifying = make_milestone("mv", vec!["released-f".to_string()]);
        let active = make_milestone("ma", vec!["work-f".to_string()]);

        let features = vec![
            Feature {
                phase: Phase::Released,
                ..make_feature("released-f", Phase::Released, false)
            },
            make_feature("work-f", Phase::Implementation, false),
        ];
        let next_actions = actions(&[
            ("released-f", ActionType::Done),
            ("work-f", ActionType::ImplementTask),
        ]);

        let items = select_parallel_work(&[verifying, active], &features, &next_actions);
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0].kind, WorkItemKind::Uat));
        assert!(matches!(&items[1].kind, WorkItemKind::Feature { slug, .. } if slug == "work-f"));
    }
}
