use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, RwLock};

/// A single running subprocess.
pub struct RunHandle {
    pub tx: broadcast::Sender<RunEvent>,
    /// Pre-subscribed receiver created before events start flowing.
    /// The first SSE subscriber takes this to guarantee no events are lost.
    /// Subsequent subscribers call `tx.subscribe()` and may miss early events.
    pub initial_rx: Mutex<Option<broadcast::Receiver<RunEvent>>>,
    /// Set to true after the subprocess finishes.
    pub completed: Arc<AtomicBool>,
}

/// Events streamed over SSE for a subprocess run.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunEvent {
    Stdout {
        line: String,
    },
    Stderr {
        line: String,
    },
    Finished {
        exit_code: i32,
        duration_seconds: f64,
    },
    Error {
        message: String,
    },
}

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub runs: Arc<RwLock<HashMap<String, RunHandle>>>,
}

impl AppState {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            runs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Remove completed runs from the registry to prevent memory leaks.
    pub async fn sweep_completed_runs(&self) {
        let mut runs = self.runs.write().await;
        runs.retain(|_, handle| !handle.completed.load(std::sync::atomic::Ordering::Relaxed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn new_state_has_empty_runs() {
        let state = AppState::new(std::path::PathBuf::from("/tmp"));
        let runs = state.runs.read().await;
        assert!(runs.is_empty());
    }

    #[tokio::test]
    async fn sweep_removes_completed_runs() {
        let state = AppState::new(std::path::PathBuf::from("/tmp"));

        // Insert a completed run
        let (tx, _) = broadcast::channel(16);
        let completed = Arc::new(AtomicBool::new(true));
        let handle = RunHandle {
            tx,
            initial_rx: Mutex::new(None),
            completed,
        };
        state.runs.write().await.insert("done-run".into(), handle);

        // Insert an active run
        let (tx2, rx2) = broadcast::channel(16);
        let handle2 = RunHandle {
            tx: tx2,
            initial_rx: Mutex::new(Some(rx2)),
            completed: Arc::new(AtomicBool::new(false)),
        };
        state
            .runs
            .write()
            .await
            .insert("active-run".into(), handle2);

        assert_eq!(state.runs.read().await.len(), 2);
        state.sweep_completed_runs().await;

        let runs = state.runs.read().await;
        assert_eq!(runs.len(), 1);
        assert!(runs.contains_key("active-run"));
        assert!(!runs.contains_key("done-run"));
    }

    #[tokio::test]
    async fn sweep_keeps_all_when_none_completed() {
        let state = AppState::new(std::path::PathBuf::from("/tmp"));

        let (tx1, rx1) = broadcast::channel(16);
        let handle1 = RunHandle {
            tx: tx1,
            initial_rx: Mutex::new(Some(rx1)),
            completed: Arc::new(AtomicBool::new(false)),
        };
        state.runs.write().await.insert("run-a".into(), handle1);

        let (tx2, rx2) = broadcast::channel(16);
        let handle2 = RunHandle {
            tx: tx2,
            initial_rx: Mutex::new(Some(rx2)),
            completed: Arc::new(AtomicBool::new(false)),
        };
        state.runs.write().await.insert("run-b".into(), handle2);

        state.sweep_completed_runs().await;
        assert_eq!(state.runs.read().await.len(), 2);
    }

    #[tokio::test]
    async fn sweep_removes_all_when_all_completed() {
        let state = AppState::new(std::path::PathBuf::from("/tmp"));

        let (tx1, _) = broadcast::channel(16);
        let handle1 = RunHandle {
            tx: tx1,
            initial_rx: Mutex::new(None),
            completed: Arc::new(AtomicBool::new(true)),
        };
        state.runs.write().await.insert("run-a".into(), handle1);

        let (tx2, _) = broadcast::channel(16);
        let handle2 = RunHandle {
            tx: tx2,
            initial_rx: Mutex::new(None),
            completed: Arc::new(AtomicBool::new(true)),
        };
        state.runs.write().await.insert("run-b".into(), handle2);

        state.sweep_completed_runs().await;
        assert!(state.runs.read().await.is_empty());
    }

    #[tokio::test]
    async fn sweep_on_empty_state_is_noop() {
        let state = AppState::new(std::path::PathBuf::from("/tmp"));
        state.sweep_completed_runs().await;
        assert!(state.runs.read().await.is_empty());
    }

    #[test]
    fn run_handle_completed_flag_toggle() {
        let (tx, _) = broadcast::channel::<RunEvent>(16);
        let completed = Arc::new(AtomicBool::new(false));
        let handle = RunHandle {
            tx,
            initial_rx: Mutex::new(None),
            completed: completed.clone(),
        };

        assert!(!handle.completed.load(Ordering::Relaxed));
        completed.store(true, Ordering::Relaxed);
        assert!(handle.completed.load(Ordering::Relaxed));
    }
}
