//! PostgreSQL-backed telemetry event storage.
//!
//! `PgTelemetryBackend` implements `TelemetryBackend` using a `sqlx::PgPool`.
//! All trait methods are synchronous and bridge async sqlx calls via
//! `crate::pg_common::block_on_pg`.
//!
//! Schema (from `crates/sdlc-server/migrations/001_telemetry.sql`):
//! ```sql
//! telemetry_events(run_id TEXT, seq BIGSERIAL, event JSONB, PRIMARY KEY(run_id, seq))
//! ```

use sdlc_core::telemetry_backend::{RunSummary, TelemetryBackend};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::collections::HashSet;

/// PostgreSQL-backed implementation of `TelemetryBackend`.
pub struct PgTelemetryBackend {
    pool: PgPool,
}

impl PgTelemetryBackend {
    /// Wrap an existing `PgPool`.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl TelemetryBackend for PgTelemetryBackend {
    fn append_raw(&self, run_id: &str, event: Value) -> anyhow::Result<()> {
        let run_id = run_id.to_string();
        let pool = self.pool.clone();
        crate::pg_common::block_on_pg(async move {
            sqlx::query("INSERT INTO telemetry_events(run_id, event) VALUES ($1, $2)")
                .bind(&run_id)
                .bind(&event)
                .execute(&pool)
                .await?;
            Ok(())
        })
    }

    fn events_for_run(&self, run_id: &str) -> anyhow::Result<Vec<Value>> {
        let run_id = run_id.to_string();
        let pool = self.pool.clone();
        crate::pg_common::block_on_pg(async move {
            use sqlx::Row;
            let rows = sqlx::query(
                "SELECT event FROM telemetry_events WHERE run_id = $1 ORDER BY seq ASC",
            )
            .bind(&run_id)
            .fetch_all(&pool)
            .await?;
            Ok(rows
                .into_iter()
                .map(|r| {
                    r.try_get::<Value, _>("event")
                        .map_err(|e| {
                            tracing::warn!(run_id = %run_id, error = %e, "corrupt telemetry event in db — skipping");
                            e
                        })
                        .unwrap_or(Value::Null)
                })
                .collect())
        })
    }

    fn summary_for_run(&self, run_id: &str) -> anyhow::Result<RunSummary> {
        let events = self.events_for_run(run_id)?;
        let mut summary = RunSummary {
            tool_calls: 0,
            tool_errors: 0,
            tools_used: HashMap::new(),
            subagents_spawned: 0,
            subagent_tokens: 0,
            total_cost_usd: None,
            total_turns: None,
        };
        for ev in &events {
            let ty = ev.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match ty {
                "assistant" => {
                    if let Some(tools) = ev.get("tools").and_then(|v| v.as_array()) {
                        for tool in tools {
                            summary.tool_calls += 1;
                            if let Some(name) = tool.get("name").and_then(|v| v.as_str()) {
                                *summary.tools_used.entry(name.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                "user" => {
                    if let Some(results) = ev.get("tool_results").and_then(|v| v.as_array()) {
                        for result in results {
                            if result
                                .get("is_error")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false)
                            {
                                summary.tool_errors += 1;
                            }
                        }
                    }
                }
                "subagent_started" => {
                    summary.subagents_spawned += 1;
                }
                "subagent_completed" | "subagent_progress" => {
                    if let Some(tokens) = ev.get("total_tokens").and_then(|v| v.as_u64()) {
                        summary.subagent_tokens = summary.subagent_tokens.saturating_add(tokens);
                    }
                }
                "result" => {
                    if let Some(cost) = ev.get("cost_usd").and_then(|v| v.as_f64()) {
                        summary.total_cost_usd = Some(cost);
                    }
                    if let Some(turns) = ev.get("turns").and_then(|v| v.as_u64()) {
                        summary.total_turns = Some(turns);
                    }
                }
                _ => {}
            }
        }
        Ok(summary)
    }

    fn prune_runs_not_in(&self, keep_ids: &HashSet<String>) -> anyhow::Result<()> {
        let keep: Vec<String> = keep_ids.iter().cloned().collect();
        let pool = self.pool.clone();
        crate::pg_common::block_on_pg(async move {
            let keep_refs: Vec<&str> = keep.iter().map(|s| s.as_str()).collect();
            sqlx::query("DELETE FROM telemetry_events WHERE run_id != ALL($1)")
                .bind(&keep_refs as &[&str])
                .execute(&pool)
                .await?;
            Ok(())
        })
    }
}
