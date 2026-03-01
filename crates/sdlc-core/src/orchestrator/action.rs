//! Action data model for the tick-rate orchestrator.
//!
//! An `Action` is the atomic unit of orchestration: a trigger condition (when
//! to fire) paired with a tool name and input (what to run). The orchestrator
//! tick loop queries `ActionDb` for due actions and calls `run_tool()` for each.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Duration;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// ActionTrigger
// ---------------------------------------------------------------------------

/// Determines when an action becomes due.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionTrigger {
    /// Fires when the tick loop runs at or after `next_tick_at`.
    Scheduled { next_tick_at: DateTime<Utc> },
    /// Fires on the next tick after a webhook payload was received.
    Webhook {
        raw_payload: Vec<u8>,
        received_at: DateTime<Utc>,
    },
}

impl ActionTrigger {
    /// The timestamp used as the redb key prefix.
    ///
    /// For `Scheduled`, this is `next_tick_at`. For `Webhook`, this is
    /// `received_at` — ensuring webhook payloads appear in the very next tick.
    pub fn key_ts(&self) -> DateTime<Utc> {
        match self {
            Self::Scheduled { next_tick_at } => *next_tick_at,
            Self::Webhook { received_at, .. } => *received_at,
        }
    }
}

// ---------------------------------------------------------------------------
// ActionStatus
// ---------------------------------------------------------------------------

/// Lifecycle state of an action.
///
/// Transitions: `Pending → Running → Completed | Failed`
///
/// The tick loop writes `Running` *before* dispatching the tool. On restart,
/// any action stuck in `Running` is recovered to `Failed` by `startup_recovery`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionStatus {
    /// Waiting to be dispatched.
    Pending,
    /// Currently executing (or crashed before completing).
    Running,
    /// Tool ran and returned a result.
    Completed { result: serde_json::Value },
    /// Tool failed or the process was interrupted.
    Failed { reason: String },
}

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

/// An orchestration action: a trigger condition + a tool to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: Uuid,
    /// Human-readable label (e.g. "my-service" or "nightly-audit").
    pub label: String,
    /// Tool slug matching a directory under `.sdlc/tools/<name>/`.
    pub tool_name: String,
    /// JSON value passed to the tool via stdin in `--run` mode.
    pub tool_input: serde_json::Value,
    pub trigger: ActionTrigger,
    pub status: ActionStatus,
    /// If set, a new `Pending` action is created after `Completed`
    /// with `next_tick_at = now + recurrence`.
    #[serde(
        serialize_with = "serialize_duration_opt",
        deserialize_with = "deserialize_duration_opt",
        default
    )]
    pub recurrence: Option<Duration>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Action {
    /// Create a new scheduled action in `Pending` state.
    pub fn new_scheduled(
        label: impl Into<String>,
        tool_name: impl Into<String>,
        tool_input: serde_json::Value,
        next_tick_at: DateTime<Utc>,
        recurrence: Option<Duration>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            tool_name: tool_name.into(),
            tool_input,
            trigger: ActionTrigger::Scheduled { next_tick_at },
            status: ActionStatus::Pending,
            recurrence,
            created_at: now,
            updated_at: now,
        }
    }
}

// ---------------------------------------------------------------------------
// Serde helpers for Duration (serialized as seconds: u64)
// ---------------------------------------------------------------------------

fn serialize_duration_opt<S>(d: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match d {
        Some(dur) => s.serialize_some(&dur.as_secs()),
        None => s.serialize_none(),
    }
}

fn deserialize_duration_opt<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<u64> = Option::deserialize(d)?;
    Ok(opt.map(Duration::from_secs))
}
