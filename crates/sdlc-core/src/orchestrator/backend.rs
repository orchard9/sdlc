//! Abstract backend trait for orchestrator storage.
//!
//! Defines `OrchestratorBackend` so that the redb implementation (`ActionDb`
//! in `db.rs`) and future backends (e.g. PostgreSQL) share a common interface.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;

use super::action::{Action, ActionStatus};
use super::webhook::{WebhookEvent, WebhookPayload, WebhookRoute};

/// Pluggable storage backend for orchestrator actions and webhook data.
///
/// All methods are synchronous — callers in async contexts must wrap calls
/// with `tokio::task::spawn_blocking`.
pub trait OrchestratorBackend: Send + Sync {
    // -----------------------------------------------------------------------
    // Action operations
    // -----------------------------------------------------------------------

    /// Insert a new action.
    fn insert(&self, action: &Action) -> Result<()>;

    /// Update the status of an action identified by `id`.
    fn set_status(&self, id: Uuid, status: ActionStatus) -> Result<()>;

    /// Return all `Pending` actions whose trigger timestamp is `<= now`.
    fn range_due(&self, now: DateTime<Utc>) -> Result<Vec<Action>>;

    /// On daemon startup, mark any `Running` action older than `max_age` as `Failed`.
    ///
    /// Returns the number of actions recovered.
    fn startup_recovery(&self, max_age: std::time::Duration) -> Result<u32>;

    /// Delete an action by UUID.
    ///
    /// Silently succeeds if the ID does not exist (idempotent).
    fn delete(&self, id: Uuid) -> Result<()>;

    /// Update the `label` and/or `recurrence` of an action identified by `id`.
    ///
    /// - `label`: if `Some`, the label is replaced; if `None`, it is unchanged.
    /// - `recurrence`: if `Some(Some(dur))`, sets recurrence; `Some(None)` clears it;
    ///   `None` leaves the current value unchanged.
    ///
    /// Returns the updated `Action`, or an error if not found.
    fn update_label_and_recurrence(
        &self,
        id: Uuid,
        label: Option<String>,
        recurrence: Option<Option<std::time::Duration>>,
    ) -> Result<Action>;

    /// Return all actions sorted by `created_at` descending (newest first).
    fn list_all(&self) -> Result<Vec<Action>>;

    // -----------------------------------------------------------------------
    // Webhook payload operations
    // -----------------------------------------------------------------------

    /// Store a raw webhook payload.
    fn insert_webhook(&self, payload: &WebhookPayload) -> Result<()>;

    /// Return all stored webhook payloads, sorted by `received_at` ascending.
    fn all_pending_webhooks(&self) -> Result<Vec<WebhookPayload>>;

    /// Delete a webhook payload by ID after successful dispatch.
    ///
    /// Silently succeeds if the ID does not exist (idempotent).
    fn delete_webhook(&self, id: Uuid) -> Result<()>;

    /// Query stored webhook payloads for a route within an optional time window.
    ///
    /// - `route_path`: must start with `/` — exact match on stored `route_path`.
    /// - `since` / `until`: inclusive bounds on `received_at`; `None` means unbounded.
    /// - `limit`: maximum number of payloads to return (results are newest-first).
    fn query_webhooks(
        &self,
        route_path: &str,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<WebhookPayload>>;

    // -----------------------------------------------------------------------
    // Webhook route operations
    // -----------------------------------------------------------------------

    /// Insert a new webhook route.
    fn insert_route(&self, route: &WebhookRoute) -> Result<()>;

    /// Return all registered webhook routes, sorted by `created_at` ascending.
    fn list_routes(&self) -> Result<Vec<WebhookRoute>>;

    /// Find the first route whose `path` matches `path`.
    ///
    /// Returns `Ok(None)` if no route is registered for the path.
    fn find_route_by_path(&self, path: &str) -> Result<Option<WebhookRoute>>;

    /// Delete a webhook route by ID.
    ///
    /// Silently succeeds if the ID does not exist (idempotent).
    fn delete_route(&self, id: Uuid) -> Result<()>;

    // -----------------------------------------------------------------------
    // Webhook event (audit ring buffer) operations
    // -----------------------------------------------------------------------

    /// Insert a `WebhookEvent` into the audit ring buffer.
    fn insert_webhook_event(&self, event: &WebhookEvent) -> Result<()>;

    /// Return all stored webhook events, newest first.
    fn list_webhook_events(&self) -> Result<Vec<WebhookEvent>>;

    /// Return the number of events currently stored in the ring buffer.
    fn webhook_event_count(&self) -> Result<u64>;
}
