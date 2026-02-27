use crate::types::{ArtifactStatus, ArtifactType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_type: ArtifactType,
    pub status: ArtifactStatus,
    pub path: String,
    pub created_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub approved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waived_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waive_reason: Option<String>,
}

impl Artifact {
    pub fn new(artifact_type: ArtifactType, path: impl Into<String>) -> Self {
        Self {
            artifact_type,
            status: ArtifactStatus::Missing,
            path: path.into(),
            created_at: None,
            approved_at: None,
            rejected_at: None,
            rejection_reason: None,
            approved_by: None,
            waived_at: None,
            waive_reason: None,
        }
    }

    pub fn mark_draft(&mut self) {
        self.status = ArtifactStatus::Draft;
        self.created_at = Some(Utc::now());
        self.waived_at = None;
        self.waive_reason = None;
    }

    pub fn approve(&mut self, approved_by: Option<String>) {
        self.status = ArtifactStatus::Approved;
        self.approved_at = Some(Utc::now());
        self.approved_by = approved_by;
        self.rejected_at = None;
        self.rejection_reason = None;
        self.waived_at = None;
        self.waive_reason = None;
    }

    pub fn reject(&mut self, reason: Option<String>) {
        self.status = ArtifactStatus::Rejected;
        self.rejected_at = Some(Utc::now());
        self.rejection_reason = reason;
        self.approved_at = None;
        self.waived_at = None;
        self.waive_reason = None;
    }

    pub fn mark_needs_fix(&mut self) {
        self.status = ArtifactStatus::NeedsFix;
    }

    pub fn mark_passed(&mut self) {
        self.status = ArtifactStatus::Passed;
    }

    pub fn mark_failed(&mut self) {
        self.status = ArtifactStatus::Failed;
    }

    pub fn waive(&mut self, reason: Option<String>) {
        self.status = ArtifactStatus::Waived;
        self.waived_at = Some(Utc::now());
        self.waive_reason = reason;
        self.approved_at = None;
        self.rejected_at = None;
        self.rejection_reason = None;
    }

    pub fn is_approved(&self) -> bool {
        matches!(
            self.status,
            ArtifactStatus::Approved | ArtifactStatus::Passed
        )
    }

    pub fn is_satisfied(&self) -> bool {
        matches!(
            self.status,
            ArtifactStatus::Approved | ArtifactStatus::Passed | ArtifactStatus::Waived
        )
    }

    pub fn exists_on_disk(&self, root: &std::path::Path) -> bool {
        root.join(&self.path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_lifecycle() {
        let mut a = Artifact::new(ArtifactType::Spec, ".sdlc/features/auth/spec.md");
        assert_eq!(a.status, ArtifactStatus::Missing);
        assert!(!a.is_approved());

        a.mark_draft();
        assert_eq!(a.status, ArtifactStatus::Draft);

        a.approve(Some("human".to_string()));
        assert_eq!(a.status, ArtifactStatus::Approved);
        assert!(a.is_approved());
        assert!(a.approved_at.is_some());

        a.reject(Some("too vague".to_string()));
        assert_eq!(a.status, ArtifactStatus::Rejected);
        assert!(!a.is_approved());
        assert_eq!(a.rejection_reason.as_deref(), Some("too vague"));
    }

    #[test]
    fn artifact_waive_lifecycle() {
        let mut a = Artifact::new(ArtifactType::Design, ".sdlc/features/auth/design.md");
        assert_eq!(a.status, ArtifactStatus::Missing);
        assert!(!a.is_satisfied());

        a.waive(Some("simple CRUD, no arch decisions".to_string()));
        assert_eq!(a.status, ArtifactStatus::Waived);
        assert!(a.is_satisfied());
        assert!(a.waived_at.is_some());
        assert_eq!(
            a.waive_reason.as_deref(),
            Some("simple CRUD, no arch decisions")
        );
    }
}
