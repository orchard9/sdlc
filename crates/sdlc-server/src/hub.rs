use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

/// Heartbeat payload sent by project instances every 30s.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HeartbeatPayload {
    pub name: String,
    pub url: String,
    pub active_milestone: Option<String>,
    pub feature_count: Option<u32>,
    pub agent_running: Option<bool>,
}

// ---------------------------------------------------------------------------
// Registry types
// ---------------------------------------------------------------------------

/// Online/stale/offline classification based on last_seen age.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    /// Last heartbeat < 30 seconds ago.
    Online,
    /// Last heartbeat 30–90 seconds ago.
    Stale,
    /// Last heartbeat 90 seconds – 5 minutes ago.
    Offline,
}

/// One entry in the hub project registry.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProjectEntry {
    pub name: String,
    pub url: String,
    pub active_milestone: Option<String>,
    pub feature_count: Option<u32>,
    pub agent_running: Option<bool>,
    pub last_seen: DateTime<Utc>,
    pub status: ProjectStatus,
}

impl ProjectEntry {
    /// Recompute `status` from the stored `last_seen` value.
    pub fn recompute_status(&mut self) {
        self.status = status_for_age(Utc::now() - self.last_seen);
    }
}

fn status_for_age(age: chrono::Duration) -> ProjectStatus {
    let secs = age.num_seconds();
    if secs < 30 {
        ProjectStatus::Online
    } else if secs < 90 {
        ProjectStatus::Stale
    } else {
        ProjectStatus::Offline
    }
}

// ---------------------------------------------------------------------------
// SSE messages
// ---------------------------------------------------------------------------

/// Events broadcast to hub SSE subscribers.
#[derive(Clone, Debug)]
pub enum HubSseMessage {
    ProjectUpdated(ProjectEntry),
    ProjectRemoved { url: String },
}

// ---------------------------------------------------------------------------
// Persistence file shape
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct HubStateFile {
    projects: Vec<ProjectEntry>,
}

// ---------------------------------------------------------------------------
// HubRegistry
// ---------------------------------------------------------------------------

/// In-memory project registry for hub mode.
pub struct HubRegistry {
    /// Project entries keyed by URL.
    pub projects: HashMap<String, ProjectEntry>,
    /// Broadcast channel for SSE subscribers.
    pub event_tx: broadcast::Sender<HubSseMessage>,
    /// Path for persistent cache file (`~/.sdlc/hub-state.yaml`).
    pub persist_path: PathBuf,
}

impl HubRegistry {
    /// Create a new registry, loading any saved state from `persist_path`.
    /// All loaded entries start as `offline` — they are warm-cache placeholders
    /// until a live heartbeat confirms them.
    pub fn new(persist_path: PathBuf) -> Self {
        let (tx, _) = broadcast::channel(64);
        let mut registry = Self {
            projects: HashMap::new(),
            event_tx: tx,
            persist_path,
        };
        registry.load_saved_state();
        registry
    }

    fn load_saved_state(&mut self) {
        if !self.persist_path.exists() {
            return;
        }
        let data = match std::fs::read_to_string(&self.persist_path) {
            Ok(d) => d,
            Err(_) => return,
        };
        let state: HubStateFile = match serde_yaml::from_str(&data) {
            Ok(s) => s,
            Err(_) => return,
        };
        for mut entry in state.projects {
            entry.status = ProjectStatus::Offline;
            self.projects.insert(entry.url.clone(), entry);
        }
    }

    /// Apply a heartbeat payload: upsert the entry, update last_seen, emit event.
    pub fn apply_heartbeat(&mut self, payload: HeartbeatPayload) -> ProjectEntry {
        let now = Utc::now();
        let entry = self
            .projects
            .entry(payload.url.clone())
            .or_insert_with(|| ProjectEntry {
                name: payload.name.clone(),
                url: payload.url.clone(),
                active_milestone: None,
                feature_count: None,
                agent_running: None,
                last_seen: now,
                status: ProjectStatus::Online,
            });

        entry.name = payload.name;
        if let Some(m) = payload.active_milestone {
            entry.active_milestone = Some(m);
        }
        if let Some(f) = payload.feature_count {
            entry.feature_count = Some(f);
        }
        if let Some(a) = payload.agent_running {
            entry.agent_running = Some(a);
        }
        entry.last_seen = now;
        entry.status = ProjectStatus::Online;

        let result = entry.clone();
        let _ = self
            .event_tx
            .send(HubSseMessage::ProjectUpdated(result.clone()));
        self.persist();
        result
    }

    /// Sweep: recompute statuses, remove entries older than 5 minutes.
    /// Emits `ProjectRemoved` for each removed entry.
    pub fn sweep(&mut self) {
        let now = Utc::now();
        let five_min = chrono::Duration::seconds(300);

        let stale_urls: Vec<String> = self
            .projects
            .iter()
            .filter(|(_, e)| now - e.last_seen > five_min)
            .map(|(url, _)| url.clone())
            .collect();

        let mut changed = false;
        for url in stale_urls {
            self.projects.remove(&url);
            let _ = self.event_tx.send(HubSseMessage::ProjectRemoved { url });
            changed = true;
        }

        // Recompute status for remaining entries.
        for entry in self.projects.values_mut() {
            let new_status = status_for_age(now - entry.last_seen);
            if entry.status != new_status {
                entry.status = new_status;
                changed = true;
            }
        }

        if changed {
            self.persist();
        }
    }

    /// Write registry to persist_path using atomic_write.
    pub fn persist(&self) {
        let state = HubStateFile {
            projects: self.projects.values().cloned().collect(),
        };
        match serde_yaml::to_string(&state) {
            Ok(yaml) => {
                if let Some(parent) = self.persist_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = sdlc_core::io::atomic_write(&self.persist_path, yaml.as_bytes()) {
                    tracing::warn!(path = %self.persist_path.display(), error = %e, "hub: persist failed");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "hub: failed to serialize registry for persistence");
            }
        }
    }

    /// Return all projects sorted by `last_seen` descending (most recent first).
    pub fn projects_sorted(&self) -> Vec<ProjectEntry> {
        let mut entries: Vec<ProjectEntry> = self.projects.values().cloned().collect();
        entries.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));
        entries
    }
}

/// Construct the hub state persistence path: `~/.sdlc/hub-state.yaml`.
pub fn default_persist_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".sdlc")
        .join("hub-state.yaml")
}

/// Spawn the hub sweep background task (runs every 15 seconds).
pub fn spawn_sweep_task(
    registry: std::sync::Arc<tokio::sync::Mutex<HubRegistry>>,
) -> tokio::task::AbortHandle {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            let mut reg = registry.lock().await;
            reg.sweep();
        }
    })
    .abort_handle()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_registry(dir: &std::path::Path) -> HubRegistry {
        HubRegistry::new(dir.join("hub-state.yaml"))
    }

    fn payload(name: &str, url: &str) -> HeartbeatPayload {
        HeartbeatPayload {
            name: name.to_string(),
            url: url.to_string(),
            active_milestone: None,
            feature_count: None,
            agent_running: None,
        }
    }

    #[test]
    fn apply_heartbeat_creates_online_entry() {
        let tmp = TempDir::new().unwrap();
        let mut reg = temp_registry(tmp.path());
        let entry = reg.apply_heartbeat(payload("payments", "http://localhost:3001"));
        assert_eq!(entry.name, "payments");
        assert_eq!(entry.status, ProjectStatus::Online);
        assert!(reg.projects.contains_key("http://localhost:3001"));
    }

    #[test]
    fn apply_heartbeat_updates_existing_entry() {
        let tmp = TempDir::new().unwrap();
        let mut reg = temp_registry(tmp.path());
        reg.apply_heartbeat(payload("payments", "http://localhost:3001"));
        let mut p2 = payload("payments-updated", "http://localhost:3001");
        p2.feature_count = Some(5);
        reg.apply_heartbeat(p2);
        assert_eq!(reg.projects.len(), 1);
        let entry = &reg.projects["http://localhost:3001"];
        assert_eq!(entry.name, "payments-updated");
        assert_eq!(entry.feature_count, Some(5));
    }

    #[test]
    fn sweep_marks_stale_after_30s() {
        let tmp = TempDir::new().unwrap();
        let mut reg = temp_registry(tmp.path());
        // Insert entry with last_seen 45s ago
        let entry = ProjectEntry {
            name: "test".into(),
            url: "http://localhost:3001".into(),
            active_milestone: None,
            feature_count: None,
            agent_running: None,
            last_seen: Utc::now() - chrono::Duration::seconds(45),
            status: ProjectStatus::Online,
        };
        reg.projects.insert(entry.url.clone(), entry);
        reg.sweep();
        assert_eq!(
            reg.projects["http://localhost:3001"].status,
            ProjectStatus::Stale
        );
    }

    #[test]
    fn sweep_marks_offline_after_90s() {
        let tmp = TempDir::new().unwrap();
        let mut reg = temp_registry(tmp.path());
        let entry = ProjectEntry {
            name: "test".into(),
            url: "http://localhost:3001".into(),
            active_milestone: None,
            feature_count: None,
            agent_running: None,
            last_seen: Utc::now() - chrono::Duration::seconds(120),
            status: ProjectStatus::Online,
        };
        reg.projects.insert(entry.url.clone(), entry);
        reg.sweep();
        assert_eq!(
            reg.projects["http://localhost:3001"].status,
            ProjectStatus::Offline
        );
    }

    #[test]
    fn sweep_removes_entry_after_5_minutes() {
        let tmp = TempDir::new().unwrap();
        let mut reg = temp_registry(tmp.path());
        let entry = ProjectEntry {
            name: "test".into(),
            url: "http://localhost:3001".into(),
            active_milestone: None,
            feature_count: None,
            agent_running: None,
            last_seen: Utc::now() - chrono::Duration::seconds(310),
            status: ProjectStatus::Offline,
        };
        reg.projects.insert(entry.url.clone(), entry);
        reg.sweep();
        assert!(!reg.projects.contains_key("http://localhost:3001"));
    }

    #[test]
    fn new_loads_persisted_state_as_offline() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("hub-state.yaml");
        let state = HubStateFile {
            projects: vec![ProjectEntry {
                name: "saved".into(),
                url: "http://localhost:4000".into(),
                active_milestone: None,
                feature_count: Some(2),
                agent_running: Some(false),
                last_seen: Utc::now() - chrono::Duration::seconds(10),
                status: ProjectStatus::Online,
            }],
        };
        let yaml = serde_yaml::to_string(&state).unwrap();
        std::fs::write(&path, yaml).unwrap();

        let reg = HubRegistry::new(path);
        assert!(reg.projects.contains_key("http://localhost:4000"));
        assert_eq!(
            reg.projects["http://localhost:4000"].status,
            ProjectStatus::Offline
        );
    }

    #[test]
    fn projects_sorted_returns_newest_first() {
        let tmp = TempDir::new().unwrap();
        let mut reg = temp_registry(tmp.path());
        for (url, secs_ago) in [
            ("http://localhost:3001", 60i64),
            ("http://localhost:3002", 10),
            ("http://localhost:3003", 120),
        ] {
            reg.projects.insert(
                url.to_string(),
                ProjectEntry {
                    name: url.to_string(),
                    url: url.to_string(),
                    active_milestone: None,
                    feature_count: None,
                    agent_running: None,
                    last_seen: Utc::now() - chrono::Duration::seconds(secs_ago),
                    status: ProjectStatus::Stale,
                },
            );
        }
        let sorted = reg.projects_sorted();
        assert_eq!(sorted[0].url, "http://localhost:3002"); // 10s ago — newest
        assert_eq!(sorted[1].url, "http://localhost:3001"); // 60s ago
        assert_eq!(sorted[2].url, "http://localhost:3003"); // 120s ago — oldest
    }
}
