//! PostgreSQL-backed orchestrator storage backend.
//!
//! Implements `OrchestratorBackend` using sqlx + Postgres. All trait methods
//! are synchronous (per the trait contract); async sqlx calls are bridged via
//! `block_on` using the ambient Tokio runtime when available, or a temporary
//! single-threaded runtime in CLI contexts.

use chrono::{DateTime, Utc};
use sdlc_core::error::{Result, SdlcError};
use sdlc_core::orchestrator::{
    Action, ActionStatus, OrchestratorBackend, WebhookEvent, WebhookPayload, WebhookRoute,
};
use sqlx::PgPool;
use uuid::Uuid;

/// Maximum number of events retained in the webhook event ring buffer.
const WEBHOOK_EVENTS_CAP: i64 = 500;

// ---------------------------------------------------------------------------
// PgOrchestratorBackend
// ---------------------------------------------------------------------------

pub struct PgOrchestratorBackend {
    pool: PgPool,
}

impl PgOrchestratorBackend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ---------------------------------------------------------------------------
// Row → struct helpers
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn row_to_action(
    id: Uuid,
    label: String,
    tool_name: String,
    tool_input: serde_json::Value,
    trigger: serde_json::Value,
    status: serde_json::Value,
    recurrence_secs: Option<i64>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<Action> {
    Ok(Action {
        id,
        label,
        tool_name,
        tool_input,
        trigger: serde_json::from_value(trigger)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?,
        status: serde_json::from_value(status)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?,
        recurrence: recurrence_secs.map(|s| std::time::Duration::from_secs(s as u64)),
        created_at,
        updated_at,
    })
}

fn row_to_webhook_payload(
    id: Uuid,
    route_path: String,
    raw_body: Vec<u8>,
    content_type: Option<String>,
    received_at: DateTime<Utc>,
) -> WebhookPayload {
    WebhookPayload {
        id,
        route_path,
        raw_body,
        content_type,
        received_at,
    }
}

fn row_to_webhook_route(
    id: Uuid,
    path: String,
    tool_name: String,
    input_template: String,
    created_at: DateTime<Utc>,
    store_only: bool,
    secret_token: Option<String>,
) -> WebhookRoute {
    WebhookRoute {
        id,
        path,
        tool_name,
        input_template,
        created_at,
        store_only,
        secret_token,
    }
}

fn row_to_webhook_event(
    id: Uuid,
    seq: i64,
    route_path: String,
    content_type: Option<String>,
    body_bytes: i64,
    outcome: serde_json::Value,
    received_at: DateTime<Utc>,
) -> Result<WebhookEvent> {
    Ok(WebhookEvent {
        id,
        seq: seq as u64,
        route_path,
        content_type,
        body_bytes: body_bytes as usize,
        outcome: serde_json::from_value(outcome)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?,
        received_at,
    })
}

// ---------------------------------------------------------------------------
// OrchestratorBackend impl
// ---------------------------------------------------------------------------

impl OrchestratorBackend for PgOrchestratorBackend {
    // -----------------------------------------------------------------------
    // Action operations
    // -----------------------------------------------------------------------

    fn insert(&self, action: &Action) -> Result<()> {
        let trigger = serde_json::to_value(&action.trigger)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let status = serde_json::to_value(&action.status)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let recurrence_secs = action.recurrence.map(|d| d.as_secs() as i64);
        let id = action.id;
        let label = action.label.clone();
        let tool_name = action.tool_name.clone();
        let tool_input = action.tool_input.clone();
        let created_at = action.created_at;
        let updated_at = action.updated_at;
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            sqlx::query(
                r#"INSERT INTO orchestrator_actions
                   (id, label, tool_name, tool_input, trigger, status, recurrence_secs, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            )
            .bind(id)
            .bind(&label)
            .bind(&tool_name)
            .bind(&tool_input)
            .bind(&trigger)
            .bind(&status)
            .bind(recurrence_secs)
            .bind(created_at)
            .bind(updated_at)
            .execute(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            Ok(())
        })
    }

    fn set_status(&self, id: Uuid, status: ActionStatus) -> Result<()> {
        let status_val =
            serde_json::to_value(&status).map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            sqlx::query(
                "UPDATE orchestrator_actions SET status = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(&status_val)
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            Ok(())
        })
    }

    fn range_due(&self, now: DateTime<Utc>) -> Result<Vec<Action>> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            // Fetch all pending actions — JSON path queries on JSONB for the trigger timestamp.
            // We filter in application code because the trigger type determines which field to use.
            let rows = sqlx::query(
                r#"SELECT id, label, tool_name, tool_input, trigger, status, recurrence_secs, created_at, updated_at
                   FROM orchestrator_actions
                   WHERE status->>'type' = 'pending'
                   ORDER BY created_at ASC"#,
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            let mut result = Vec::new();
            for row in rows {
                use sqlx::Row;
                let trigger_val: serde_json::Value = row
                    .try_get("trigger")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

                // Check if the trigger timestamp is <= now
                let ts_str = trigger_val
                    .get("next_tick_at")
                    .or_else(|| trigger_val.get("received_at"))
                    .and_then(|v| v.as_str());

                let is_due = match ts_str {
                    Some(s) => {
                        let ts: DateTime<Utc> = s.parse().map_err(|e: chrono::ParseError| {
                            SdlcError::OrchestratorDb(e.to_string())
                        })?;
                        ts <= now
                    }
                    None => false,
                };

                if is_due {
                    let id: Uuid = row
                        .try_get("id")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let label: String = row
                        .try_get("label")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let tool_name: String = row
                        .try_get("tool_name")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let tool_input: serde_json::Value = row
                        .try_get("tool_input")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let status_val: serde_json::Value = row
                        .try_get("status")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let recurrence_secs: Option<i64> = row
                        .try_get("recurrence_secs")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let created_at: DateTime<Utc> = row
                        .try_get("created_at")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let updated_at: DateTime<Utc> = row
                        .try_get("updated_at")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

                    result.push(row_to_action(
                        id,
                        label,
                        tool_name,
                        tool_input,
                        trigger_val,
                        status_val,
                        recurrence_secs,
                        created_at,
                        updated_at,
                    )?);
                }
            }
            Ok(result)
        })
    }

    fn startup_recovery(&self, max_age: std::time::Duration) -> Result<u32> {
        let cutoff = Utc::now()
            - chrono::Duration::from_std(max_age)
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let pool = self.pool.clone();
        let failed_status =
            serde_json::json!({"type": "failed", "reason": "recovered from restart"});

        crate::pg_common::block_on_pg(async move {
            let result = sqlx::query(
                r#"UPDATE orchestrator_actions
                   SET status = $1, updated_at = NOW()
                   WHERE status->>'type' = 'running' AND updated_at < $2"#,
            )
            .bind(&failed_status)
            .bind(cutoff)
            .execute(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            Ok(result.rows_affected() as u32)
        })
    }

    fn delete(&self, id: Uuid) -> Result<()> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            sqlx::query("DELETE FROM orchestrator_actions WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            Ok(())
        })
    }

    fn update_label_and_recurrence(
        &self,
        id: Uuid,
        label: Option<String>,
        recurrence: Option<Option<std::time::Duration>>,
    ) -> Result<Action> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            // Fetch existing action first
            let row = sqlx::query(
                r#"SELECT id, label, tool_name, tool_input, trigger, status, recurrence_secs, created_at, updated_at
                   FROM orchestrator_actions WHERE id = $1"#,
            )
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?
            .ok_or_else(|| SdlcError::OrchestratorDb(format!("action not found: {id}")))?;

            use sqlx::Row;
            let current_label: String = row
                .try_get("label")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let current_recurrence_secs: Option<i64> = row
                .try_get("recurrence_secs")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            let new_label = label.unwrap_or(current_label);
            let new_recurrence_secs = match recurrence {
                Some(Some(dur)) => Some(dur.as_secs() as i64),
                Some(None) => None,
                None => current_recurrence_secs,
            };

            let updated_row = sqlx::query(
                r#"UPDATE orchestrator_actions
                   SET label = $1, recurrence_secs = $2, updated_at = NOW()
                   WHERE id = $3
                   RETURNING id, label, tool_name, tool_input, trigger, status, recurrence_secs, created_at, updated_at"#,
            )
            .bind(&new_label)
            .bind(new_recurrence_secs)
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            let ret_id: Uuid = updated_row
                .try_get("id")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let ret_label: String = updated_row
                .try_get("label")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let ret_tool_name: String = updated_row
                .try_get("tool_name")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let ret_tool_input: serde_json::Value = updated_row
                .try_get("tool_input")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let ret_trigger: serde_json::Value = updated_row
                .try_get("trigger")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let ret_status: serde_json::Value = updated_row
                .try_get("status")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let ret_recurrence_secs: Option<i64> = updated_row
                .try_get("recurrence_secs")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let ret_created_at: DateTime<Utc> = updated_row
                .try_get("created_at")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            let ret_updated_at: DateTime<Utc> = updated_row
                .try_get("updated_at")
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            row_to_action(
                ret_id,
                ret_label,
                ret_tool_name,
                ret_tool_input,
                ret_trigger,
                ret_status,
                ret_recurrence_secs,
                ret_created_at,
                ret_updated_at,
            )
        })
    }

    fn list_all(&self) -> Result<Vec<Action>> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            let rows = sqlx::query(
                r#"SELECT id, label, tool_name, tool_input, trigger, status, recurrence_secs, created_at, updated_at
                   FROM orchestrator_actions
                   ORDER BY created_at DESC"#,
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            let mut result = Vec::new();
            for row in rows {
                use sqlx::Row;
                let id: Uuid = row
                    .try_get("id")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let label: String = row
                    .try_get("label")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let tool_name: String = row
                    .try_get("tool_name")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let tool_input: serde_json::Value = row
                    .try_get("tool_input")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let trigger_val: serde_json::Value = row
                    .try_get("trigger")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let status_val: serde_json::Value = row
                    .try_get("status")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let recurrence_secs: Option<i64> = row
                    .try_get("recurrence_secs")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let created_at: DateTime<Utc> = row
                    .try_get("created_at")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let updated_at: DateTime<Utc> = row
                    .try_get("updated_at")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                result.push(row_to_action(
                    id,
                    label,
                    tool_name,
                    tool_input,
                    trigger_val,
                    status_val,
                    recurrence_secs,
                    created_at,
                    updated_at,
                )?);
            }
            Ok(result)
        })
    }

    // -----------------------------------------------------------------------
    // Webhook payload operations
    // -----------------------------------------------------------------------

    fn insert_webhook(&self, payload: &WebhookPayload) -> Result<()> {
        let id = payload.id;
        let route_path = payload.route_path.clone();
        let raw_body = payload.raw_body.clone();
        let content_type = payload.content_type.clone();
        let received_at = payload.received_at;
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            sqlx::query(
                r#"INSERT INTO orchestrator_webhooks
                   (id, route_path, raw_body, content_type, received_at)
                   VALUES ($1, $2, $3, $4, $5)"#,
            )
            .bind(id)
            .bind(&route_path)
            .bind(&raw_body)
            .bind(content_type.as_deref())
            .bind(received_at)
            .execute(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            Ok(())
        })
    }

    fn all_pending_webhooks(&self) -> Result<Vec<WebhookPayload>> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            let rows = sqlx::query(
                "SELECT id, route_path, raw_body, content_type, received_at FROM orchestrator_webhooks ORDER BY received_at ASC",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            let mut result = Vec::new();
            for row in rows {
                use sqlx::Row;
                let id: Uuid = row
                    .try_get("id")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let route_path: String = row
                    .try_get("route_path")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let raw_body: Vec<u8> = row
                    .try_get("raw_body")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let content_type: Option<String> = row
                    .try_get("content_type")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let received_at: DateTime<Utc> = row
                    .try_get("received_at")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                result.push(row_to_webhook_payload(
                    id,
                    route_path,
                    raw_body,
                    content_type,
                    received_at,
                ));
            }
            Ok(result)
        })
    }

    fn delete_webhook(&self, id: Uuid) -> Result<()> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            sqlx::query("DELETE FROM orchestrator_webhooks WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            Ok(())
        })
    }

    fn query_webhooks(
        &self,
        route_path: &str,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<WebhookPayload>> {
        let route_path = route_path.to_string();
        let pool = self.pool.clone();
        let limit_i64 = limit as i64;

        crate::pg_common::block_on_pg(async move {
            // Build query dynamically: always filter by route_path, optionally
            // filter by since/until. Use positional params numbered from $2 onward.
            let mut sql = String::from(
                "SELECT id, route_path, raw_body, content_type, received_at \
                 FROM orchestrator_webhooks WHERE route_path = $1",
            );
            let mut param_idx = 2usize;

            if since.is_some() {
                sql.push_str(&format!(" AND received_at >= ${param_idx}"));
                param_idx += 1;
            }
            if until.is_some() {
                sql.push_str(&format!(" AND received_at <= ${param_idx}"));
                param_idx += 1;
            }
            sql.push_str(&format!(" ORDER BY received_at DESC LIMIT ${param_idx}"));

            let mut q = sqlx::query(&sql).bind(&route_path);
            if let Some(s) = since {
                q = q.bind(s);
            }
            if let Some(u) = until {
                q = q.bind(u);
            }
            q = q.bind(limit_i64);

            let rows = q
                .fetch_all(&pool)
                .await
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            let mut result = Vec::new();
            for row in rows {
                use sqlx::Row;
                let id: Uuid = row
                    .try_get("id")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let rp: String = row
                    .try_get("route_path")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let raw_body: Vec<u8> = row
                    .try_get("raw_body")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let content_type: Option<String> = row
                    .try_get("content_type")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let received_at: DateTime<Utc> = row
                    .try_get("received_at")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                result.push(row_to_webhook_payload(
                    id,
                    rp,
                    raw_body,
                    content_type,
                    received_at,
                ));
            }
            Ok(result)
        })
    }

    // -----------------------------------------------------------------------
    // Webhook route operations
    // -----------------------------------------------------------------------

    fn insert_route(&self, route: &WebhookRoute) -> Result<()> {
        let id = route.id;
        let path = route.path.clone();
        let tool_name = route.tool_name.clone();
        let input_template = route.input_template.clone();
        let created_at = route.created_at;
        let store_only = route.store_only;
        let secret_token = route.secret_token.clone();
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            sqlx::query(
                r#"INSERT INTO orchestrator_webhook_routes
                   (id, path, tool_name, template, created_at, store_only, secret_token)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            )
            .bind(id)
            .bind(&path)
            .bind(&tool_name)
            .bind(&input_template)
            .bind(created_at)
            .bind(store_only)
            .bind(secret_token.as_deref())
            .execute(&pool)
            .await
            .map_err(|e| {
                // Normalize postgres UNIQUE VIOLATION (23505) to the canonical error
                // message so the route handler can detect conflicts consistently
                // across both redb and postgres backends via string matching.
                if let Some(db_err) = e.as_database_error() {
                    if db_err
                        .code()
                        .map(|c| c.as_ref() == "23505")
                        .unwrap_or(false)
                    {
                        return SdlcError::OrchestratorDb(
                            "duplicate webhook route path".to_string(),
                        );
                    }
                }
                SdlcError::OrchestratorDb(e.to_string())
            })?;
            Ok(())
        })
    }

    fn list_routes(&self) -> Result<Vec<WebhookRoute>> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            let rows = sqlx::query(
                "SELECT id, path, tool_name, template, created_at, store_only, secret_token FROM orchestrator_webhook_routes ORDER BY created_at ASC",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            let mut result = Vec::new();
            for row in rows {
                use sqlx::Row;
                let id: Uuid = row
                    .try_get("id")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let path: String = row
                    .try_get("path")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let tool_name: String = row
                    .try_get("tool_name")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let template: String = row
                    .try_get("template")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let created_at: DateTime<Utc> = row
                    .try_get("created_at")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let store_only: bool = row
                    .try_get("store_only")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let secret_token: Option<String> = row
                    .try_get("secret_token")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                result.push(row_to_webhook_route(
                    id,
                    path,
                    tool_name,
                    template,
                    created_at,
                    store_only,
                    secret_token,
                ));
            }
            Ok(result)
        })
    }

    fn find_route_by_path(&self, path: &str) -> Result<Option<WebhookRoute>> {
        let path = path.to_string();
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            let row = sqlx::query(
                "SELECT id, path, tool_name, template, created_at, store_only, secret_token FROM orchestrator_webhook_routes WHERE path = $1",
            )
            .bind(&path)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            match row {
                None => Ok(None),
                Some(row) => {
                    use sqlx::Row;
                    let id: Uuid = row
                        .try_get("id")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let path: String = row
                        .try_get("path")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let tool_name: String = row
                        .try_get("tool_name")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let template: String = row
                        .try_get("template")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let created_at: DateTime<Utc> = row
                        .try_get("created_at")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let store_only: bool = row
                        .try_get("store_only")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    let secret_token: Option<String> = row
                        .try_get("secret_token")
                        .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                    Ok(Some(row_to_webhook_route(
                        id,
                        path,
                        tool_name,
                        template,
                        created_at,
                        store_only,
                        secret_token,
                    )))
                }
            }
        })
    }

    fn delete_route(&self, id: Uuid) -> Result<()> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            sqlx::query("DELETE FROM orchestrator_webhook_routes WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            Ok(())
        })
    }

    // -----------------------------------------------------------------------
    // Webhook event (audit ring buffer) operations
    // -----------------------------------------------------------------------

    fn insert_webhook_event(&self, event: &WebhookEvent) -> Result<()> {
        let route_path = event.route_path.clone();
        let content_type = event.content_type.clone();
        let body_bytes = event.body_bytes as i64;
        let outcome = serde_json::to_value(&event.outcome)
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
        let received_at = event.received_at;
        let id = event.id;
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            // Wrap the ring-buffer cap enforcement and insert in a single
            // transaction to prevent concurrent inserts from both skipping
            // the delete and temporarily exceeding the cap.
            let mut tx = pool
                .begin()
                .await
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            // Enforce ring buffer cap: delete oldest entry if at 500
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orchestrator_webhook_events")
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            if count >= WEBHOOK_EVENTS_CAP {
                sqlx::query(
                    "DELETE FROM orchestrator_webhook_events WHERE seq = (SELECT MIN(seq) FROM orchestrator_webhook_events)",
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            }

            // Insert new event (seq is BIGSERIAL, assigned by the DB)
            sqlx::query(
                r#"INSERT INTO orchestrator_webhook_events
                   (id, route_path, content_type, body_bytes, outcome, received_at)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(id)
            .bind(&route_path)
            .bind(content_type.as_deref())
            .bind(body_bytes)
            .bind(&outcome)
            .bind(received_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            tx.commit()
                .await
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            Ok(())
        })
    }

    fn list_webhook_events(&self) -> Result<Vec<WebhookEvent>> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            let rows = sqlx::query(
                "SELECT id, seq, route_path, content_type, body_bytes, outcome, received_at FROM orchestrator_webhook_events ORDER BY seq DESC",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;

            let mut result = Vec::new();
            for row in rows {
                use sqlx::Row;
                let id: Uuid = row
                    .try_get("id")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let seq: i64 = row
                    .try_get("seq")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let route_path: String = row
                    .try_get("route_path")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let content_type: Option<String> = row
                    .try_get("content_type")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let body_bytes: i64 = row
                    .try_get("body_bytes")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let outcome: serde_json::Value = row
                    .try_get("outcome")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                let received_at: DateTime<Utc> = row
                    .try_get("received_at")
                    .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
                result.push(row_to_webhook_event(
                    id,
                    seq,
                    route_path,
                    content_type,
                    body_bytes,
                    outcome,
                    received_at,
                )?);
            }
            Ok(result)
        })
    }

    fn webhook_event_count(&self) -> Result<u64> {
        let pool = self.pool.clone();

        crate::pg_common::block_on_pg(async move {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orchestrator_webhook_events")
                .fetch_one(&pool)
                .await
                .map_err(|e| SdlcError::OrchestratorDb(e.to_string()))?;
            Ok(count as u64)
        })
    }
}
