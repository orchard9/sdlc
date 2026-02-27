use crate::error::{Result, SdlcError};
use crate::io::atomic_write;
use crate::paths::{user_sdlc_dir, user_ui_record_path};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// UiRecord
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRecord {
    pub project: String,
    pub root: PathBuf,
    pub pid: u32,
    pub port: u16,
    pub url: String,
    pub started_at: DateTime<Utc>,
}

impl UiRecord {
    /// Atomically write this record to `~/.sdlc/{project}.yaml`.
    pub fn write(&self) -> Result<()> {
        let path = user_ui_record_path(&self.project)?;
        let data = serde_yaml::to_string(self)?;
        atomic_write(&path, data.as_bytes())
    }

    /// Remove this record file. Silently succeeds if the file is gone.
    pub fn remove(&self) -> Result<()> {
        let path = user_ui_record_path(&self.project)?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Registry helpers
// ---------------------------------------------------------------------------

/// Read all records from `~/.sdlc/*.yaml`. Invalid / non-record files are skipped.
pub fn read_all() -> Result<Vec<UiRecord>> {
    let dir = user_sdlc_dir()?;
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut records = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let data = match std::fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        if let Ok(record) = serde_yaml::from_str::<UiRecord>(&data) {
            records.push(record);
        }
    }
    Ok(records)
}

/// Find a record by project name.
pub fn find_by_name(name: &str) -> Result<Option<UiRecord>> {
    let path = user_ui_record_path(name)?;
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)?;
    let record: UiRecord = serde_yaml::from_str(&data)?;
    Ok(Some(record))
}

// ---------------------------------------------------------------------------
// PID helpers (Unix only)
// ---------------------------------------------------------------------------

/// Returns true if the process is still alive (`kill -0 {pid}`).
pub fn is_pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        // TODO: Windows — use winapi or tasklist
        let _ = pid;
        false
    }
}

/// Send SIGTERM to a process (`kill -TERM {pid}`).
pub fn kill_pid(pid: u32) -> Result<()> {
    #[cfg(unix)]
    {
        let status = std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()?;
        if !status.success() {
            return Err(SdlcError::Io(std::io::Error::other(format!(
                "kill -TERM {pid} failed with exit code {:?}",
                status.code()
            ))));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        // TODO: Windows
        let _ = pid;
        Err(SdlcError::Io(std::io::Error::other(
            "kill_pid is not supported on Windows",
        )))
    }
}
