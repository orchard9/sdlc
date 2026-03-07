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
pub struct ActiveDirective {
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
    #[serde(default)]
    pub active_features: Vec<String>,
    #[serde(default, alias = "active_work")]
    pub active_directives: Vec<ActiveDirective>,
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
    #[serde(default)]
    pub blocked: Vec<BlockedItem>,
    #[serde(default)]
    pub milestones: Vec<String>,
    #[serde(default)]
    pub active_ponders: Vec<String>,
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
            active_directives: Vec::new(),
            history: Vec::new(),
            blocked: Vec::new(),
            milestones: Vec::new(),
            active_ponders: Vec::new(),
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
        let path_display = path.display().to_string();
        let data = std::fs::read_to_string(&path)?;

        // Phase 1: parse raw YAML (catches syntax errors with path context).
        let value: serde_yaml::Value =
            serde_yaml::from_str(&data).map_err(|e| SdlcError::ManifestParseFailed {
                path: path_display.clone(),
                message: e.to_string(),
            })?;

        // Phase 2: typed deserialization with actionable error message.
        // State has no structural migrations — #[serde(default)] covers all Vec fields.
        let state: State =
            serde_yaml::from_value(value).map_err(|e| SdlcError::ManifestIncompatible {
                path: path_display.clone(),
                entity: "State".to_string(),
                message: e.to_string(),
                fix_hint: crate::migrations::state_fix_hint(&e),
            })?;

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

    pub fn issue_directive(&mut self, feature: &str, action: ActionType) {
        // Remove any existing directive for this feature
        self.active_directives.retain(|w| w.feature != feature);
        self.active_directives.push(ActiveDirective {
            feature: feature.to_string(),
            action,
            started_at: Utc::now(),
            timeout_minutes: action.timeout_minutes(),
        });
        self.last_updated = Utc::now();
    }

    pub fn complete_directive(&mut self, feature: &str) {
        self.active_directives.retain(|w| w.feature != feature);
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

    pub fn add_ponder(&mut self, slug: &str) {
        if !self.active_ponders.contains(&slug.to_string()) {
            self.active_ponders.push(slug.to_string());
        }
        self.last_updated = Utc::now();
    }

    pub fn remove_ponder(&mut self, slug: &str) {
        self.active_ponders.retain(|s| s != slug);
        self.last_updated = Utc::now();
    }

    pub fn last_action(&self) -> Option<&HistoryEntry> {
        self.history.last()
    }

    // ---------------------------------------------------------------------------
    // Rebuild from disk
    // ---------------------------------------------------------------------------

    /// Reconstruct `active_features`, `milestones`, and `active_ponders` by
    /// scanning the `.sdlc/` directory tree.  History and other fields are
    /// preserved from the current in-memory state (caller should load first).
    pub fn rebuild(root: &Path) -> Result<Self> {
        let sdlc = root.join(".sdlc");

        // Load existing state for project name + history, or start fresh.
        let mut state = match Self::load(root) {
            Ok(s) => s,
            Err(SdlcError::NotInitialized) => {
                // Derive project name from directory
                let project = root
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                Self::new(project)
            }
            Err(e) => return Err(e),
        };

        // --- active_features: every subdir of .sdlc/features/ ---
        state.active_features.clear();
        let features_dir = sdlc.join("features");
        if features_dir.is_dir() {
            let mut slugs: Vec<String> = std::fs::read_dir(&features_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            slugs.sort();
            state.active_features = slugs;
        }

        // --- milestones: every subdir of .sdlc/milestones/ ---
        state.milestones.clear();
        let milestones_dir = sdlc.join("milestones");
        if milestones_dir.is_dir() {
            let mut slugs: Vec<String> = std::fs::read_dir(&milestones_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            slugs.sort();
            state.milestones = slugs;
        }

        // --- active_ponders: roadmap entries with exploring/converging status ---
        state.active_ponders.clear();
        let roadmap_dir = sdlc.join("roadmap");
        if roadmap_dir.is_dir() {
            let mut slugs: Vec<String> = Vec::new();
            for entry in std::fs::read_dir(&roadmap_dir)?.filter_map(|e| e.ok()) {
                if !entry.path().is_dir() {
                    continue;
                }
                let manifest = entry.path().join("manifest.yaml");
                if !manifest.exists() {
                    continue;
                }
                // Check status field — active if exploring or converging
                if let Ok(data) = std::fs::read_to_string(&manifest) {
                    let is_active = data.lines().any(|line| {
                        if let Some(val) = line.strip_prefix("status:") {
                            let val = val.trim();
                            val == "exploring" || val == "converging"
                        } else {
                            false
                        }
                    });
                    if is_active {
                        if let Ok(slug) = entry.file_name().into_string() {
                            slugs.push(slug);
                        }
                    }
                }
            }
            slugs.sort();
            state.active_ponders = slugs;
        }

        state.last_updated = Utc::now();
        Ok(state)
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
    fn active_directives_tracking() {
        let mut state = State::new("proj");
        state.issue_directive("auth", ActionType::ImplementTask);
        assert_eq!(state.active_directives.len(), 1);
        assert_eq!(state.active_directives[0].timeout_minutes, 45);

        state.complete_directive("auth");
        assert!(state.active_directives.is_empty());
    }
}
