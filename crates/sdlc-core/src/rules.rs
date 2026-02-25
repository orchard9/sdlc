use crate::classifier::{EvalContext, Rule};
use crate::comment::CommentFlag;
use crate::types::{ActionType, ArtifactStatus, ArtifactType, Phase, TaskStatus};

// ---------------------------------------------------------------------------
// Helper macros for concise rule definitions
// ---------------------------------------------------------------------------

macro_rules! rule {
    (
        id: $id:expr,
        condition: $cond:expr,
        action: $action:expr,
        message: $msg:expr,
        next_command: $cmd:expr
        $(, output_path: $path:expr)?
        $(, transition_to: $trans:expr)?
        $(, task_id: $tid:expr)?
    ) => {
        Rule {
            id: $id,
            condition: $cond,
            action: $action,
            message: $msg,
            next_command: $cmd,
            output_path: {
                #[allow(unused_assignments, unused_mut)]
                let mut v: Option<fn(&EvalContext) -> String> = None;
                $(v = Some($path);)?
                v
            },
            transition_to: {
                #[allow(unused_assignments, unused_mut)]
                let mut v: Option<Phase> = None;
                $(v = Some($trans);)?
                v
            },
            task_id: {
                #[allow(unused_assignments, unused_mut)]
                let mut v: Option<fn(&EvalContext) -> String> = None;
                $(v = Some($tid);)?
                v
            },
        }
    };
}

// ---------------------------------------------------------------------------
// Condition helpers
// ---------------------------------------------------------------------------

fn is_blocked(ctx: &EvalContext) -> bool {
    ctx.feature.is_blocked()
}

fn artifact_missing(ctx: &EvalContext, t: ArtifactType) -> bool {
    ctx.feature
        .artifact(t)
        .map(|a| matches!(a.status, ArtifactStatus::Missing))
        .unwrap_or(true)
}

fn artifact_needs_approval(ctx: &EvalContext, t: ArtifactType) -> bool {
    ctx.feature
        .artifact(t)
        .map(|a| matches!(a.status, ArtifactStatus::Draft | ArtifactStatus::NeedsFix))
        .unwrap_or(false)
}

fn artifact_approved(ctx: &EvalContext, t: ArtifactType) -> bool {
    ctx.feature
        .artifact(t)
        .map(|a| a.is_approved())
        .unwrap_or(false)
}

fn artifact_rejected(ctx: &EvalContext, t: ArtifactType) -> bool {
    ctx.feature
        .artifact(t)
        .map(|a| matches!(a.status, ArtifactStatus::Rejected))
        .unwrap_or(false)
}

fn in_phase(ctx: &EvalContext, p: Phase) -> bool {
    ctx.feature.phase == p
}

fn has_pending_task(ctx: &EvalContext) -> bool {
    ctx.feature
        .tasks
        .iter()
        .any(|t| matches!(t.status, TaskStatus::Pending | TaskStatus::InProgress))
}

fn feature_dir(ctx: &EvalContext) -> String {
    format!(".sdlc/features/{}", ctx.feature.slug)
}

fn has_blocker_comments(ctx: &EvalContext) -> bool {
    ctx.feature.comments.iter().any(|c| {
        matches!(
            &c.flag,
            Some(CommentFlag::Blocker) | Some(CommentFlag::Question)
        )
    })
}

fn blocker_comments_message(ctx: &EvalContext) -> String {
    let blockers: Vec<_> = ctx
        .feature
        .comments
        .iter()
        .filter(|c| {
            matches!(
                &c.flag,
                Some(CommentFlag::Blocker) | Some(CommentFlag::Question)
            )
        })
        .collect();
    let details: Vec<String> = blockers
        .iter()
        .map(|c| format!("[{}] {}", c.id, c.body))
        .collect();
    format!(
        "Feature '{}' has {} unresolved blocker comment(s): {}",
        ctx.feature.slug,
        blockers.len(),
        details.join("; ")
    )
}

// ---------------------------------------------------------------------------
// Default rules (priority-ordered)
// ---------------------------------------------------------------------------

pub fn default_rules() -> Vec<Rule> {
    vec![
        // 1. Blocked by dependency — must be resolved first
        rule! {
            id: "blocked_dependency",
            condition: is_blocked,
            action: ActionType::UnblockDependency,
            message: |ctx| format!(
                "Feature '{}' is blocked: {}",
                ctx.feature.slug,
                ctx.feature.blockers.join(", ")
            ),
            next_command: |_| String::new()
        },
        // 2. Blocker-flagged comments block progress until resolved
        rule! {
            id: "blocker_comment",
            condition: |ctx| !is_blocked(ctx) && has_blocker_comments(ctx),
            action: ActionType::WaitForApproval,
            message: blocker_comments_message,
            next_command: |_| String::new()
        },
        // 3. Draft — no spec exists
        rule! {
            id: "needs_spec",
            condition: |ctx| in_phase(ctx, Phase::Draft) && artifact_missing(ctx, ArtifactType::Spec),
            action: ActionType::CreateSpec,
            message: |ctx| {
                let mut msg = format!(
                    "No spec exists. Write the feature specification for '{}' ({}).",
                    ctx.feature.slug, ctx.feature.title
                );
                if let Some(ref desc) = ctx.feature.description {
                    msg.push_str(&format!("\nDescription: {desc}"));
                }
                msg
            },
            next_command: |ctx| format!("/spec-feature {}", ctx.feature.slug),
            output_path: |ctx| format!("{}/spec.md", feature_dir(ctx))
        },
        // 3. Draft — spec written, awaiting approval
        rule! {
            id: "spec_needs_approval",
            condition: |ctx| in_phase(ctx, Phase::Draft) && artifact_needs_approval(ctx, ArtifactType::Spec),
            action: ActionType::ApproveSpec,
            message: |ctx| format!("Spec for '{}' is ready for review.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc artifact approve {} spec", ctx.feature.slug)
        },
        // 4. Draft — spec rejected, needs rewrite
        rule! {
            id: "spec_rejected",
            condition: |ctx| in_phase(ctx, Phase::Draft) && artifact_rejected(ctx, ArtifactType::Spec),
            action: ActionType::CreateSpec,
            message: |ctx| {
                let mut msg = format!(
                    "Spec for '{}' ({}) was rejected. Rewrite it.",
                    ctx.feature.slug, ctx.feature.title
                );
                if let Some(ref desc) = ctx.feature.description {
                    msg.push_str(&format!("\nDescription: {desc}"));
                }
                msg
            },
            next_command: |ctx| format!("/spec-feature {}", ctx.feature.slug),
            output_path: |ctx| format!("{}/spec.md", feature_dir(ctx))
        },
        // 5. Draft — spec approved, transition to Specified
        rule! {
            id: "spec_approved",
            condition: |ctx| in_phase(ctx, Phase::Draft) && artifact_approved(ctx, ArtifactType::Spec),
            action: ActionType::ApproveSpec,
            message: |ctx| format!("Spec approved. Transitioning '{}' to specified.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc feature transition {} specified", ctx.feature.slug),
            transition_to: Phase::Specified
        },
        // 6. Specified — no design
        rule! {
            id: "needs_design",
            condition: |ctx| in_phase(ctx, Phase::Specified) && artifact_missing(ctx, ArtifactType::Design),
            action: ActionType::CreateDesign,
            message: |ctx| format!("No design exists. Write the design document for '{}'.", ctx.feature.slug),
            next_command: |ctx| format!("/design-feature {}", ctx.feature.slug),
            output_path: |ctx| format!("{}/design.md", feature_dir(ctx))
        },
        // 7. Specified — design needs approval
        rule! {
            id: "design_needs_approval",
            condition: |ctx| in_phase(ctx, Phase::Specified) && artifact_needs_approval(ctx, ArtifactType::Design),
            action: ActionType::ApproveDesign,
            message: |ctx| format!("Design for '{}' is ready for review.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc artifact approve {} design", ctx.feature.slug)
        },
        // 8. Specified — design rejected
        rule! {
            id: "design_rejected",
            condition: |ctx| in_phase(ctx, Phase::Specified) && artifact_rejected(ctx, ArtifactType::Design),
            action: ActionType::CreateDesign,
            message: |ctx| format!("Design for '{}' was rejected. Rewrite it.", ctx.feature.slug),
            next_command: |ctx| format!("/design-feature {}", ctx.feature.slug),
            output_path: |ctx| format!("{}/design.md", feature_dir(ctx))
        },
        // 9. Specified — design approved, no tasks
        rule! {
            id: "needs_tasks",
            condition: |ctx| in_phase(ctx, Phase::Specified)
                && artifact_approved(ctx, ArtifactType::Design)
                && artifact_missing(ctx, ArtifactType::Tasks),
            action: ActionType::CreateTasks,
            message: |ctx| format!("Design approved. Write the task breakdown for '{}'.", ctx.feature.slug),
            next_command: |ctx| format!("/tasks-feature {}", ctx.feature.slug),
            output_path: |ctx| format!("{}/tasks.md", feature_dir(ctx))
        },
        // 10. Specified — tasks exist, no QA plan
        rule! {
            id: "needs_qa_plan",
            condition: |ctx| in_phase(ctx, Phase::Specified)
                && artifact_approved(ctx, ArtifactType::Design)
                && !artifact_missing(ctx, ArtifactType::Tasks)
                && artifact_missing(ctx, ArtifactType::QaPlan),
            action: ActionType::CreateQaPlan,
            message: |ctx| format!("Write the QA plan for '{}'.", ctx.feature.slug),
            next_command: |ctx| format!("/qa-plan {}", ctx.feature.slug),
            output_path: |ctx| format!("{}/qa-plan.md", feature_dir(ctx))
        },
        // 11. Specified — all planning artifacts approved, transition to Planned
        rule! {
            id: "ready_to_plan",
            condition: |ctx| in_phase(ctx, Phase::Specified)
                && artifact_approved(ctx, ArtifactType::Design)
                && artifact_approved(ctx, ArtifactType::Tasks)
                && artifact_approved(ctx, ArtifactType::QaPlan),
            action: ActionType::WaitForApproval,
            message: |ctx| format!("All planning artifacts approved. Transitioning '{}' to planned.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc feature transition {} planned", ctx.feature.slug),
            transition_to: Phase::Planned
        },
        // 12. Planned — transition to Ready (no further gates)
        rule! {
            id: "planned_to_ready",
            condition: |ctx| in_phase(ctx, Phase::Planned),
            action: ActionType::ImplementTask,
            message: |ctx| format!("Feature '{}' is planned. Marking ready for implementation.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc feature transition {} ready", ctx.feature.slug),
            transition_to: Phase::Ready
        },
        // 13. Ready — has pending tasks to implement
        rule! {
            id: "implement_task",
            condition: |ctx| in_phase(ctx, Phase::Ready) && has_pending_task(ctx),
            action: ActionType::ImplementTask,
            message: |ctx| format!("Implement the next task for '{}'.", ctx.feature.slug),
            next_command: |ctx| format!("/implement {}", ctx.feature.slug),
            task_id: |ctx| {
                ctx.feature
                    .tasks
                    .iter()
                    .find(|t| matches!(t.status, TaskStatus::Pending | TaskStatus::InProgress))
                    .map(|t| t.id.clone())
                    .unwrap_or_default()
            }
        },
        // 14. Ready — all tasks done, no review
        rule! {
            id: "needs_review",
            condition: |ctx| in_phase(ctx, Phase::Ready)
                && !has_pending_task(ctx)
                && artifact_missing(ctx, ArtifactType::Review),
            action: ActionType::CreateReview,
            message: |ctx| format!("All tasks complete. Write the code review for '{}'.", ctx.feature.slug),
            next_command: |ctx| format!("/review-feature {}", ctx.feature.slug),
            output_path: |ctx| format!("{}/review.md", feature_dir(ctx)),
            transition_to: Phase::Review
        },
        // 15. Review — review needs approval
        rule! {
            id: "review_needs_approval",
            condition: |ctx| in_phase(ctx, Phase::Review) && artifact_needs_approval(ctx, ArtifactType::Review),
            action: ActionType::ApproveReview,
            message: |ctx| format!("Review for '{}' is ready for approval.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc artifact approve {} review", ctx.feature.slug)
        },
        // 16. Review — review rejected, fix issues
        rule! {
            id: "fix_review_issues",
            condition: |ctx| in_phase(ctx, Phase::Review) && artifact_rejected(ctx, ArtifactType::Review),
            action: ActionType::FixReviewIssues,
            message: |ctx| format!("Review for '{}' failed. Fix the issues.", ctx.feature.slug),
            next_command: |ctx| format!("/fix-review {}", ctx.feature.slug)
        },
        // 17. Review — approved, transition to Audit
        rule! {
            id: "review_approved",
            condition: |ctx| in_phase(ctx, Phase::Review) && artifact_approved(ctx, ArtifactType::Review),
            action: ActionType::CreateAudit,
            message: |ctx| format!("Review approved. Transitioning '{}' to audit.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc feature transition {} audit", ctx.feature.slug),
            transition_to: Phase::Audit
        },
        // 18. Audit — no audit
        rule! {
            id: "needs_audit",
            condition: |ctx| in_phase(ctx, Phase::Audit) && artifact_missing(ctx, ArtifactType::Audit),
            action: ActionType::CreateAudit,
            message: |ctx| format!("Write the security audit for '{}'.", ctx.feature.slug),
            next_command: |ctx| format!("/audit-feature {}", ctx.feature.slug),
            output_path: |ctx| format!("{}/audit.md", feature_dir(ctx))
        },
        // 19. Audit — approved, transition to QA
        rule! {
            id: "audit_approved",
            condition: |ctx| in_phase(ctx, Phase::Audit) && artifact_approved(ctx, ArtifactType::Audit),
            action: ActionType::RunQa,
            message: |ctx| format!("Audit approved. Transitioning '{}' to QA.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc feature transition {} qa", ctx.feature.slug),
            transition_to: Phase::Qa
        },
        // 20. QA — no results
        rule! {
            id: "needs_qa",
            condition: |ctx| in_phase(ctx, Phase::Qa) && artifact_missing(ctx, ArtifactType::QaResults),
            action: ActionType::RunQa,
            message: |ctx| format!("Run QA tests for '{}'.", ctx.feature.slug),
            next_command: |ctx| format!("/run-qa {}", ctx.feature.slug),
            output_path: |ctx| format!("{}/qa-results.md", feature_dir(ctx))
        },
        // 21. QA — results need approval / merge gate
        rule! {
            id: "qa_needs_approval",
            condition: |ctx| in_phase(ctx, Phase::Qa) && artifact_needs_approval(ctx, ArtifactType::QaResults),
            action: ActionType::ApproveMerge,
            message: |ctx| format!("QA results for '{}' are ready for approval.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc artifact approve {} qa_results", ctx.feature.slug)
        },
        // 22. QA — results failed
        rule! {
            id: "qa_failed",
            condition: |ctx| in_phase(ctx, Phase::Qa)
                && ctx.feature.artifact(ArtifactType::QaResults)
                    .map(|a| matches!(a.status, ArtifactStatus::Failed | ArtifactStatus::Rejected))
                    .unwrap_or(false),
            action: ActionType::FixReviewIssues,
            message: |ctx| format!("QA failed for '{}'. Fix the issues.", ctx.feature.slug),
            next_command: |ctx| format!("/fix-qa {}", ctx.feature.slug)
        },
        // 23. QA passed, approve merge → Merge phase
        rule! {
            id: "qa_approved",
            condition: |ctx| in_phase(ctx, Phase::Qa) && artifact_approved(ctx, ArtifactType::QaResults),
            action: ActionType::Merge,
            message: |ctx| format!("QA passed. '{}' is ready to merge.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc feature transition {} merge", ctx.feature.slug),
            transition_to: Phase::Merge
        },
        // 24. Merge phase — execute the merge
        rule! {
            id: "do_merge",
            condition: |ctx| in_phase(ctx, Phase::Merge),
            action: ActionType::Merge,
            message: |ctx| format!("Merge '{}' to main.", ctx.feature.slug),
            next_command: |ctx| format!("sdlc merge {}", ctx.feature.slug)
        },
        // 25. Released — nothing left
        rule! {
            id: "released",
            condition: |ctx| in_phase(ctx, Phase::Released),
            action: ActionType::Done,
            message: |ctx| format!("Feature '{}' is released.", ctx.feature.slug),
            next_command: |_| String::new()
        },
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::{Classifier, EvalContext};
    use crate::config::Config;
    use crate::feature::Feature;
    use crate::state::State;
    use crate::types::ArtifactType;
    use tempfile::TempDir;

    fn make_context<'a>(
        feature: &'a Feature,
        state: &'a State,
        config: &'a Config,
        root: &'a std::path::Path,
    ) -> EvalContext<'a> {
        EvalContext {
            feature,
            state,
            config,
            root,
        }
    }

    fn fresh_feature(dir: &TempDir, slug: &str) -> Feature {
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();
        Feature::create(dir.path(), slug, "Test").unwrap()
    }

    #[test]
    fn draft_no_spec_gives_create_spec() {
        let dir = TempDir::new().unwrap();
        let feature = fresh_feature(&dir, "auth");
        let state = State::new("proj");
        let config = Config::new("proj");
        let classifier = Classifier::new(default_rules());

        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        assert_eq!(c.action, ActionType::CreateSpec);
    }

    #[test]
    fn draft_spec_draft_gives_approve_spec() {
        let dir = TempDir::new().unwrap();
        let mut feature = fresh_feature(&dir, "auth");
        feature.mark_artifact_draft(ArtifactType::Spec).unwrap();

        let state = State::new("proj");
        let config = Config::new("proj");
        let classifier = Classifier::new(default_rules());
        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        assert_eq!(c.action, ActionType::ApproveSpec);
    }

    #[test]
    fn classification_includes_gates_from_config() {
        use crate::gate::{GateDefinition, GateKind};
        use std::collections::HashMap;

        let dir = TempDir::new().unwrap();
        let feature = fresh_feature(&dir, "auth");
        let state = State::new("proj");
        let mut config = Config::new("proj");

        // Add a gate for create_spec action
        let mut gates = HashMap::new();
        gates.insert(
            "create_spec".to_string(),
            vec![GateDefinition {
                name: "lint".to_string(),
                gate_type: GateKind::Shell {
                    command: "npm run lint".to_string(),
                },
                auto: true,
                max_retries: 0,
                timeout_seconds: 60,
            }],
        );
        config.gates = gates;

        let classifier = Classifier::new(default_rules());
        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        assert_eq!(c.action, ActionType::CreateSpec);
        assert_eq!(c.gates.len(), 1);
        assert_eq!(c.gates[0].name, "lint");
    }

    #[test]
    fn classification_empty_gates_when_not_configured() {
        let dir = TempDir::new().unwrap();
        let feature = fresh_feature(&dir, "auth");
        let state = State::new("proj");
        let config = Config::new("proj");

        let classifier = Classifier::new(default_rules());
        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        assert_eq!(c.action, ActionType::CreateSpec);
        assert!(c.gates.is_empty());
    }

    #[test]
    fn classification_gates_not_in_json_when_empty() {
        let dir = TempDir::new().unwrap();
        let feature = fresh_feature(&dir, "auth");
        let state = State::new("proj");
        let config = Config::new("proj");

        let classifier = Classifier::new(default_rules());
        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        let json = serde_json::to_string(&c).unwrap();
        assert!(!json.contains("gates"));
    }

    #[test]
    fn classification_gates_in_json_when_present() {
        use crate::gate::{GateDefinition, GateKind};
        use std::collections::HashMap;

        let dir = TempDir::new().unwrap();
        let feature = fresh_feature(&dir, "auth");
        let state = State::new("proj");
        let mut config = Config::new("proj");

        let mut gates = HashMap::new();
        gates.insert(
            "create_spec".to_string(),
            vec![GateDefinition {
                name: "build".to_string(),
                gate_type: GateKind::Shell {
                    command: "cargo build".to_string(),
                },
                auto: true,
                max_retries: 1,
                timeout_seconds: 120,
            }],
        );
        config.gates = gates;

        let classifier = Classifier::new(default_rules());
        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"gates\""));
        assert!(json.contains("\"build\""));
        assert!(json.contains("cargo build"));
    }

    #[test]
    fn blocked_feature_gives_unblock() {
        let dir = TempDir::new().unwrap();
        let mut feature = fresh_feature(&dir, "auth");
        feature.blockers.push("waiting for dependency".to_string());

        let state = State::new("proj");
        let config = Config::new("proj");
        let classifier = Classifier::new(default_rules());
        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        assert_eq!(c.action, ActionType::UnblockDependency);
    }

    #[test]
    fn blocker_comment_gives_wait_for_approval() {
        use crate::comment::{add_comment, CommentFlag, CommentTarget};

        let dir = TempDir::new().unwrap();
        let mut feature = fresh_feature(&dir, "auth");
        add_comment(
            &mut feature.comments,
            &mut feature.next_comment_seq,
            "Waiting on security review",
            Some(CommentFlag::Blocker),
            CommentTarget::Feature,
            None,
        );

        let state = State::new("proj");
        let config = Config::new("proj");
        let classifier = Classifier::new(default_rules());
        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        assert_eq!(c.action, ActionType::WaitForApproval);
        assert!(c.message.contains("blocker comment"));
    }

    #[test]
    fn question_comment_gives_wait_for_approval() {
        use crate::comment::{add_comment, CommentFlag, CommentTarget};

        let dir = TempDir::new().unwrap();
        let mut feature = fresh_feature(&dir, "auth");
        add_comment(
            &mut feature.comments,
            &mut feature.next_comment_seq,
            "What auth strategy should we use?",
            Some(CommentFlag::Question),
            CommentTarget::Feature,
            None,
        );

        let state = State::new("proj");
        let config = Config::new("proj");
        let classifier = Classifier::new(default_rules());
        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        assert_eq!(c.action, ActionType::WaitForApproval);
        assert!(c.message.contains("blocker comment"));
    }

    #[test]
    fn released_gives_done() {
        let dir = TempDir::new().unwrap();
        let mut feature = fresh_feature(&dir, "auth");
        // Force phase to released
        feature.phase = Phase::Released;

        let state = State::new("proj");
        let config = Config::new("proj");
        let classifier = Classifier::new(default_rules());
        let ctx = make_context(&feature, &state, &config, dir.path());
        let c = classifier.classify(&ctx);
        assert_eq!(c.action, ActionType::Done);
    }
}
