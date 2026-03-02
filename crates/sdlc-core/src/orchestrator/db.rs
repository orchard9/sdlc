//! Persistent storage for orchestrator actions and webhook payloads using redb.
//!
//! # Table design
//!
//! ## ACTIONS table
//!
//! A 24-byte composite key:
//! ```text
//! [ timestamp_ms: u64 big-endian (8 bytes) | uuid: 16 bytes ]
//! ```
//!
//! Because the timestamp occupies the high bytes in big-endian encoding,
//! byte ordering equals timestamp ordering. A single range scan
//! `..=due_upper_bound(now)` returns all actions due by `now` without
//! any post-filtering for timestamp — only `Pending` status filtering
//! is needed in application code.
//!
//! ## WEBHOOKS table
//!
//! A 16-byte UUID key (raw bytes). Webhook payloads do not need ordered range
//! scans — they need O(1) lookup by ID for delete-after-dispatch. Full scan
//! via `all_pending_webhooks()` is sorted by `received_at` in application code.

use std::{path::Path, time::Duration};

use chrono::{DateTime, Utc};
use redb::{Database, ReadableTable, TableDefinition};
use uuid::Uuid;

use crate::error::{Result, SdlcError};

use super::action::{Action, ActionStatus};
use super::webhook::{WebhookPayload, WebhookRoute};

// ---------------------------------------------------------------------------
// Table definitions
// ---------------------------------------------------------------------------

/// Key: 24-byte composite (timestamp_ms big-endian ++ uuid bytes)
/// Value: JSON-encoded Action
const ACTIONS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("actions");

/// Key: 16-byte UUID (raw bytes)
/// Value: JSON-encoded WebhookPayload
const WEBHOOKS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("webhooks");

/// Key: 16-byte UUID (raw bytes)
/// Value: JSON-encoded WebhookRoute
const WEBHOOK_ROUTES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("webhook_routes");

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
    /// Creates the `ACTIONS`, `WEBHOOKS`, and `WEBHOOK_ROUTES` tables if they
    /// don't already exist. Safe to call on an existing database — redb creates
    /// missing tables in place without affecting existing data.
    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::create(path).map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        // Ensure all tables exist before any reads
        let wt = db
            .begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        wt.open_table(ACTIONS)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        wt.open_table(WEBHOOKS)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        wt.open_table(WEBHOOK_ROUTES)
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

    // -----------------------------------------------------------------------
    // Webhook storage
    // -----------------------------------------------------------------------

    /// Store a raw webhook payload in the `WEBHOOKS` table.
    ///
    /// The key is the 16-byte UUID of the payload. The value is the
    /// JSON-encoded `WebhookPayload`.
    pub fn insert_webhook(&self, payload: &WebhookPayload) -> Result<()> {
        let key = payload.id.as_bytes().to_vec();
        let value =
            serde_json::to_vec(payload).map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let wt = self
            .db
            .begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        {
            let mut table = wt
                .open_table(WEBHOOKS)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            table
                .insert(key.as_slice(), value.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        }
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(())
    }

    /// Return all stored webhook payloads, sorted by `received_at` ascending.
    ///
    /// Intended for use by the tick loop to consume pending webhooks.
    pub fn all_pending_webhooks(&self) -> Result<Vec<WebhookPayload>> {
        let rt = self
            .db
            .begin_read()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let table = rt
            .open_table(WEBHOOKS)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

        let mut result = Vec::new();
        for entry in table
            .iter()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?
        {
            let (_, v) = entry.map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let payload: WebhookPayload = serde_json::from_slice(v.value())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            result.push(payload);
        }
        result.sort_by(|a, b| a.received_at.cmp(&b.received_at));
        Ok(result)
    }

    /// Delete a webhook payload by ID after successful dispatch.
    ///
    /// Silently succeeds if the ID does not exist (idempotent).
    pub fn delete_webhook(&self, id: Uuid) -> Result<()> {
        let key = id.as_bytes().to_vec();
        let wt = self
            .db
            .begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        {
            let mut table = wt
                .open_table(WEBHOOKS)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            table
                .remove(key.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        }
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Webhook route storage
    // -----------------------------------------------------------------------

    /// Insert a new webhook route.
    ///
    /// Returns an `OrchestratorDb` error if a route with the same `path` is
    /// already registered. Use the error message to detect duplicates.
    pub fn insert_route(&self, route: &WebhookRoute) -> Result<()> {
        // Check for duplicate path
        if self.find_route_by_path(&route.path)?.is_some() {
            return Err(SdlcError::OrchestratorDb(format!(
                "duplicate webhook route path: {}",
                route.path
            )));
        }
        let key = route.id.as_bytes().to_vec();
        let value =
            serde_json::to_vec(route).map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let wt = self
            .db
            .begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        {
            let mut table = wt
                .open_table(WEBHOOK_ROUTES)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            table
                .insert(key.as_slice(), value.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        }
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(())
    }

    /// Return all registered webhook routes, sorted by `created_at` ascending.
    pub fn list_routes(&self) -> Result<Vec<WebhookRoute>> {
        let rt = self
            .db
            .begin_read()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let table = rt
            .open_table(WEBHOOK_ROUTES)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

        let mut result = Vec::new();
        for entry in table
            .iter()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?
        {
            let (_, v) = entry.map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let route: WebhookRoute = serde_json::from_slice(v.value())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            result.push(route);
        }
        result.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(result)
    }

    /// Find the first route whose `path` matches `path`.
    ///
    /// Returns `Ok(None)` if no route is registered for the path.
    pub fn find_route_by_path(&self, path: &str) -> Result<Option<WebhookRoute>> {
        let all = self.list_routes()?;
        Ok(all.into_iter().find(|r| r.path == path))
    }

    /// Delete a webhook route by ID.
    ///
    /// Silently succeeds if the ID does not exist (idempotent).
    pub fn delete_route(&self, id: Uuid) -> Result<()> {
        let key = id.as_bytes().to_vec();
        let wt = self
            .db
            .begin_write()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        {
            let mut table = wt
                .open_table(WEBHOOK_ROUTES)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            table
                .remove(key.as_slice())
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        }
        wt.commit()
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        Ok(())
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

    // -----------------------------------------------------------------------
    // Webhook tests
    // -----------------------------------------------------------------------

    fn make_webhook(route: &str, body: &[u8]) -> WebhookPayload {
        WebhookPayload::new(route, body.to_vec(), Some("application/json".to_string()))
    }

    #[test]
    fn webhook_insert_and_retrieve_round_trip() {
        let (_dir, db) = open_tmp();
        let payload = make_webhook("github", b"{\"event\":\"push\"}");
        let id = payload.id;
        let route = payload.route_path.clone();
        let body = payload.raw_body.clone();
        let ct = payload.content_type.clone();

        db.insert_webhook(&payload).unwrap();

        let all = db.all_pending_webhooks().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, id);
        assert_eq!(all[0].route_path, route);
        assert_eq!(all[0].raw_body, body);
        assert_eq!(all[0].content_type, ct);
    }

    #[test]
    fn webhook_delete_removes_record() {
        let (_dir, db) = open_tmp();
        let payload = make_webhook("stripe", b"{}");
        let id = payload.id;

        db.insert_webhook(&payload).unwrap();
        assert_eq!(db.all_pending_webhooks().unwrap().len(), 1);

        db.delete_webhook(id).unwrap();
        assert!(db.all_pending_webhooks().unwrap().is_empty());
    }

    #[test]
    fn webhook_multiple_payloads_sorted_by_received_at() {
        let (_dir, db) = open_tmp();

        let mut first = make_webhook("ci", b"first");
        let mut second = make_webhook("ci", b"second");

        // Manually set received_at to control ordering
        first.received_at = Utc::now() - CDur::seconds(10);
        second.received_at = Utc::now();

        // Insert in reverse order to confirm sorting works
        db.insert_webhook(&second).unwrap();
        db.insert_webhook(&first).unwrap();

        let all = db.all_pending_webhooks().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].raw_body, b"first");
        assert_eq!(all[1].raw_body, b"second");
    }

    #[test]
    fn empty_db_all_pending_webhooks_returns_empty() {
        let (_dir, db) = open_tmp();
        let webhooks = db.all_pending_webhooks().unwrap();
        assert!(webhooks.is_empty());
    }

    #[test]
    fn existing_db_open_adds_webhooks_table() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        // First open — creates ACTIONS and WEBHOOKS tables
        {
            let db = ActionDb::open(&db_path).unwrap();
            let payload = make_webhook("test", b"hello");
            db.insert_webhook(&payload).unwrap();
        }

        // Second open — WEBHOOKS table already exists; must not fail
        {
            let db = ActionDb::open(&db_path).unwrap();
            let all = db.all_pending_webhooks().unwrap();
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].raw_body, b"hello");
        }
    }

    #[test]
    fn webhook_delete_nonexistent_is_idempotent() {
        let (_dir, db) = open_tmp();
        let id = uuid::Uuid::new_v4();
        // Should not panic or error
        db.delete_webhook(id).unwrap();
    }

    // -----------------------------------------------------------------------
    // Webhook route tests
    // -----------------------------------------------------------------------

    fn make_route(path: &str) -> WebhookRoute {
        WebhookRoute::new(path, "my-tool", r#"{"body": {{payload}}}"#)
    }

    #[test]
    fn route_insert_and_find_by_path() {
        let (_dir, db) = open_tmp();
        let route = make_route("/hooks/github");
        db.insert_route(&route).unwrap();

        let found = db.find_route_by_path("/hooks/github").unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.path, "/hooks/github");
        assert_eq!(found.tool_name, "my-tool");
    }

    #[test]
    fn route_duplicate_path_returns_error() {
        let (_dir, db) = open_tmp();
        let route1 = make_route("/hooks/stripe");
        let route2 = make_route("/hooks/stripe");
        db.insert_route(&route1).unwrap();
        let err = db.insert_route(&route2).unwrap_err();
        assert!(
            matches!(err, SdlcError::OrchestratorDb(_)),
            "expected OrchestratorDb error for duplicate path"
        );
    }

    #[test]
    fn route_list_sorted_by_created_at() {
        let (_dir, db) = open_tmp();

        let mut first = make_route("/hooks/first");
        let mut second = make_route("/hooks/second");

        // Manually set created_at to control ordering
        first.created_at = Utc::now() - CDur::seconds(10);
        second.created_at = Utc::now();

        // Insert second first to verify sorting works regardless of insert order
        db.insert_route(&second).unwrap();
        db.insert_route(&first).unwrap();

        let routes = db.list_routes().unwrap();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].path, "/hooks/first");
        assert_eq!(routes[1].path, "/hooks/second");
    }

    #[test]
    fn route_find_not_found_returns_none() {
        let (_dir, db) = open_tmp();
        let found = db.find_route_by_path("/hooks/nonexistent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn route_delete_removes_record() {
        let (_dir, db) = open_tmp();
        let route = make_route("/hooks/to-delete");
        let id = route.id;
        db.insert_route(&route).unwrap();
        assert_eq!(db.list_routes().unwrap().len(), 1);

        db.delete_route(id).unwrap();
        assert!(db.list_routes().unwrap().is_empty());
    }

    #[test]
    fn route_delete_nonexistent_is_idempotent() {
        let (_dir, db) = open_tmp();
        let id = uuid::Uuid::new_v4();
        db.delete_route(id).unwrap();
    }

    #[test]
    fn existing_db_open_adds_webhook_routes_table() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test2.db");

        // First open: creates ACTIONS + WEBHOOKS + WEBHOOK_ROUTES
        {
            let db = ActionDb::open(&db_path).unwrap();
            let route = make_route("/hooks/init");
            db.insert_route(&route).unwrap();
        }

        // Second open: WEBHOOK_ROUTES table already exists; data survives
        {
            let db = ActionDb::open(&db_path).unwrap();
            let routes = db.list_routes().unwrap();
            assert_eq!(routes.len(), 1);
            assert_eq!(routes[0].path, "/hooks/init");
        }
    }
}
