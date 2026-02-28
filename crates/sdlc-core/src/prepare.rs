use crate::classifier::{Classifier, EvalContext};
use crate::config::Config;
use crate::error::Result;
use crate::feature::Feature;
use crate::milestone::Milestone;
use crate::ponder::{PonderEntry, PonderStatus};
use crate::rules::default_rules;
use crate::state::State;
use crate::types::{ActionType, Phase};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;

// ---------------------------------------------------------------------------
// ProjectPhase
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum ProjectPhase {
    Idle,
    Pondering,
    Planning { milestone: String },
    Executing { milestone: String },
    Verifying { milestone: String },
}

impl ProjectPhase {
    pub fn milestone_slug(&self) -> Option<&str> {
        match self {
            ProjectPhase::Planning { milestone }
            | ProjectPhase::Executing { milestone }
            | ProjectPhase::Verifying { milestone } => Some(milestone),
            _ => None,
        }
    }
}

impl fmt::Display for ProjectPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectPhase::Idle => write!(f, "idle"),
            ProjectPhase::Pondering => write!(f, "pondering"),
            ProjectPhase::Planning { milestone } => write!(f, "planning ({milestone})"),
            ProjectPhase::Executing { milestone } => write!(f, "executing ({milestone})"),
            ProjectPhase::Verifying { milestone } => write!(f, "verifying ({milestone})"),
        }
    }
}

// ---------------------------------------------------------------------------
// Gap
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GapSeverity {
    Blocker,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gap {
    pub feature: String,
    pub severity: GapSeverity,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Wave
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveItem {
    pub slug: String,
    pub title: String,
    pub phase: Phase,
    pub action: String,
    pub needs_worktree: bool,
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wave {
    pub number: usize,
    pub label: String,
    pub items: Vec<WaveItem>,
    pub needs_worktrees: bool,
}

// ---------------------------------------------------------------------------
// BlockedFeature
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedFeature {
    pub slug: String,
    pub title: String,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// MilestoneProgress
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneProgress {
    pub total: usize,
    pub released: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub pending: usize,
}

// ---------------------------------------------------------------------------
// PrepareResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareResult {
    pub project_phase: ProjectPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone_progress: Option<MilestoneProgress>,
    pub gaps: Vec<Gap>,
    pub waves: Vec<Wave>,
    pub blocked: Vec<BlockedFeature>,
    pub next_commands: Vec<String>,
}

// ---------------------------------------------------------------------------
// project_phase()
// ---------------------------------------------------------------------------

/// Determine the current project lifecycle phase by inspecting milestones
/// and features. Read-only — no side effects.
pub fn project_phase(root: &Path) -> Result<ProjectPhase> {
    let state = State::load(root)?;

    for milestone_slug in &state.milestones {
        let milestone = match Milestone::load(root, milestone_slug) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // Skip explicitly terminal milestones
        if milestone.skipped_at.is_some() || milestone.released_at.is_some() {
            continue;
        }

        let features: Vec<Feature> = milestone
            .features
            .iter()
            .filter_map(|slug| Feature::load(root, slug).ok())
            .collect();

        let non_archived: Vec<&Feature> = features.iter().filter(|f| !f.archived).collect();

        // All non-archived features released → Verifying
        if !non_archived.is_empty() && non_archived.iter().all(|f| f.phase == Phase::Released) {
            return Ok(ProjectPhase::Verifying {
                milestone: milestone.slug,
            });
        }

        // Any feature past Planned → Executing
        if non_archived.iter().any(|f| f.phase > Phase::Planned) {
            return Ok(ProjectPhase::Executing {
                milestone: milestone.slug,
            });
        }

        // Otherwise → Planning
        return Ok(ProjectPhase::Planning {
            milestone: milestone.slug,
        });
    }

    // No active milestones — check ponders
    let ponders = PonderEntry::list(root)?;
    let has_active = ponders
        .iter()
        .any(|p| !matches!(p.status, PonderStatus::Committed | PonderStatus::Parked));

    if has_active {
        Ok(ProjectPhase::Pondering)
    } else {
        Ok(ProjectPhase::Idle)
    }
}

// ---------------------------------------------------------------------------
// prepare()
// ---------------------------------------------------------------------------

/// Survey a milestone: find gaps, organize features into parallelizable
/// waves. Read-only — no side effects.
///
/// If `milestone_slug` is `Some`, analyze that specific milestone regardless
/// of current project phase. If `None`, auto-detect from `project_phase()`.
pub fn prepare(root: &Path, milestone_slug: Option<&str>) -> Result<PrepareResult> {
    let phase = project_phase(root)?;

    // Determine which milestone to analyze
    let target_slug = match milestone_slug {
        Some(slug) => slug.to_string(),
        None => match &phase {
            ProjectPhase::Planning { milestone }
            | ProjectPhase::Executing { milestone }
            | ProjectPhase::Verifying { milestone } => milestone.clone(),
            // Idle/Pondering → nothing to analyze
            _ => {
                return Ok(PrepareResult {
                    project_phase: phase,
                    milestone: None,
                    milestone_title: None,
                    milestone_progress: None,
                    gaps: Vec::new(),
                    waves: Vec::new(),
                    blocked: Vec::new(),
                    next_commands: Vec::new(),
                });
            }
        },
    };

    let milestone = Milestone::load(root, &target_slug)?;
    let config = Config::load(root)?;
    let state = State::load(root)?;
    let classifier = Classifier::new(default_rules());

    // All feature slugs in project (for dep validation)
    let all_features = Feature::list(root)?;
    let all_slugs: HashSet<String> = all_features.iter().map(|f| f.slug.clone()).collect();

    // Load, classify, and gap-check each feature in the milestone
    let mut gaps = Vec::new();
    let mut features: HashMap<String, ClassifiedFeature> = HashMap::new();

    for feature_slug in &milestone.features {
        let feature = match Feature::load(root, feature_slug) {
            Ok(f) => f,
            Err(_) => {
                gaps.push(Gap {
                    feature: feature_slug.clone(),
                    severity: GapSeverity::Blocker,
                    message: format!(
                        "Feature '{}' listed in milestone but not found",
                        feature_slug
                    ),
                });
                continue;
            }
        };

        if feature.archived {
            continue;
        }

        // Gap: missing description
        if feature.description.is_none() {
            gaps.push(Gap {
                feature: feature_slug.clone(),
                severity: GapSeverity::Warning,
                message: format!("Feature '{}' has no description", feature_slug),
            });
        }

        // Gap: broken dependency references
        for dep in &feature.dependencies {
            if !all_slugs.contains(dep) {
                gaps.push(Gap {
                    feature: feature_slug.clone(),
                    severity: GapSeverity::Blocker,
                    message: format!(
                        "Feature '{}' depends on '{}' which does not exist",
                        feature_slug, dep
                    ),
                });
            }
        }

        // Classify
        let ctx = EvalContext {
            feature: &feature,
            state: &state,
            config: &config,
            root,
        };
        let classification = classifier.classify(&ctx);

        features.insert(
            feature_slug.clone(),
            ClassifiedFeature {
                action: classification.action,
                action_str: classification.action.to_string(),
                feature,
            },
        );
    }

    // -- Partition features --
    let mut completed: HashSet<String> = HashSet::new();
    let mut hitl_blocked: HashSet<String> = HashSet::new();

    for (slug, info) in &features {
        if info.feature.phase == Phase::Released || info.action == ActionType::Done {
            completed.insert(slug.clone());
        } else if matches!(
            info.action,
            ActionType::WaitForApproval | ActionType::UnblockDependency
        ) {
            hitl_blocked.insert(slug.clone());
        }
    }

    // Transitive blocking: features depending on HITL-blocked features
    let mut blocked_set = hitl_blocked.clone();
    loop {
        let mut changed = false;
        for (slug, info) in &features {
            if completed.contains(slug) || blocked_set.contains(slug) {
                continue;
            }
            let dep_blocked = info
                .feature
                .dependencies
                .iter()
                .any(|dep| blocked_set.contains(dep) && features.contains_key(dep));
            if dep_blocked {
                blocked_set.insert(slug.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // Build blocked list
    let mut blocked_features: Vec<BlockedFeature> = Vec::new();
    for slug in &blocked_set {
        if let Some(info) = features.get(slug) {
            let reason = if hitl_blocked.contains(slug) {
                match info.action {
                    ActionType::WaitForApproval => "Waiting for human approval".to_string(),
                    ActionType::UnblockDependency => "Blocked by unresolved dependency".to_string(),
                    _ => "Blocked".to_string(),
                }
            } else {
                let blocking_dep = info
                    .feature
                    .dependencies
                    .iter()
                    .find(|d| blocked_set.contains(*d))
                    .cloned()
                    .unwrap_or_default();
                format!("Depends on blocked feature '{}'", blocking_dep)
            };
            blocked_features.push(BlockedFeature {
                slug: slug.clone(),
                title: info.feature.title.clone(),
                reason,
            });
        }
    }
    blocked_features.sort_by(|a, b| a.slug.cmp(&b.slug));

    // Gap: HITL features blocking dependents
    for slug in &hitl_blocked {
        let dependent_count = features
            .values()
            .filter(|info| {
                info.feature.dependencies.contains(slug) && !completed.contains(&info.feature.slug)
            })
            .count();
        if dependent_count > 0 {
            if let Some(info) = features.get(slug) {
                gaps.push(Gap {
                    feature: slug.clone(),
                    severity: GapSeverity::Info,
                    message: format!(
                        "Feature '{}' is at a human gate and blocking {} dependent feature(s)",
                        info.feature.title, dependent_count
                    ),
                });
            }
        }
    }

    // -- Wave computation (Kahn's algorithm) --

    let candidates: HashSet<String> = features
        .keys()
        .filter(|slug| !completed.contains(*slug) && !blocked_set.contains(*slug))
        .cloned()
        .collect();

    // Adjacency list: dep → dependents (both must be candidates)
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    for slug in &candidates {
        adj.entry(slug.clone()).or_default();
        in_degree.entry(slug.clone()).or_insert(0);
    }

    for slug in &candidates {
        if let Some(info) = features.get(slug) {
            for dep in &info.feature.dependencies {
                if candidates.contains(dep) {
                    adj.entry(dep.clone()).or_default().push(slug.clone());
                    *in_degree.entry(slug.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    // BFS topological sort — group by wave level
    let mut wave_groups: Vec<Vec<String>> = Vec::new();
    let mut remaining = candidates.len();

    let mut current: Vec<String> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(slug, _)| slug.clone())
        .collect();
    current.sort(); // deterministic ordering

    while !current.is_empty() {
        remaining -= current.len();

        let mut next: Vec<String> = Vec::new();
        for slug in &current {
            if let Some(dependents) = adj.get(slug) {
                for dependent in dependents {
                    if let Some(deg) = in_degree.get_mut(dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            next.push(dependent.clone());
                        }
                    }
                }
            }
        }

        wave_groups.push(current);
        next.sort();
        current = next;
    }

    // Cycle detection
    if remaining > 0 {
        let processed: HashSet<String> = wave_groups.iter().flatten().cloned().collect();
        let cycled: Vec<String> = candidates
            .iter()
            .filter(|s| !processed.contains(*s))
            .cloned()
            .collect();
        gaps.push(Gap {
            feature: cycled.first().cloned().unwrap_or_default(),
            severity: GapSeverity::Blocker,
            message: format!(
                "Dependency cycle detected among features: {}",
                cycled.join(", ")
            ),
        });
    }

    // Build Wave structs
    let mut assigned: HashSet<String> = HashSet::new();
    let wave_structs: Vec<Wave> = wave_groups
        .into_iter()
        .enumerate()
        .map(|(i, slugs)| {
            let items: Vec<WaveItem> = slugs
                .iter()
                .filter_map(|slug| {
                    let info = features.get(slug)?;
                    let needs_worktree = info.action.is_heavy();

                    // blocked_by = deps that were in earlier waves
                    let blocked_by: Vec<String> = info
                        .feature
                        .dependencies
                        .iter()
                        .filter(|dep| assigned.contains(*dep))
                        .cloned()
                        .collect();

                    Some(WaveItem {
                        slug: slug.clone(),
                        title: info.feature.title.clone(),
                        phase: info.feature.phase,
                        action: info.action_str.clone(),
                        needs_worktree,
                        blocked_by,
                    })
                })
                .collect();

            // Mark this wave's features as assigned
            for slug in &slugs {
                assigned.insert(slug.clone());
            }

            let needs_worktrees = items.iter().any(|item| item.needs_worktree);
            let label = wave_label(&items);

            Wave {
                number: i + 1,
                label,
                items,
                needs_worktrees,
            }
        })
        .collect();

    // -- Progress --
    let mut released_count = 0usize;
    let mut blocked_count = 0usize;
    let mut in_progress_count = 0usize;
    let mut pending_count = 0usize;

    for (slug, info) in &features {
        if info.feature.phase == Phase::Released || info.action == ActionType::Done {
            released_count += 1;
        } else if blocked_set.contains(slug) {
            blocked_count += 1;
        } else if info.feature.phase > Phase::Draft {
            in_progress_count += 1;
        } else {
            pending_count += 1;
        }
    }

    let progress = MilestoneProgress {
        total: features.len(),
        released: released_count,
        in_progress: in_progress_count,
        blocked: blocked_count,
        pending: pending_count,
    };

    // -- Next commands --
    // When the milestone hasn't started yet (nothing released or in progress)
    // and all Wave-1 features share the same planning action, suggest
    // /sdlc-prepare <slug> for holistic readiness analysis instead of
    // N individual /sdlc-run commands.
    let next_commands: Vec<String> = match wave_structs.first() {
        None => Vec::new(),
        Some(wave1) => {
            let milestone_fresh = released_count == 0 && in_progress_count == 0;
            let uniform_action = {
                let actions: std::collections::HashSet<&str> =
                    wave1.items.iter().map(|i| i.action.as_str()).collect();
                actions.len() == 1
            };
            if milestone_fresh && uniform_action && wave1.items.len() > 1 {
                vec![format!("/sdlc-prepare {}", milestone.slug)]
            } else {
                wave1
                    .items
                    .iter()
                    .map(|item| format!("/sdlc-run {}", item.slug))
                    .collect()
            }
        }
    };

    // Sort gaps by severity (blockers first)
    gaps.sort_by_key(|g| match g.severity {
        GapSeverity::Blocker => 0,
        GapSeverity::Warning => 1,
        GapSeverity::Info => 2,
    });

    Ok(PrepareResult {
        project_phase: phase,
        milestone: Some(milestone.slug),
        milestone_title: Some(milestone.title),
        milestone_progress: Some(progress),
        gaps,
        waves: wave_structs,
        blocked: blocked_features,
        next_commands,
    })
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

struct ClassifiedFeature {
    feature: Feature,
    action: ActionType,
    action_str: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn wave_label(items: &[WaveItem]) -> String {
    if items.is_empty() {
        return "Empty".to_string();
    }

    let mut planning = 0usize;
    let mut implementation = 0usize;
    let mut review = 0usize;

    for item in items {
        match item.phase {
            Phase::Draft | Phase::Specified | Phase::Planned | Phase::Ready => planning += 1,
            Phase::Implementation => implementation += 1,
            Phase::Review | Phase::Audit | Phase::Qa | Phase::Merge => review += 1,
            Phase::Released => {} // shouldn't be in waves
        }
    }

    let max = planning.max(implementation).max(review);
    if max == 0 {
        return "Mixed".to_string();
    }

    if planning == max && implementation < max && review < max {
        "Planning".to_string()
    } else if implementation == max && planning < max && review < max {
        "Implementation".to_string()
    } else if review == max && planning < max && implementation < max {
        "Review".to_string()
    } else {
        "Mixed".to_string()
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
    use crate::ponder::PonderEntry;
    use crate::state::State;
    use crate::task::Task;
    use crate::types::Phase;
    use tempfile::TempDir;

    fn setup(dir: &TempDir) -> State {
        let root = dir.path();
        std::fs::create_dir_all(root.join(".sdlc/features")).unwrap();
        std::fs::create_dir_all(root.join(".sdlc/milestones")).unwrap();
        std::fs::create_dir_all(root.join(".sdlc/roadmap")).unwrap();
        Config::new("test").save(root).unwrap();
        let state = State::new("test");
        state.save(root).unwrap();
        state
    }

    fn add_feature(dir: &TempDir, slug: &str) -> Feature {
        let f = Feature::create(dir.path(), slug, slug).unwrap();
        let mut state = State::load(dir.path()).unwrap();
        state.add_active_feature(slug);
        state.save(dir.path()).unwrap();
        f
    }

    fn add_milestone(dir: &TempDir, slug: &str, feature_slugs: &[&str]) -> Milestone {
        let mut m = Milestone::create(dir.path(), slug, slug).unwrap();
        for &fs in feature_slugs {
            m.add_feature(fs);
        }
        m.save(dir.path()).unwrap();
        let mut state = State::load(dir.path()).unwrap();
        state.add_milestone(slug);
        state.save(dir.path()).unwrap();
        m
    }

    // -----------------------------------------------------------------------
    // project_phase tests
    // -----------------------------------------------------------------------

    #[test]
    fn project_phase_idle() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let phase = project_phase(dir.path()).unwrap();
        assert_eq!(phase, ProjectPhase::Idle);
    }

    #[test]
    fn project_phase_pondering() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // Create an active ponder (Exploring status)
        PonderEntry::create(dir.path(), "my-idea", "My Idea").unwrap();

        let phase = project_phase(dir.path()).unwrap();
        assert_eq!(phase, ProjectPhase::Pondering);
    }

    #[test]
    fn project_phase_planning() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        add_feature(&dir, "feat-a");
        add_feature(&dir, "feat-b");
        add_milestone(&dir, "v1", &["feat-a", "feat-b"]);

        let phase = project_phase(dir.path()).unwrap();
        assert_eq!(
            phase,
            ProjectPhase::Planning {
                milestone: "v1".to_string()
            }
        );
    }

    #[test]
    fn project_phase_executing() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        add_feature(&dir, "feat-a");
        add_feature(&dir, "feat-b");

        // Move feat-a past Planned
        let mut f = Feature::load(dir.path(), "feat-a").unwrap();
        f.phase = Phase::Implementation;
        f.save(dir.path()).unwrap();

        add_milestone(&dir, "v1", &["feat-a", "feat-b"]);

        let phase = project_phase(dir.path()).unwrap();
        assert_eq!(
            phase,
            ProjectPhase::Executing {
                milestone: "v1".to_string()
            }
        );
    }

    #[test]
    fn project_phase_verifying() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        add_feature(&dir, "feat-a");
        add_feature(&dir, "feat-b");

        // Release both features
        let mut fa = Feature::load(dir.path(), "feat-a").unwrap();
        fa.phase = Phase::Released;
        fa.save(dir.path()).unwrap();

        let mut fb = Feature::load(dir.path(), "feat-b").unwrap();
        fb.phase = Phase::Released;
        fb.save(dir.path()).unwrap();

        // Milestone not explicitly released_at — features are released
        add_milestone(&dir, "v1", &["feat-a", "feat-b"]);

        let phase = project_phase(dir.path()).unwrap();
        assert_eq!(
            phase,
            ProjectPhase::Verifying {
                milestone: "v1".to_string()
            }
        );
    }

    // -----------------------------------------------------------------------
    // prepare tests
    // -----------------------------------------------------------------------

    #[test]
    fn prepare_empty_milestone() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        add_milestone(&dir, "v1", &[]);

        let result = prepare(dir.path(), Some("v1")).unwrap();
        assert!(result.waves.is_empty());
        assert_eq!(result.milestone.as_deref(), Some("v1"));
        assert_eq!(result.milestone_progress.as_ref().unwrap().total, 0);
    }

    #[test]
    fn prepare_no_deps() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        add_feature(&dir, "feat-a");
        add_feature(&dir, "feat-b");
        add_feature(&dir, "feat-c");
        add_milestone(&dir, "v1", &["feat-a", "feat-b", "feat-c"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();
        assert_eq!(result.waves.len(), 1);
        assert_eq!(result.waves[0].items.len(), 3);
        // All in Wave 1 — no deps
        assert_eq!(result.waves[0].number, 1);
        for item in &result.waves[0].items {
            assert!(item.blocked_by.is_empty());
        }
    }

    #[test]
    fn prepare_linear_deps() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // A → B → C (C depends on B, B depends on A)
        add_feature(&dir, "feat-a");
        add_feature(&dir, "feat-b");
        add_feature(&dir, "feat-c");

        let mut fb = Feature::load(dir.path(), "feat-b").unwrap();
        fb.dependencies = vec!["feat-a".to_string()];
        fb.save(dir.path()).unwrap();

        let mut fc = Feature::load(dir.path(), "feat-c").unwrap();
        fc.dependencies = vec!["feat-b".to_string()];
        fc.save(dir.path()).unwrap();

        add_milestone(&dir, "v1", &["feat-a", "feat-b", "feat-c"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();
        assert_eq!(result.waves.len(), 3);
        assert_eq!(result.waves[0].items.len(), 1);
        assert_eq!(result.waves[0].items[0].slug, "feat-a");
        assert_eq!(result.waves[1].items.len(), 1);
        assert_eq!(result.waves[1].items[0].slug, "feat-b");
        assert_eq!(result.waves[2].items.len(), 1);
        assert_eq!(result.waves[2].items[0].slug, "feat-c");

        // Wave 2 blocked_by Wave 1
        assert_eq!(result.waves[1].items[0].blocked_by, vec!["feat-a"]);
        // Wave 3 blocked_by Wave 2
        assert_eq!(result.waves[2].items[0].blocked_by, vec!["feat-b"]);
    }

    #[test]
    fn prepare_diamond_deps() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // A → B, A → C, B+C → D
        add_feature(&dir, "feat-a");
        add_feature(&dir, "feat-b");
        add_feature(&dir, "feat-c");
        add_feature(&dir, "feat-d");

        let mut fb = Feature::load(dir.path(), "feat-b").unwrap();
        fb.dependencies = vec!["feat-a".to_string()];
        fb.save(dir.path()).unwrap();

        let mut fc = Feature::load(dir.path(), "feat-c").unwrap();
        fc.dependencies = vec!["feat-a".to_string()];
        fc.save(dir.path()).unwrap();

        let mut fd = Feature::load(dir.path(), "feat-d").unwrap();
        fd.dependencies = vec!["feat-b".to_string(), "feat-c".to_string()];
        fd.save(dir.path()).unwrap();

        add_milestone(&dir, "v1", &["feat-a", "feat-b", "feat-c", "feat-d"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();
        assert_eq!(result.waves.len(), 3);

        // Wave 1: A (no deps)
        assert_eq!(result.waves[0].items.len(), 1);
        assert_eq!(result.waves[0].items[0].slug, "feat-a");

        // Wave 2: B, C (both depend on A)
        assert_eq!(result.waves[1].items.len(), 2);
        let wave2_slugs: Vec<&str> = result.waves[1]
            .items
            .iter()
            .map(|i| i.slug.as_str())
            .collect();
        assert!(wave2_slugs.contains(&"feat-b"));
        assert!(wave2_slugs.contains(&"feat-c"));

        // Wave 3: D (depends on B and C)
        assert_eq!(result.waves[2].items.len(), 1);
        assert_eq!(result.waves[2].items[0].slug, "feat-d");
    }

    #[test]
    fn prepare_cycle_detection() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // A → B → A (cycle)
        add_feature(&dir, "feat-a");
        add_feature(&dir, "feat-b");

        let mut fa = Feature::load(dir.path(), "feat-a").unwrap();
        fa.dependencies = vec!["feat-b".to_string()];
        fa.save(dir.path()).unwrap();

        let mut fb = Feature::load(dir.path(), "feat-b").unwrap();
        fb.dependencies = vec!["feat-a".to_string()];
        fb.save(dir.path()).unwrap();

        add_milestone(&dir, "v1", &["feat-a", "feat-b"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();

        // Should have a blocker gap for cycle
        let cycle_gaps: Vec<&Gap> = result
            .gaps
            .iter()
            .filter(|g| g.severity == GapSeverity::Blocker && g.message.contains("cycle"))
            .collect();
        assert!(!cycle_gaps.is_empty());

        // No waves produced (all features in cycle)
        assert!(result.waves.is_empty());
    }

    #[test]
    fn prepare_broken_dep_ref() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        add_feature(&dir, "feat-a");

        let mut fa = Feature::load(dir.path(), "feat-a").unwrap();
        fa.dependencies = vec!["nonexistent".to_string()];
        fa.save(dir.path()).unwrap();

        add_milestone(&dir, "v1", &["feat-a"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();

        let broken_gaps: Vec<&Gap> = result
            .gaps
            .iter()
            .filter(|g| g.severity == GapSeverity::Blocker && g.message.contains("does not exist"))
            .collect();
        assert_eq!(broken_gaps.len(), 1);
        assert!(broken_gaps[0].message.contains("nonexistent"));
    }

    #[test]
    fn prepare_missing_description() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // Feature without description
        add_feature(&dir, "feat-a");

        // Feature with description
        Feature::create_with_description(dir.path(), "feat-b", "B", Some("Has a desc".into()))
            .unwrap();
        let mut state = State::load(dir.path()).unwrap();
        state.add_active_feature("feat-b");
        state.save(dir.path()).unwrap();

        add_milestone(&dir, "v1", &["feat-a", "feat-b"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();

        let desc_gaps: Vec<&Gap> = result
            .gaps
            .iter()
            .filter(|g| g.severity == GapSeverity::Warning && g.message.contains("no description"))
            .collect();
        assert_eq!(desc_gaps.len(), 1);
        assert_eq!(desc_gaps[0].feature, "feat-a");
    }

    #[test]
    fn prepare_hitl_excluded() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // Normal feature
        add_feature(&dir, "feat-a");

        // Feature with blockers → triggers UnblockDependency
        add_feature(&dir, "feat-blocked");
        let mut fb = Feature::load(dir.path(), "feat-blocked").unwrap();
        fb.blockers = vec!["waiting-on-api".to_string()];
        fb.save(dir.path()).unwrap();

        add_milestone(&dir, "v1", &["feat-a", "feat-blocked"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();

        // feat-blocked should be in blocked list, not in waves
        assert!(result.blocked.iter().any(|b| b.slug == "feat-blocked"));

        // feat-a should be in waves
        let wave_slugs: Vec<&str> = result
            .waves
            .iter()
            .flat_map(|w| w.items.iter().map(|i| i.slug.as_str()))
            .collect();
        assert!(wave_slugs.contains(&"feat-a"));
        assert!(!wave_slugs.contains(&"feat-blocked"));
    }

    #[test]
    fn prepare_implementation_needs_worktree() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // Feature at Implementation phase with pending tasks
        add_feature(&dir, "feat-impl");
        let mut f = Feature::load(dir.path(), "feat-impl").unwrap();
        f.phase = Phase::Implementation;
        // Add a pending task so the classifier returns ImplementTask
        f.tasks.push(Task::new("t1", "Do the thing"));
        f.save(dir.path()).unwrap();

        // Feature at Draft phase
        add_feature(&dir, "feat-draft");

        add_milestone(&dir, "v1", &["feat-impl", "feat-draft"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();

        let impl_item = result
            .waves
            .iter()
            .flat_map(|w| &w.items)
            .find(|i| i.slug == "feat-impl");
        assert!(impl_item.is_some());
        assert!(impl_item.unwrap().needs_worktree);

        let draft_item = result
            .waves
            .iter()
            .flat_map(|w| &w.items)
            .find(|i| i.slug == "feat-draft");
        assert!(draft_item.is_some());
        assert!(!draft_item.unwrap().needs_worktree);
    }

    #[test]
    fn prepare_idle_returns_empty() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // No milestone specified, no active milestones → Idle
        let result = prepare(dir.path(), None).unwrap();
        assert_eq!(result.project_phase, ProjectPhase::Idle);
        assert!(result.waves.is_empty());
        assert!(result.milestone.is_none());
    }

    #[test]
    fn prepare_next_commands() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // Fresh milestone: all draft, same action → suggest /sdlc-prepare
        add_feature(&dir, "feat-a");
        add_feature(&dir, "feat-b");
        add_milestone(&dir, "v1", &["feat-a", "feat-b"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();
        assert_eq!(result.next_commands.len(), 1);
        assert_eq!(result.next_commands[0], "/sdlc-prepare v1");
    }

    #[test]
    fn prepare_next_commands_in_progress_falls_back_to_run() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // One feature past Draft (in_progress_count > 0) → individual /sdlc-run
        add_feature(&dir, "feat-a");
        let mut fa = Feature::load(dir.path(), "feat-a").unwrap();
        fa.phase = Phase::Specified;
        fa.save(dir.path()).unwrap();

        add_feature(&dir, "feat-b");
        add_milestone(&dir, "v1", &["feat-a", "feat-b"]);

        let result = prepare(dir.path(), Some("v1")).unwrap();
        assert!(!result.next_commands.is_empty());
        assert!(result
            .next_commands
            .iter()
            .all(|c| c.starts_with("/sdlc-run")));
    }

    #[test]
    fn prepare_progress() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        // Released feature
        add_feature(&dir, "released");
        let mut fr = Feature::load(dir.path(), "released").unwrap();
        fr.phase = Phase::Released;
        fr.save(dir.path()).unwrap();

        // In-progress feature (past Draft)
        add_feature(&dir, "in-progress");
        let mut fip = Feature::load(dir.path(), "in-progress").unwrap();
        fip.phase = Phase::Specified;
        fip.save(dir.path()).unwrap();

        // Blocked feature
        add_feature(&dir, "blocked");
        let mut fb = Feature::load(dir.path(), "blocked").unwrap();
        fb.blockers = vec!["reason".to_string()];
        fb.save(dir.path()).unwrap();

        // Pending feature (Draft)
        add_feature(&dir, "pending");

        add_milestone(
            &dir,
            "v1",
            &["released", "in-progress", "blocked", "pending"],
        );

        let result = prepare(dir.path(), Some("v1")).unwrap();
        let progress = result.milestone_progress.unwrap();
        assert_eq!(progress.total, 4);
        assert_eq!(progress.released, 1);
        assert_eq!(progress.in_progress, 1);
        assert_eq!(progress.blocked, 1);
        assert_eq!(progress.pending, 1);
    }
}
