//! Abstract backend trait for telemetry event storage.
//!
//! Defines the `TelemetryBackend` trait and the `RunSummary` struct so that
//! both the redb implementation (in `sdlc-server`) and future backends (e.g.
//! PostgreSQL) can share the same interface without depending on each other.

use std::collections::HashMap;

/// Aggregated stats for a single agent run.
#[derive(Debug, serde::Serialize)]
pub struct RunSummary {
    pub tool_calls: u64,
    pub tool_errors: u64,
    pub tools_used: HashMap<String, u64>,
    pub subagents_spawned: u64,
    pub subagent_tokens: u64,
    pub total_cost_usd: Option<f64>,
    pub total_turns: Option<u64>,
}

/// Pluggable storage backend for raw agent telemetry events.
///
/// All methods are synchronous — callers in async contexts must wrap calls
/// with `tokio::task::spawn_blocking`.
pub trait TelemetryBackend: Send + Sync {
    /// Append a raw JSON event for `run_id`.
    fn append_raw(&self, run_id: &str, event: serde_json::Value) -> anyhow::Result<()>;

    /// Return all events for `run_id` in sequence order.
    fn events_for_run(&self, run_id: &str) -> anyhow::Result<Vec<serde_json::Value>>;

    /// Aggregate all events for `run_id` into a `RunSummary`.
    fn summary_for_run(&self, run_id: &str) -> anyhow::Result<RunSummary>;

    /// Delete all events whose run_id is NOT in `keep_ids`.
    ///
    /// Safe to call with an empty `keep_ids` — it will delete everything.
    fn prune_runs_not_in(&self, keep_ids: &std::collections::HashSet<String>)
        -> anyhow::Result<()>;
}
