use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Phase
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Draft,
    Specified,
    Planned,
    Ready,
    Implementation,
    Review,
    Audit,
    Qa,
    Merge,
    Released,
}

impl Phase {
    pub fn all() -> &'static [Phase] {
        &[
            Phase::Draft,
            Phase::Specified,
            Phase::Planned,
            Phase::Ready,
            Phase::Implementation,
            Phase::Review,
            Phase::Audit,
            Phase::Qa,
            Phase::Merge,
            Phase::Released,
        ]
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn next(self) -> Option<Phase> {
        let all = Phase::all();
        let i = self.index();
        all.get(i + 1).copied()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Draft => "draft",
            Phase::Specified => "specified",
            Phase::Planned => "planned",
            Phase::Ready => "ready",
            Phase::Implementation => "implementation",
            Phase::Review => "review",
            Phase::Audit => "audit",
            Phase::Qa => "qa",
            Phase::Merge => "merge",
            Phase::Released => "released",
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Phase {
    type Err = crate::error::SdlcError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Phase::Draft),
            "specified" => Ok(Phase::Specified),
            "planned" => Ok(Phase::Planned),
            "ready" => Ok(Phase::Ready),
            "implementation" => Ok(Phase::Implementation),
            "review" => Ok(Phase::Review),
            "audit" => Ok(Phase::Audit),
            "qa" => Ok(Phase::Qa),
            "merge" => Ok(Phase::Merge),
            "released" => Ok(Phase::Released),
            _ => Err(crate::error::SdlcError::InvalidPhase(s.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// ArtifactType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Spec,
    Design,
    Tasks,
    QaPlan,
    Review,
    Audit,
    QaResults,
}

impl ArtifactType {
    pub fn as_str(self) -> &'static str {
        match self {
            ArtifactType::Spec => "spec",
            ArtifactType::Design => "design",
            ArtifactType::Tasks => "tasks",
            ArtifactType::QaPlan => "qa_plan",
            ArtifactType::Review => "review",
            ArtifactType::Audit => "audit",
            ArtifactType::QaResults => "qa_results",
        }
    }

    pub fn filename(self) -> &'static str {
        match self {
            ArtifactType::Spec => "spec.md",
            ArtifactType::Design => "design.md",
            ArtifactType::Tasks => "tasks.md",
            ArtifactType::QaPlan => "qa-plan.md",
            ArtifactType::Review => "review.md",
            ArtifactType::Audit => "audit.md",
            ArtifactType::QaResults => "qa-results.md",
        }
    }
}

impl fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ArtifactType {
    type Err = crate::error::SdlcError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "spec" => Ok(ArtifactType::Spec),
            "design" => Ok(ArtifactType::Design),
            "tasks" => Ok(ArtifactType::Tasks),
            "qa_plan" | "qa-plan" => Ok(ArtifactType::QaPlan),
            "review" => Ok(ArtifactType::Review),
            "audit" => Ok(ArtifactType::Audit),
            "qa_results" | "qa-results" => Ok(ArtifactType::QaResults),
            _ => Err(crate::error::SdlcError::ArtifactNotFound(s.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// ArtifactStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    Missing,
    Draft,
    Approved,
    Rejected,
    NeedsFix,
    Passed,
    Failed,
}

impl fmt::Display for ArtifactStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ArtifactStatus::Missing => "missing",
            ArtifactStatus::Draft => "draft",
            ArtifactStatus::Approved => "approved",
            ArtifactStatus::Rejected => "rejected",
            ArtifactStatus::NeedsFix => "needs_fix",
            ArtifactStatus::Passed => "passed",
            ArtifactStatus::Failed => "failed",
        };
        f.write_str(s)
    }
}

// ---------------------------------------------------------------------------
// ActionType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    CreateSpec,
    ApproveSpec,
    CreateDesign,
    ApproveDesign,
    CreateTasks,
    CreateQaPlan,
    ImplementTask,
    FixReviewIssues,
    CreateReview,
    ApproveReview,
    CreateAudit,
    RunQa,
    ApproveMerge,
    Merge,
    Archive,
    UnblockDependency,
    WaitForApproval,
    Done,
}

impl ActionType {
    pub fn all() -> &'static [ActionType] {
        &[
            ActionType::CreateSpec,
            ActionType::ApproveSpec,
            ActionType::CreateDesign,
            ActionType::ApproveDesign,
            ActionType::CreateTasks,
            ActionType::CreateQaPlan,
            ActionType::ImplementTask,
            ActionType::FixReviewIssues,
            ActionType::CreateReview,
            ActionType::ApproveReview,
            ActionType::CreateAudit,
            ActionType::RunQa,
            ActionType::ApproveMerge,
            ActionType::Merge,
            ActionType::Archive,
            ActionType::UnblockDependency,
            ActionType::WaitForApproval,
            ActionType::Done,
        ]
    }

    /// Returns true if the given string is a valid ActionType name.
    pub fn is_valid(s: &str) -> bool {
        Self::all().iter().any(|a| a.as_str() == s)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ActionType::CreateSpec => "create_spec",
            ActionType::ApproveSpec => "approve_spec",
            ActionType::CreateDesign => "create_design",
            ActionType::ApproveDesign => "approve_design",
            ActionType::CreateTasks => "create_tasks",
            ActionType::CreateQaPlan => "create_qa_plan",
            ActionType::ImplementTask => "implement_task",
            ActionType::FixReviewIssues => "fix_review_issues",
            ActionType::CreateReview => "create_review",
            ActionType::ApproveReview => "approve_review",
            ActionType::CreateAudit => "create_audit",
            ActionType::RunQa => "run_qa",
            ActionType::ApproveMerge => "approve_merge",
            ActionType::Merge => "merge",
            ActionType::Archive => "archive",
            ActionType::UnblockDependency => "unblock_dependency",
            ActionType::WaitForApproval => "wait_for_approval",
            ActionType::Done => "done",
        }
    }

    pub fn is_heavy(self) -> bool {
        matches!(
            self,
            ActionType::ImplementTask | ActionType::FixReviewIssues | ActionType::RunQa
        )
    }

    pub fn timeout_minutes(self) -> u32 {
        if self.is_heavy() {
            45
        } else {
            10
        }
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// TaskStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Blocked => "blocked",
        };
        f.write_str(s)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_ordering() {
        assert!(Phase::Draft < Phase::Specified);
        assert!(Phase::Specified < Phase::Planned);
        assert!(Phase::Released > Phase::Qa);
    }

    #[test]
    fn phase_next() {
        assert_eq!(Phase::Draft.next(), Some(Phase::Specified));
        assert_eq!(Phase::Specified.next(), Some(Phase::Planned));
        assert_eq!(Phase::Released.next(), None);
    }

    #[test]
    fn phase_roundtrip() {
        use std::str::FromStr;
        for phase in Phase::all() {
            let s = phase.as_str();
            let parsed = Phase::from_str(s).unwrap();
            assert_eq!(*phase, parsed);
        }
    }

    #[test]
    fn artifact_type_roundtrip() {
        use std::str::FromStr;
        let pairs = [
            ("spec", ArtifactType::Spec),
            ("design", ArtifactType::Design),
            ("tasks", ArtifactType::Tasks),
            ("qa_plan", ArtifactType::QaPlan),
            ("review", ArtifactType::Review),
            ("audit", ArtifactType::Audit),
            ("qa_results", ArtifactType::QaResults),
        ];
        for (s, expected) in pairs {
            assert_eq!(ArtifactType::from_str(s).unwrap(), expected);
        }
    }

    #[test]
    fn action_type_all_complete() {
        // Ensure all() returns 18 variants
        assert_eq!(ActionType::all().len(), 18);
    }

    #[test]
    fn action_type_is_valid() {
        assert!(ActionType::is_valid("create_spec"));
        assert!(ActionType::is_valid("implement_task"));
        assert!(ActionType::is_valid("done"));
        assert!(!ActionType::is_valid("bogus_action"));
        assert!(!ActionType::is_valid(""));
    }

    #[test]
    fn heavy_actions() {
        assert!(ActionType::ImplementTask.is_heavy());
        assert!(ActionType::FixReviewIssues.is_heavy());
        assert!(!ActionType::CreateSpec.is_heavy());
    }
}
