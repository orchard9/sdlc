//! Persistent storage for orchestrator actions using redb.
//!
//! # Table design
//!
//! A single `ACTIONS` table uses a 24-byte composite key:
//! ```text
//! [ timestamp_ms: u64 big-endian (8 bytes) | uuid: 16 bytes ]
//! ```
//!
//! Because the timestamp occupies the high bytes in big-endian encoding,
//! byte ordering equals timestamp ordering. A single range scan
//! `..=due_upper_bound(now)` returns all actions due by `now` without
//! any post-filtering for timestamp — only `Pending` status filtering
//! is needed in application code.

use std::{path::Path, time::Duration};

use chrono::{DateTime, Utc};
use redb::{Database, ReadableTable, TableDefinition};
use uuid::Uuid;

use crate::error::{Result, SdlcError};

use super::action::{Action, ActionStatus};

// ---------------------------------------------------------------------------
// Table definition
// ---------------------------------------------------------------------------

/// Key: 24-byte composite (timestamp_ms big-endian ++ uuid bytes)
/// Value: JSON-encoded Action
const ACTIONS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("actions");

// ---------------------------------------------------------------------------
// Key helpers
// ---------------------------------------------------------------------------

fn action_key(ts: DateTime<Utc>, id: Uuid) -> [u8; 24] {
    let mut key = [0u8; 24];
    let ms = ts.timestamp_millis().max(0) as u64;
    key[..8].copy_from_slice(&ms.to_be_bytes());
    key[8..].copy_from_slice(id.as_bytes());
    key
}

/// Upper bound for a range scan returning all actions due by `now`.
///
/// The UUID suffix is `0xff` × 16, which is greater than any valid UUID,
/// so all actions with `timestamp_ms <= now_ms` are included.
fn due_upper_bound(now: DateTime<Utc>) -> [u8; 24] {
    let mut key = [0u8; 24];
    let ms = now.timestamp_millis().max(0) as u64;
    key[..8].copy_from_slice(&ms.to_be_bytes());
    key[8..].fill(0xff);
    key
}

// ---------------------------------------------------------------------------
// ActionDb
// ---------------------------------------------------------------------------

/// Persistent store for orchestrator `Action` records.
pub struct ActionDb {
    db: Database,
}

impl ActionDb {
    /// Open or create the redb database at `path`.
    ///
    /// Creates the `ACTIONS` table if it doesn't already exist.
    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::create(path).map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        // Ensure the table exists before any reads
        let wt = db
            .begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        wt.open_table(ACTIONS)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(Self { db })
    }

    /// Insert a new action. The key is derived from the trigger timestamp.
    pub fn insert(&self, action: &Action) -> Result<()> {
        let key = action_key(action.trigger.key_ts(), action.id);
        let value =
            serde_json::to_vec(action).map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let wt = self
            .db
            .begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        {
            let mut table = wt
                .open_table(ACTIONS)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            table
                .insert(key.as_slice(), value.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        }
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(())
    }

    /// Update the status of an action identified by `id`.
    ///
    /// Finds the action by scanning all records, removes the old record,
    /// and reinserts with the updated status and `updated_at`.
    pub fn set_status(&self, id: Uuid, status: ActionStatus) -> Result<()> {
        let all = self.list_all()?;
        let mut action = all
            .into_iter()
            .find(|a| a.id == id)
            .ok_or_else(|| SdlcError::OrchestratorDb(format!("action not found: {id}")))?;

        let key = action_key(action.trigger.key_ts(), action.id);
        action.status = status;
        action.updated_at = Utc::now();

        let new_value =
            serde_json::to_vec(&action).map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

        let wt = self
            .db
            .begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        {
            let mut table = wt
                .open_table(ACTIONS)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            // Remove old entry and reinsert with same key but new value
            table
                .remove(key.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            table
                .insert(key.as_slice(), new_value.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        }
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(())
    }

    /// Return all `Pending` actions whose trigger timestamp is `<= now`.
    ///
    /// Results are in timestamp order (ascending) due to the composite key design.
    pub fn range_due(&self, now: DateTime<Utc>) -> Result<Vec<Action>> {
        let upper = due_upper_bound(now);
        let rt = self
            .db
            .begin_read()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let table = rt
            .open_table(ACTIONS)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

        let mut result = Vec::new();
        for entry in table
            .range(..=upper.as_slice())
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?
        {
            let (_, v) = entry.map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let action: Action = serde_json::from_slice(v.value())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            if matches!(action.status, ActionStatus::Pending) {
                result.push(action);
            }
        }
        Ok(result)
    }

    /// On daemon startup, mark any `Running` action older than `max_age` as `Failed`.
    ///
    /// Returns the number of actions recovered.
    pub fn startup_recovery(&self, max_age: Duration) -> Result<u32> {
        let cutoff = Utc::now()
            - chrono::Duration::from_std(max_age)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

        let all = self.list_all()?;
        let mut count = 0u32;
        for action in all {
            if matches!(action.status, ActionStatus::Running) && action.updated_at < cutoff {
                self.set_status(
                    action.id,
                    ActionStatus::Failed {
                        reason: "recovered from restart".into(),
                    },
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// List all actions, sorted by `created_at` descending (newest first).
    pub fn list_all(&self) -> Result<Vec<Action>> {
        let rt = self
            .db
            .begin_read()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let table = rt
            .open_table(ACTIONS)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

        let mut result = Vec::new();
        for entry in table
            .iter()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?
        {
            let (_, v) = entry.map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let action: Action = serde_json::from_slice(v.value())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            result.push(action);
        }
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::action::Action;
    use chrono::Duration as CDur;
    use tempfile::TempDir;

    fn open_tmp() -> (TempDir, ActionDb) {
        let dir = TempDir::new().unwrap();
        let db = ActionDb::open(&dir.path().join("test.db")).unwrap();
        (dir, db)
    }

    fn scheduled_at(label: &str, ts: DateTime<Utc>) -> Action {
        Action::new_scheduled(label, "quality-check", serde_json::json!({}), ts, None)
    }

    #[test]
    fn insert_and_range_due_returns_only_past_actions() {
        let (_dir, db) = open_tmp();
        let now = Utc::now();
        let early = scheduled_at("early", now - CDur::milliseconds(100));
        let late = scheduled_at("late", now + CDur::seconds(60));

        db.insert(&early).unwrap();
        db.insert(&late).unwrap();

        let due = db.range_due(now).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].label, "early");
    }

    #[test]
    fn range_due_excludes_non_pending() {
        let (_dir, db) = open_tmp();
        let now = Utc::now();
        let action = scheduled_at("running-action", now - CDur::seconds(1));
        db.insert(&action).unwrap();
        db.set_status(action.id, ActionStatus::Running).unwrap();

        let due = db.range_due(now).unwrap();
        assert!(
            due.is_empty(),
            "Running actions must not appear in range_due"
        );
    }

    #[test]
    fn composite_key_ordering_is_by_timestamp() {
        let (_dir, db) = open_tmp();
        let now = Utc::now();
        // Insert in reverse chronological order
        let second = scheduled_at("second", now - CDur::milliseconds(50));
        let first = scheduled_at("first", now - CDur::milliseconds(200));

        db.insert(&second).unwrap();
        db.insert(&first).unwrap();

        let due = db.range_due(now).unwrap();
        assert_eq!(due.len(), 2);
        // range_due returns in key order = timestamp order ascending
        assert_eq!(due[0].label, "first");
        assert_eq!(due[1].label, "second");
    }

    #[test]
    fn startup_recovery_marks_old_running_as_failed() {
        let (_dir, db) = open_tmp();
        let action = scheduled_at("stale", Utc::now() - CDur::minutes(1));
        db.insert(&action).unwrap();
        // Force status to Running with an old updated_at by manipulating via set_status
        // then manually backdating by reinserting
        db.set_status(action.id, ActionStatus::Running).unwrap();

        // Backdate the updated_at so it's older than max_age
        let all = db.list_all().unwrap();
        let mut stale = all.into_iter().find(|a| a.id == action.id).unwrap();
        stale.updated_at = Utc::now() - CDur::minutes(10);
        let key = action_key(stale.trigger.key_ts(), stale.id);
        let wt = db.db.begin_write().unwrap();
        {
            let mut table = wt.open_table(ACTIONS).unwrap();
            table.remove(key.as_slice()).unwrap();
            table
                .insert(
                    key.as_slice(),
                    serde_json::to_vec(&stale).unwrap().as_slice(),
                )
                .unwrap();
        }
        wt.commit().unwrap();

        let recovered = db.startup_recovery(Duration::from_secs(120)).unwrap();
        assert_eq!(recovered, 1);

        let all = db.list_all().unwrap();
        let recovered_action = all.into_iter().find(|a| a.id == action.id).unwrap();
        match &recovered_action.status {
            ActionStatus::Failed { reason } => {
                assert!(reason.contains("recovered"), "reason: {reason}");
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn startup_recovery_leaves_recent_running_alone() {
        let (_dir, db) = open_tmp();
        let action = scheduled_at("fresh", Utc::now() - CDur::seconds(5));
        db.insert(&action).unwrap();
        db.set_status(action.id, ActionStatus::Running).unwrap();

        // max_age = 2 minutes, action was updated 5 seconds ago → should NOT recover
        let recovered = db.startup_recovery(Duration::from_secs(120)).unwrap();
        assert_eq!(recovered, 0);

        let all = db.list_all().unwrap();
        let still_running = all.into_iter().find(|a| a.id == action.id).unwrap();
        assert!(matches!(still_running.status, ActionStatus::Running));
    }

    #[test]
    fn empty_db_range_due_returns_empty() {
        let (_dir, db) = open_tmp();
        let due = db.range_due(Utc::now()).unwrap();
        assert!(due.is_empty());
    }

    #[test]
    fn startup_recovery_on_empty_db_returns_zero() {
        let (_dir, db) = open_tmp();
        let n = db.startup_recovery(Duration::from_secs(60)).unwrap();
        assert_eq!(n, 0);
    }
}
