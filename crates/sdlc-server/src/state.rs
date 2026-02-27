use std::path::PathBuf;
use tokio::sync::broadcast;

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub event_tx: broadcast::Sender<()>,
}

impl AppState {
    pub fn new(root: PathBuf) -> Self {
        let (tx, _) = broadcast::channel(64);
        let state = Self {
            root,
            event_tx: tx.clone(),
        };

        // Watch .sdlc/state.yaml mtime and broadcast when it changes.
        // This catches both web-UI mutations and external CLI updates.
        // Guard: only spawn if inside a Tokio runtime (skipped in sync unit tests).
        if tokio::runtime::Handle::try_current().is_ok() {
            let state_file = state.root.join(".sdlc").join("state.yaml");
            tokio::spawn(async move {
                let mut last_mtime = None::<std::time::SystemTime>;
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    if let Ok(meta) = tokio::fs::metadata(&state_file).await {
                        if let Ok(mtime) = meta.modified() {
                            if last_mtime != Some(mtime) {
                                last_mtime = Some(mtime);
                                let _ = tx.send(());
                            }
                        }
                    }
                }
            });
        }

        state
    }
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
