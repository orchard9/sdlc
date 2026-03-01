//! Tick-rate orchestrator data model.
//!
//! Provides `Action`, `ActionTrigger`, `ActionStatus`, and `ActionDb` — the
//! complete data layer for the orchestrator. The tick loop in `sdlc orchestrate`
//! uses these types to query due actions and persist results.

pub mod action;
pub mod db;

pub use action::{Action, ActionStatus, ActionTrigger};
pub use db::ActionDb;
