//! Tick-rate orchestrator data model.
//!
//! Provides `Action`, `ActionTrigger`, `ActionStatus`, `ActionDb`,
//! `WebhookPayload`, and `WebhookRoute` — the complete data layer for the
//! orchestrator. The tick loop in `sdlc orchestrate` uses these types to query
//! due actions and persist results. Webhook payloads arrive via
//! `POST /api/orchestrator/webhooks/*path` and are stored in redb until the
//! tick loop matches them against registered `WebhookRoute`s and dispatches.

pub mod action;
pub mod db;
pub mod webhook;

pub use action::{Action, ActionStatus, ActionTrigger};
pub use db::ActionDb;
pub use webhook::{WebhookPayload, WebhookRoute};
