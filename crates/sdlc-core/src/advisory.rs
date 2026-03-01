//! Advisory history — persistent codebase health and maturity findings.
//!
//! The advisory system tracks a progressive maturity ladder (Health → Consistency
//! → Refactor → Structure → Roadmap → Advanced). An agent scans the codebase,
//! appends an `AdvisoryRun`, and merges `Finding` entries into the flat list.
//! Status transitions on findings are the only mutations; runs are append-only.

use crate::{error::Result, io};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const ADVISORY_FILE: &str = ".sdlc/advisory.yaml";

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaturityStage {
    Health,
    Consistency,
    Refactor,
    Structure,
    Roadmap,
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Open,
    Acknowledged,
    Resolved,
    Dismissed,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub stage: MaturityStage,
    pub title: String,
    pub description: String,
    pub status: FindingStatus,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryRun {
    pub run_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_count: Option<u32>,
    pub stage_reached: MaturityStage,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvisoryHistory {
    #[serde(default)]
    pub runs: Vec<AdvisoryRun>,
    #[serde(default)]
    pub findings: Vec<Finding>,
}

// ---------------------------------------------------------------------------
// Load / save
// ---------------------------------------------------------------------------

impl AdvisoryHistory {
    /// Load `.sdlc/advisory.yaml`. Returns an empty default if the file is absent.
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(ADVISORY_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(&path)?;
        let history: Self = serde_yaml::from_str(&data)?;
        Ok(history)
    }

    /// Atomically write `.sdlc/advisory.yaml`.
    pub fn save(&self, root: &Path) -> Result<()> {
        let path = root.join(ADVISORY_FILE);
        let data = serde_yaml::to_string(self)?;
        io::atomic_write(&path, data.as_bytes())
    }

    /// Load, update the specified finding's status, save, and return the updated finding.
    /// Returns `None` if no finding with the given `id` exists.
    pub fn update_finding_status(
        root: &Path,
        id: &str,
        status: FindingStatus,
    ) -> Result<Option<Finding>> {
        let mut history = Self::load(root)?;
        let Some(finding) = history.findings.iter_mut().find(|f| f.id == id) else {
            return Ok(None);
        };
        finding.status = status;
        if matches!(status, FindingStatus::Resolved) {
            finding.resolved_at = Some(Utc::now());
        }
        let updated = finding.clone();
        history.save(root)?;
        Ok(Some(updated))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn load_on_missing_file_returns_empty_default() {
        let dir = TempDir::new().unwrap();
        let history = AdvisoryHistory::load(dir.path()).unwrap();
        assert!(history.runs.is_empty());
        assert!(history.findings.is_empty());
    }

    #[test]
    fn save_and_reload_roundtrip() {
        let dir = TempDir::new().unwrap();
        let history = AdvisoryHistory {
            runs: vec![AdvisoryRun {
                run_at: Utc::now(),
                file_count: Some(42),
                stage_reached: MaturityStage::Health,
                summary: "All good".to_string(),
            }],
            findings: vec![Finding {
                id: "adv-abc123".to_string(),
                stage: MaturityStage::Health,
                title: "Dead code in auth module".to_string(),
                description: "Unused function in auth.rs".to_string(),
                status: FindingStatus::Open,
                created_at: Utc::now(),
                resolved_at: None,
            }],
        };
        history.save(dir.path()).unwrap();
        let loaded = AdvisoryHistory::load(dir.path()).unwrap();
        assert_eq!(loaded.runs.len(), 1);
        assert_eq!(loaded.findings.len(), 1);
        assert_eq!(loaded.findings[0].id, "adv-abc123");
    }

    #[test]
    fn update_finding_status_returns_none_for_unknown_id() {
        let dir = TempDir::new().unwrap();
        let result = AdvisoryHistory::update_finding_status(
            dir.path(),
            "does-not-exist",
            FindingStatus::Resolved,
        )
        .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn update_finding_status_persists_change() {
        let dir = TempDir::new().unwrap();
        let history = AdvisoryHistory {
            runs: vec![],
            findings: vec![Finding {
                id: "adv-f001".to_string(),
                stage: MaturityStage::Consistency,
                title: "Inconsistent logging".to_string(),
                description: "Mixed tracing::info! and println!".to_string(),
                status: FindingStatus::Open,
                created_at: Utc::now(),
                resolved_at: None,
            }],
        };
        history.save(dir.path()).unwrap();

        let updated = AdvisoryHistory::update_finding_status(
            dir.path(),
            "adv-f001",
            FindingStatus::Acknowledged,
        )
        .unwrap()
        .unwrap();
        assert_eq!(updated.status, FindingStatus::Acknowledged);

        let reloaded = AdvisoryHistory::load(dir.path()).unwrap();
        assert_eq!(reloaded.findings[0].status, FindingStatus::Acknowledged);
    }
}
