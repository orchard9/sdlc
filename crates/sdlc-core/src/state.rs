use crate::error::{Result, SdlcError};
use crate::paths;
use crate::types::{ActionType, Phase};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWork {
    pub feature: String,
    pub action: ActionType,
    pub started_at: DateTime<Utc>,
    pub timeout_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub feature: String,
    pub action: ActionType,
    pub phase: Phase,
    pub timestamp: DateTime<Utc>,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedItem {
    pub feature: String,
    pub reason: String,
    pub since: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_version")]
    pub version: u32,
    pub project: String,
    pub active_features: Vec<String>,
    pub active_work: Vec<ActiveWork>,
    pub history: Vec<HistoryEntry>,
    pub blocked: Vec<BlockedItem>,
    #[serde(default)]
    pub milestones: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

fn default_version() -> u32 {
    1
}

impl State {
    pub fn new(project: impl Into<String>) -> Self {
        Self {
            version: 1,
            project: project.into(),
            active_features: Vec::new(),
            active_work: Vec::new(),
            history: Vec::new(),
            blocked: Vec::new(),
            milestones: Vec::new(),
            last_updated: Utc::now(),
        }
    }

    // ---------------------------------------------------------------------------
    // Persistence
    // ---------------------------------------------------------------------------

    pub fn load(root: &Path) -> Result<Self> {
        let path = paths::state_path(root);
        if !path.exists() {
            return Err(SdlcError::NotInitialized);
        }
        let data = std::fs::read_to_string(&path)?;
        let state: State = serde_yaml::from_str(&data)?;
        Ok(state)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let path = paths::state_path(root);
        let data = serde_yaml::to_string(self)?;
        crate::io::atomic_write(&path, data.as_bytes())
    }

    // ---------------------------------------------------------------------------
    // Mutations
    // ---------------------------------------------------------------------------

    pub fn add_active_feature(&mut self, slug: &str) {
        if !self.active_features.contains(&slug.to_string()) {
            self.active_features.push(slug.to_string());
        }
        self.last_updated = Utc::now();
    }

    pub fn remove_active_feature(&mut self, slug: &str) {
        self.active_features.retain(|s| s != slug);
        self.last_updated = Utc::now();
    }

    pub fn record_action(
        &mut self,
        feature: &str,
        action: ActionType,
        phase: Phase,
        outcome: &str,
    ) {
        self.history.push(HistoryEntry {
            feature: feature.to_string(),
            action,
            phase,
            timestamp: Utc::now(),
            outcome: outcome.to_string(),
        });
        // Trim history to last 200 entries
        if self.history.len() > 200 {
            self.history.drain(..self.history.len() - 200);
        }
        self.last_updated = Utc::now();
    }

    pub fn start_work(&mut self, feature: &str, action: ActionType) {
        // Remove any existing work for this feature
        self.active_work.retain(|w| w.feature != feature);
        self.active_work.push(ActiveWork {
            feature: feature.to_string(),
            action,
            started_at: Utc::now(),
            timeout_minutes: action.timeout_minutes(),
        });
        self.last_updated = Utc::now();
    }

    pub fn finish_work(&mut self, feature: &str) {
        self.active_work.retain(|w| w.feature != feature);
        self.last_updated = Utc::now();
    }

    pub fn set_blocked(&mut self, feature: &str, reason: &str) {
        self.blocked.retain(|b| b.feature != feature);
        self.blocked.push(BlockedItem {
            feature: feature.to_string(),
            reason: reason.to_string(),
            since: Utc::now(),
        });
        self.last_updated = Utc::now();
    }

    pub fn clear_blocked(&mut self, feature: &str) {
        self.blocked.retain(|b| b.feature != feature);
        self.last_updated = Utc::now();
    }

    pub fn add_milestone(&mut self, slug: &str) {
        if !self.milestones.contains(&slug.to_string()) {
            self.milestones.push(slug.to_string());
        }
        self.last_updated = Utc::now();
    }

    pub fn remove_milestone(&mut self, slug: &str) {
        self.milestones.retain(|s| s != slug);
        self.last_updated = Utc::now();
    }

    pub fn last_action(&self) -> Option<&HistoryEntry> {
        self.history.last()
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
    fn state_roundtrip() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();

        let mut state = State::new("my-project");
        state.add_active_feature("auth-login");
        state.record_action("auth-login", ActionType::CreateSpec, Phase::Draft, "ok");
        state.save(dir.path()).unwrap();

        let loaded = State::load(dir.path()).unwrap();
        assert_eq!(loaded.project, "my-project");
        assert!(loaded.active_features.contains(&"auth-login".to_string()));
        assert_eq!(loaded.history.len(), 1);
    }

    #[test]
    fn state_not_initialized() {
        let dir = TempDir::new().unwrap();
        assert!(matches!(
            State::load(dir.path()),
            Err(SdlcError::NotInitialized)
        ));
    }

    #[test]
    fn active_work_tracking() {
        let mut state = State::new("proj");
        state.start_work("auth", ActionType::ImplementTask);
        assert_eq!(state.active_work.len(), 1);
        assert_eq!(state.active_work[0].timeout_minutes, 45);

        state.finish_work("auth");
        assert!(state.active_work.is_empty());
    }
}
