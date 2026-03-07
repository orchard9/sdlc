use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use uuid::Uuid;

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

/// Provision lifecycle for a fleet workspace requested through the hub.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionState {
    Requested,
    Provisioning,
    Ready,
    Failed,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProvisionEntry {
    pub slug: String,
    pub url: String,
    pub status: ProvisionState,
    pub source: String,
    pub detail: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivitySeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HubActivityEntry {
    pub id: String,
    pub kind: String,
    pub severity: ActivitySeverity,
    pub title: String,
    pub detail: Option<String>,
    pub slug: Option<String>,
    pub url: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// SSE messages
// ---------------------------------------------------------------------------

/// Events broadcast to hub SSE subscribers.
#[derive(Clone, Debug)]
pub enum HubSseMessage {
    ProjectUpdated(ProjectEntry),
    ProjectRemoved {
        url: String,
    },
    FleetUpdated(crate::fleet::FleetInstance),
    FleetProvisioned(crate::fleet::FleetInstance),
    ProvisionUpdated(ProvisionEntry),
    ActivityAppended(HubActivityEntry),
    FleetAgentStatus {
        total_active_runs: usize,
        projects_with_agents: usize,
    },
}

// ---------------------------------------------------------------------------
// Persistence file shape
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct HubStateFile {
    #[serde(default)]
    projects: Vec<ProjectEntry>,
    #[serde(default)]
    provisions: Vec<ProvisionEntry>,
    #[serde(default)]
    activity: Vec<HubActivityEntry>,
}

// ---------------------------------------------------------------------------
// HubRegistry
// ---------------------------------------------------------------------------

/// In-memory project registry for hub mode.
pub struct HubRegistry {
    /// Project entries keyed by URL.
    pub projects: HashMap<String, ProjectEntry>,
    /// Provision records keyed by slug.
    pub provisions: HashMap<String, ProvisionEntry>,
    /// Recent fleet activity entries, newest last.
    pub activity: Vec<HubActivityEntry>,
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
            provisions: HashMap::new(),
            activity: Vec::new(),
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
        for provision in state.provisions {
            self.provisions.insert(provision.slug.clone(), provision);
        }
        self.activity = state.activity;
        if self.activity.len() > 100 {
            self.activity.drain(..self.activity.len() - 100);
        }
    }

    pub fn push_activity(
        &mut self,
        kind: &str,
        severity: ActivitySeverity,
        title: impl Into<String>,
        detail: Option<String>,
        slug: Option<String>,
        url: Option<String>,
    ) -> HubActivityEntry {
        let entry = HubActivityEntry {
            id: Uuid::new_v4().to_string(),
            kind: kind.to_string(),
            severity,
            title: title.into(),
            detail,
            slug,
            url,
            created_at: Utc::now(),
        };
        self.activity.push(entry.clone());
        if self.activity.len() > 100 {
            self.activity.drain(..self.activity.len() - 100);
        }
        let _ = self
            .event_tx
            .send(HubSseMessage::ActivityAppended(entry.clone()));
        entry
    }

    fn active_agent_summary(&self) -> (usize, usize) {
        let projects_with_agents = self
            .projects
            .values()
            .filter(|p| p.agent_running == Some(true) && p.status != ProjectStatus::Offline)
            .count();
        (projects_with_agents, projects_with_agents)
    }

    fn broadcast_agent_summary(&self) {
        let (total_active_runs, projects_with_agents) = self.active_agent_summary();
        let _ = self.event_tx.send(HubSseMessage::FleetAgentStatus {
            total_active_runs,
            projects_with_agents,
        });
    }

    fn set_provision_status(
        &mut self,
        slug: &str,
        status: ProvisionState,
        detail: Option<String>,
    ) -> Option<ProvisionEntry> {
        let (previous, result) = {
            let provision = self.provisions.get_mut(slug)?;
            if provision.status == status && (detail.is_none() || detail == provision.detail) {
                return Some(provision.clone());
            }
            let previous = provision.status.clone();
            provision.status = status.clone();
            provision.updated_at = Utc::now();
            if detail.is_some() {
                provision.detail = detail.clone();
            }
            (previous, provision.clone())
        };
        let _ = self
            .event_tx
            .send(HubSseMessage::ProvisionUpdated(result.clone()));

        let (severity, title) = match status {
            ProvisionState::Requested => (
                ActivitySeverity::Info,
                format!("Provision requested for {}", result.slug),
            ),
            ProvisionState::Provisioning => (
                ActivitySeverity::Info,
                format!("Provisioning {}", result.slug),
            ),
            ProvisionState::Ready => (
                ActivitySeverity::Success,
                format!("{} is ready", result.slug),
            ),
            ProvisionState::Failed => (
                ActivitySeverity::Error,
                format!("Provision failed for {}", result.slug),
            ),
        };
        if previous != status {
            self.push_activity(
                "provision_status",
                severity,
                title,
                detail,
                Some(result.slug.clone()),
                Some(result.url.clone()),
            );
        }
        Some(result)
    }

    pub fn start_provision(
        &mut self,
        slug: &str,
        url: String,
        source: &str,
        detail: Option<String>,
    ) -> ProvisionEntry {
        let now = Utc::now();
        let entry = self
            .provisions
            .entry(slug.to_string())
            .or_insert_with(|| ProvisionEntry {
                slug: slug.to_string(),
                url: url.clone(),
                status: ProvisionState::Requested,
                source: source.to_string(),
                detail: detail.clone(),
                created_at: now,
                updated_at: now,
            });
        entry.url = url;
        entry.source = source.to_string();
        entry.status = ProvisionState::Requested;
        entry.detail = detail.clone();
        entry.updated_at = now;

        let result = entry.clone();
        let _ = self
            .event_tx
            .send(HubSseMessage::ProvisionUpdated(result.clone()));
        self.push_activity(
            "provision_requested",
            ActivitySeverity::Info,
            format!("Provision requested for {slug}"),
            detail,
            Some(slug.to_string()),
            Some(result.url.clone()),
        );
        self.persist();
        result
    }

    pub fn reconcile_fleet(&mut self, instances: &[crate::fleet::FleetInstance]) {
        for instance in instances {
            let target = match instance.deployment_status {
                crate::fleet::DeploymentStatus::Failed => Some(ProvisionState::Failed),
                crate::fleet::DeploymentStatus::Pending => Some(ProvisionState::Provisioning),
                crate::fleet::DeploymentStatus::Running if instance.pod_healthy => {
                    Some(ProvisionState::Ready)
                }
                _ => None,
            };
            if let Some(target_status) = target {
                let detail = if !instance.attention_reasons.is_empty() {
                    Some(instance.attention_reasons.join("; "))
                } else {
                    None
                };
                let _ = self.set_provision_status(&instance.slug, target_status, detail);
            }
        }
        self.persist();
    }

    pub fn activity_recent(&self, limit: usize) -> Vec<HubActivityEntry> {
        let start = self.activity.len().saturating_sub(limit);
        let mut items = self.activity[start..].to_vec();
        items.reverse();
        items
    }

    /// Apply a heartbeat payload: upsert the entry, update last_seen, emit event.
    pub fn apply_heartbeat(&mut self, payload: HeartbeatPayload) -> ProjectEntry {
        let now = Utc::now();
        let mut is_new = false;
        let entry = self.projects.entry(payload.url.clone()).or_insert_with(|| {
            is_new = true;
            ProjectEntry {
                name: payload.name.clone(),
                url: payload.url.clone(),
                active_milestone: None,
                feature_count: None,
                agent_running: None,
                last_seen: now,
                status: ProjectStatus::Online,
            }
        });

        let prev_status = entry.status.clone();
        let prev_agent_running = entry.agent_running;
        let prev_milestone = entry.active_milestone.clone();
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
        if is_new {
            self.push_activity(
                "project_connected",
                ActivitySeverity::Success,
                format!("{} connected", result.name),
                None,
                Some(result.name.clone()),
                Some(result.url.clone()),
            );
        } else {
            if prev_status != ProjectStatus::Online {
                self.push_activity(
                    "project_recovered",
                    ActivitySeverity::Success,
                    format!("{} is back online", result.name),
                    None,
                    Some(result.name.clone()),
                    Some(result.url.clone()),
                );
            }
            if prev_milestone != result.active_milestone && result.active_milestone.is_some() {
                self.push_activity(
                    "milestone_changed",
                    ActivitySeverity::Info,
                    format!("{} switched milestone", result.name),
                    result.active_milestone.clone(),
                    Some(result.name.clone()),
                    Some(result.url.clone()),
                );
            }
            if prev_agent_running != Some(true) && result.agent_running == Some(true) {
                self.push_activity(
                    "agent_started",
                    ActivitySeverity::Info,
                    format!("Agent started in {}", result.name),
                    result.active_milestone.clone(),
                    Some(result.name.clone()),
                    Some(result.url.clone()),
                );
            }
            if prev_agent_running == Some(true) && result.agent_running != Some(true) {
                self.push_activity(
                    "agent_stopped",
                    ActivitySeverity::Info,
                    format!("Agent stopped in {}", result.name),
                    result.active_milestone.clone(),
                    Some(result.name.clone()),
                    Some(result.url.clone()),
                );
            }
        }
        if self.provisions.contains_key(&result.name) {
            let _ = self.set_provision_status(
                &result.name,
                ProvisionState::Ready,
                Some("Project heartbeat received".to_string()),
            );
        }
        if is_new || prev_agent_running != result.agent_running {
            self.broadcast_agent_summary();
        }
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
            if let Some(project) = self.projects.remove(&url) {
                self.push_activity(
                    "project_removed",
                    ActivitySeverity::Warning,
                    format!("{} stopped reporting", project.name),
                    Some("No heartbeat for 5 minutes".to_string()),
                    Some(project.name),
                    Some(url.clone()),
                );
            }
            let _ = self.event_tx.send(HubSseMessage::ProjectRemoved { url });
            changed = true;
        }

        let mut status_events: Vec<(ActivitySeverity, String, Option<String>, String, String)> =
            Vec::new();

        // Recompute status for remaining entries.
        for entry in self.projects.values_mut() {
            let new_status = status_for_age(now - entry.last_seen);
            if entry.status != new_status {
                entry.status = new_status;
                let severity = match entry.status {
                    ProjectStatus::Online => ActivitySeverity::Success,
                    ProjectStatus::Stale => ActivitySeverity::Warning,
                    ProjectStatus::Offline => ActivitySeverity::Error,
                };
                let detail = match entry.status {
                    ProjectStatus::Online => None,
                    ProjectStatus::Stale => Some("Heartbeat is older than 30 seconds".to_string()),
                    ProjectStatus::Offline => {
                        Some("Heartbeat is older than 90 seconds".to_string())
                    }
                };
                let title = match entry.status {
                    ProjectStatus::Online => format!("{} is online", entry.name),
                    ProjectStatus::Stale => format!("{} heartbeat is stale", entry.name),
                    ProjectStatus::Offline => format!("{} heartbeat is offline", entry.name),
                };
                status_events.push((
                    severity,
                    title,
                    detail,
                    entry.name.clone(),
                    entry.url.clone(),
                ));
                let _ = self
                    .event_tx
                    .send(HubSseMessage::ProjectUpdated(entry.clone()));
                changed = true;
            }
        }

        for (severity, title, detail, name, url) in status_events {
            self.push_activity(
                "project_status",
                severity,
                title,
                detail,
                Some(name),
                Some(url),
            );
        }

        if changed {
            self.broadcast_agent_summary();
            self.persist();
        }
    }

    /// Write registry to persist_path using atomic_write.
    pub fn persist(&self) {
        let state = HubStateFile {
            projects: self.projects.values().cloned().collect(),
            provisions: self.provisions.values().cloned().collect(),
            activity: self.activity.clone(),
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

    /// Remove a project from both the projects and provisions maps.
    /// Emits `ProjectRemoved` SSE event and persists.
    pub fn remove_project(&mut self, slug: &str) {
        // Projects are keyed by URL — find by name match
        let url = self
            .projects
            .iter()
            .find(|(_, p)| p.name == slug)
            .map(|(url, _)| url.clone());
        if let Some(url) = &url {
            self.projects.remove(url);
            let _ = self
                .event_tx
                .send(HubSseMessage::ProjectRemoved { url: url.clone() });
        }
        self.provisions.remove(slug);
        self.push_activity(
            "project_deleted",
            ActivitySeverity::Warning,
            format!("{slug} deleted"),
            None,
            Some(slug.to_string()),
            url,
        );
        self.persist();
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
            provisions: Vec::new(),
            activity: Vec::new(),
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

    #[test]
    fn start_provision_creates_requested_record() {
        let tmp = TempDir::new().unwrap();
        let mut reg = temp_registry(tmp.path());
        let provision = reg.start_provision(
            "auth-service",
            "https://auth-service.sdlc.threesix.ai".to_string(),
            "manual",
            None,
        );
        assert_eq!(provision.status, ProvisionState::Requested);
        assert!(reg.provisions.contains_key("auth-service"));
        assert!(!reg.activity.is_empty());
    }

    #[test]
    fn remove_project_clears_both_maps_and_persists() {
        let tmp = TempDir::new().unwrap();
        let mut reg = temp_registry(tmp.path());
        reg.apply_heartbeat(payload("my-app", "http://localhost:4000"));
        reg.start_provision(
            "my-app",
            "https://my-app.sdlc.threesix.ai".to_string(),
            "manual",
            None,
        );
        assert!(reg.projects.contains_key("http://localhost:4000"));
        assert!(reg.provisions.contains_key("my-app"));

        reg.remove_project("my-app");

        assert!(!reg.projects.contains_key("http://localhost:4000"));
        assert!(!reg.provisions.contains_key("my-app"));
        // Verify persisted file exists (persist was called)
        assert!(tmp.path().join("hub-state.yaml").exists());
        // Verify activity was logged
        assert!(reg
            .activity
            .iter()
            .any(|a| a.kind == "project_deleted" && a.slug.as_deref() == Some("my-app")));
    }

    #[test]
    fn heartbeat_marks_matching_provision_ready() {
        let tmp = TempDir::new().unwrap();
        let mut reg = temp_registry(tmp.path());
        reg.start_provision(
            "payments-api",
            "https://payments-api.sdlc.threesix.ai".to_string(),
            "manual",
            None,
        );

        let _ = reg.apply_heartbeat(HeartbeatPayload {
            name: "payments-api".to_string(),
            url: "https://payments-api.sdlc.threesix.ai".to_string(),
            active_milestone: None,
            feature_count: None,
            agent_running: Some(false),
        });

        assert_eq!(reg.provisions["payments-api"].status, ProvisionState::Ready);
    }
}
