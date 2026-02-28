use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

// ---------------------------------------------------------------------------
// RunRecord — persistent agent run metadata
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub key: String,
    pub run_type: String,
    pub target: String,
    pub label: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub cost_usd: Option<f64>,
    pub turns: Option<u64>,
    pub error: Option<String>,
}

/// Generate a timestamp-based run ID: "20260227-143022-abc"
pub fn generate_run_id() -> String {
    let now = chrono::Utc::now();
    let ts = now.format("%Y%m%d-%H%M%S").to_string();
    let suffix: String = (0..3).map(|_| (b'a' + (rand_u8() % 26)) as char).collect();
    format!("{ts}-{suffix}")
}

fn rand_u8() -> u8 {
    // Simple random byte from system time nanos
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as u8)
        .wrapping_mul(37)
        .wrapping_add(std::process::id() as u8)
}

// ---------------------------------------------------------------------------
// Persistence helpers
// ---------------------------------------------------------------------------

fn runs_dir(root: &Path) -> PathBuf {
    root.join(".sdlc").join(".runs")
}

/// Load all RunRecords from `.sdlc/.runs/*.json`, marking any `running` as `stopped`.
pub fn load_run_history(root: &Path) -> Vec<RunRecord> {
    let dir = runs_dir(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut records: Vec<RunRecord> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "json")
                && !e.file_name().to_string_lossy().ends_with(".events.json")
        })
        .filter_map(|e| {
            let data = std::fs::read_to_string(e.path()).ok()?;
            let mut rec: RunRecord = serde_json::from_str(&data).ok()?;
            // Mark stale running records as stopped
            if rec.status == "running" {
                rec.status = "stopped".to_string();
                rec.completed_at = Some(chrono::Utc::now().to_rfc3339());
                // Best-effort persist the update
                let _ = std::fs::write(
                    e.path(),
                    serde_json::to_string_pretty(&rec).unwrap_or_default(),
                );
            }
            Some(rec)
        })
        .collect();

    records.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    records
}

/// Write a RunRecord to `.sdlc/.runs/{id}.json`.
pub fn persist_run(root: &Path, record: &RunRecord) {
    let dir = runs_dir(root);
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{}.json", record.id));
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(record).unwrap_or_default(),
    );
}

/// Write events sidecar to `.sdlc/.runs/{id}.events.json`.
pub fn persist_run_events(root: &Path, id: &str, events: &[serde_json::Value]) {
    let dir = runs_dir(root);
    let path = dir.join(format!("{id}.events.json"));
    let _ = std::fs::write(path, serde_json::to_string(events).unwrap_or_default());
}

/// Load events sidecar from `.sdlc/.runs/{id}.events.json`.
pub fn load_run_events(root: &Path, id: &str) -> Vec<serde_json::Value> {
    let path = runs_dir(root).join(format!("{id}.events.json"));
    match std::fs::read_to_string(path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Delete oldest files if count > max.
pub fn enforce_retention(root: &Path, max: usize) {
    let dir = runs_dir(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut record_files: Vec<(PathBuf, String)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".json") && !name.ends_with(".events.json")
        })
        .map(|e| {
            let id = e
                .path()
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            (e.path(), id)
        })
        .collect();

    if record_files.len() <= max {
        return;
    }

    // Sort oldest first (by filename = timestamp-based ID)
    record_files.sort_by(|a, b| a.1.cmp(&b.1));

    let to_remove = record_files.len() - max;
    for (path, id) in record_files.into_iter().take(to_remove) {
        let _ = std::fs::remove_file(&path);
        let events_path = dir.join(format!("{id}.events.json"));
        let _ = std::fs::remove_file(events_path);
    }
}

// ---------------------------------------------------------------------------
// SSE messages
// ---------------------------------------------------------------------------

/// Typed SSE messages broadcast to all connected clients.
#[derive(Clone, Debug)]
pub enum SseMessage {
    /// Generic state-changed ping (file watcher, CLI mutations).
    Update,
    /// A ponder agent session has started.
    PonderRunStarted { slug: String, session: u32 },
    /// A ponder agent session completed (session file landed).
    PonderRunCompleted { slug: String, session: u32 },
    /// A ponder agent session was stopped by the user.
    PonderRunStopped { slug: String },
    /// An investigation agent session has started.
    InvestigationRunStarted { slug: String, session: u32 },
    /// An investigation agent session completed (session file landed).
    InvestigationRunCompleted { slug: String, session: u32 },
    /// An investigation agent session was stopped by the user.
    InvestigationRunStopped { slug: String },
    /// An agent run started (feature, milestone_uat, ponder).
    RunStarted {
        id: String,
        key: String,
        label: String,
    },
    /// An agent run finished (completed, failed, stopped).
    RunFinished {
        id: String,
        key: String,
        status: String,
    },
}

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub event_tx: broadcast::Sender<SseMessage>,
    /// Active agent runs keyed by feature slug. Each sender broadcasts
    /// JSON-serialized agent events to SSE subscribers.
    pub agent_runs: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    /// Persistent run history (active + completed).
    pub run_history: Arc<Mutex<Vec<RunRecord>>>,
}

impl AppState {
    pub fn new(root: PathBuf) -> Self {
        let (tx, _) = broadcast::channel(64);
        let history = load_run_history(&root);
        let state = Self {
            root,
            event_tx: tx.clone(),
            agent_runs: Arc::new(Mutex::new(HashMap::new())),
            run_history: Arc::new(Mutex::new(history)),
        };

        // Watch .sdlc/state.yaml mtime and broadcast when it changes.
        // This catches both web-UI mutations and external CLI updates.
        // Guard: only spawn if inside a Tokio runtime (skipped in sync unit tests).
        if tokio::runtime::Handle::try_current().is_ok() {
            let state_file = state.root.join(".sdlc").join("state.yaml");
            let tx2 = tx.clone();
            tokio::spawn(async move {
                let mut last_mtime = None::<std::time::SystemTime>;
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    if let Ok(meta) = tokio::fs::metadata(&state_file).await {
                        if let Ok(mtime) = meta.modified() {
                            if last_mtime != Some(mtime) {
                                last_mtime = Some(mtime);
                                let _ = tx2.send(SseMessage::Update);
                            }
                        }
                    }
                }
            });

            // Watch roadmap manifests for ponder space changes.
            // atomic_write uses rename, which updates the parent slug dir's mtime
            // but NOT the top-level roadmap/ dir mtime — so we scan each
            // manifest file directly instead of watching the directory.
            let roadmap_dir = state.root.join(".sdlc").join("roadmap");
            let tx_roadmap = tx.clone();
            tokio::spawn(async move {
                let mut last_mtime = None::<std::time::SystemTime>;
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    let latest = scan_dir_mtime(&roadmap_dir).await;
                    if latest != last_mtime {
                        last_mtime = latest;
                        let _ = tx_roadmap.send(SseMessage::Update);
                    }
                }
            });

            // Watch investigations dir for investigation workspace changes.
            let investigations_dir = state.root.join(".sdlc").join("investigations");
            tokio::spawn(async move {
                let mut last_mtime = None::<std::time::SystemTime>;
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    let latest = scan_dir_mtime(&investigations_dir).await;
                    if latest != last_mtime {
                        last_mtime = latest;
                        let _ = tx.send(SseMessage::Update);
                    }
                }
            });
        }

        state
    }
}

/// Scan `<dir>/<slug>/manifest.yaml` and `<dir>/<slug>/sessions/`
/// across all slug subdirectories and return the most recent mtime found.
/// This is needed because atomic_write uses rename, which updates the slug
/// subdirectory mtime but not the top-level directory mtime.
async fn scan_dir_mtime(roadmap_dir: &std::path::Path) -> Option<std::time::SystemTime> {
    let mut latest: Option<std::time::SystemTime> = None;

    let mut dir = match tokio::fs::read_dir(roadmap_dir).await {
        Ok(d) => d,
        Err(_) => return None,
    };

    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // Check manifest.yaml
        let manifest = path.join("manifest.yaml");
        if let Ok(meta) = tokio::fs::metadata(&manifest).await {
            if let Ok(mtime) = meta.modified() {
                if latest.is_none_or(|l| mtime > l) {
                    latest = Some(mtime);
                }
            }
        }
        // Check sessions directory mtime
        let sessions_dir = path.join("sessions");
        if let Ok(meta) = tokio::fs::metadata(&sessions_dir).await {
            if let Ok(mtime) = meta.modified() {
                if latest.is_none_or(|l| mtime > l) {
                    latest = Some(mtime);
                }
            }
        }
    }

    latest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_stores_root() {
        let state = AppState::new(std::path::PathBuf::from("/tmp/test"));
        assert_eq!(state.root, std::path::PathBuf::from("/tmp/test"));
    }
}
