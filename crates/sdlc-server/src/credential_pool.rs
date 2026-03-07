use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tracing::{debug, info, warn};

/// A single Claude credential checked out from the pool.
#[derive(Debug, Clone)]
pub struct ClaudeCredential {
    pub id: i64,
    pub account_name: String,
    pub token: String,
}

/// PostgreSQL-backed credential pool for Claude OAuth tokens.
///
/// Uses round-robin checkout (least recently used) so multiple pods can run
/// agents concurrently without overloading any single Claude account.
pub struct CredentialPool {
    pool: PgPool,
}

impl CredentialPool {
    /// Connect to the given `database_url` with a max of 5 connections.
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        info!("credential pool connected to database");
        Ok(Self { pool })
    }

    /// Create the `claude_credentials` table and index if they do not exist.
    ///
    /// Safe to call multiple times (idempotent).
    pub async fn initialize_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS claude_credentials (
                id           BIGSERIAL   PRIMARY KEY,
                account_name TEXT        NOT NULL,
                token        TEXT        NOT NULL,
                is_active    BOOLEAN     NOT NULL DEFAULT true,
                last_used_at TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01',
                use_count    BIGINT      NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS claude_credentials_lru_idx
            ON claude_credentials (last_used_at ASC)
            WHERE is_active
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("credential pool schema initialized");
        Ok(())
    }

    /// Check out the least-recently-used active credential using SELECT FOR UPDATE SKIP LOCKED.
    ///
    /// Updates `last_used_at` and `use_count` atomically in the same transaction.
    /// Returns `None` if no active credentials exist or all are currently locked.
    pub async fn checkout(&self) -> Result<Option<ClaudeCredential>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query(
            r#"
            SELECT id, account_name, token
            FROM claude_credentials
            WHERE is_active
            ORDER BY last_used_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let row = match row {
            Some(r) => r,
            None => {
                tx.rollback().await?;
                warn!("no active Claude credentials available in pool");
                return Ok(None);
            }
        };

        let id: i64 = row.try_get("id")?;
        let account_name: String = row.try_get("account_name")?;
        let token: String = row.try_get("token")?;

        sqlx::query(
            r#"
            UPDATE claude_credentials
            SET last_used_at = NOW(), use_count = use_count + 1
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        debug!(account_name = %account_name, "credential checked out");
        Ok(Some(ClaudeCredential {
            id,
            account_name,
            token,
        }))
    }

    /// List all credentials (token field omitted — never returned over the wire).
    pub async fn list(&self) -> Result<Vec<CredentialRow>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, account_name, is_active, last_used_at, use_count
            FROM claude_credentials
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| {
                Ok(CredentialRow {
                    id: r.try_get("id")?,
                    account_name: r.try_get("account_name")?,
                    is_active: r.try_get("is_active")?,
                    last_used_at: r
                        .try_get::<chrono::DateTime<chrono::Utc>, _>("last_used_at")?
                        .to_rfc3339(),
                    use_count: r.try_get("use_count")?,
                })
            })
            .collect()
    }

    /// Insert a new credential. Returns the new row's id.
    pub async fn add(&self, account_name: &str, token: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO claude_credentials (account_name, token)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(account_name)
        .bind(token)
        .fetch_one(&self.pool)
        .await?;
        info!(account_name = %account_name, "added Claude credential to pool");
        row.try_get("id")
    }

    /// Toggle the `is_active` flag for a credential.
    /// Returns true if a row was updated, false if the id was not found.
    pub async fn set_active(&self, id: i64, is_active: bool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"UPDATE claude_credentials SET is_active = $1 WHERE id = $2"#)
            .bind(is_active)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Delete a credential by id. Returns true if a row was deleted.
    pub async fn delete(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM claude_credentials WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        info!(credential_id = id, "credential deleted");
        Ok(result.rows_affected() > 0)
    }

    /// Pool status: total and active credential counts.
    pub async fn status(&self) -> Result<PoolStatus, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) AS total,
                   COUNT(*) FILTER (WHERE is_active) AS active
            FROM claude_credentials
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(PoolStatus {
            total: row.try_get::<i64, _>("total")? as u64,
            active: row.try_get::<i64, _>("active")? as u64,
        })
    }
}

/// A credential row returned by list() — token field intentionally omitted.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CredentialRow {
    pub id: i64,
    pub account_name: String,
    pub is_active: bool,
    pub last_used_at: String,
    pub use_count: i64,
}

/// Aggregate pool statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolStatus {
    pub total: u64,
    pub active: u64,
}

/// Wrapper that holds an optional credential pool.
///
/// When `DATABASE_URL` is absent or the database is unreachable, this degrades
/// gracefully to `Disabled` — `checkout()` returns `Ok(None)` without error,
/// and the server continues with ambient auth (e.g. `~/.claude/` credentials).
pub enum OptionalCredentialPool {
    /// Active pool connected to Postgres.
    Active(CredentialPool),
    /// Pool is disabled — no DB connection, `checkout()` always returns `Ok(None)`.
    Disabled,
}

impl OptionalCredentialPool {
    /// Construct from the `DATABASE_URL` environment variable.
    ///
    /// - If `DATABASE_URL` is absent: returns `Disabled` with a warning log.
    /// - If `DATABASE_URL` is present but connection fails: returns `Disabled` with a warning log.
    /// - If connection succeeds: initializes the schema and returns `Active`.
    pub async fn from_env() -> Self {
        let url = match std::env::var("DATABASE_URL") {
            Ok(u) => u,
            Err(_) => {
                warn!("credential pool disabled: DATABASE_URL not set");
                return Self::Disabled;
            }
        };

        match CredentialPool::new(&url).await {
            Ok(cp) => {
                if let Err(e) = cp.initialize_schema().await {
                    warn!(error = %e, "credential pool schema initialization failed — running disabled");
                    return Self::Disabled;
                }
                info!("credential pool ready");
                Self::Active(cp)
            }
            Err(e) => {
                warn!(error = %e, "credential pool unavailable — running disabled");
                Self::Disabled
            }
        }
    }

    /// Check out the least-recently-used credential, or return `Ok(None)` if disabled.
    pub async fn checkout(&self) -> Result<Option<ClaudeCredential>, sqlx::Error> {
        match self {
            Self::Active(pool) => pool.checkout().await,
            Self::Disabled => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Unit tests — no live database required
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn disabled_pool_returns_none() {
        let pool = OptionalCredentialPool::Disabled;
        let result = pool.checkout().await;
        assert!(result.is_ok(), "checkout on Disabled pool must not error");
        assert!(
            result.unwrap().is_none(),
            "checkout on Disabled pool must return None"
        );
    }

    // -------------------------------------------------------------------------
    // Integration tests — require TEST_DATABASE_URL env var
    // -------------------------------------------------------------------------

    fn test_db_url() -> Option<String> {
        std::env::var("TEST_DATABASE_URL").ok()
    }

    async fn fresh_pool(url: &str) -> CredentialPool {
        let cp = CredentialPool::new(url).await.expect("connect");
        cp.initialize_schema().await.expect("schema");
        // Start each test with a clean table
        sqlx::query("DELETE FROM claude_credentials")
            .execute(&cp.pool)
            .await
            .expect("cleanup");
        cp
    }

    #[tokio::test]
    async fn schema_creates_table() {
        let Some(url) = test_db_url() else {
            return;
        };
        let cp = CredentialPool::new(&url).await.expect("connect");
        // Call twice — must be idempotent
        cp.initialize_schema().await.expect("first call");
        cp.initialize_schema()
            .await
            .expect("second call (idempotent)");
    }

    #[tokio::test]
    async fn checkout_empty_returns_none() {
        let Some(url) = test_db_url() else {
            return;
        };
        let cp = fresh_pool(&url).await;
        let result = cp.checkout().await.expect("checkout");
        assert!(result.is_none(), "empty table should yield None");
    }

    #[tokio::test]
    async fn checkout_single_row() {
        let Some(url) = test_db_url() else {
            return;
        };
        let cp = fresh_pool(&url).await;

        sqlx::query(
            "INSERT INTO claude_credentials (account_name, token) VALUES ('test@example.com', 'tok_abc')",
        )
        .execute(&cp.pool)
        .await
        .expect("insert");

        let cred = cp
            .checkout()
            .await
            .expect("checkout")
            .expect("should have row");
        assert_eq!(cred.account_name, "test@example.com");
        assert_eq!(cred.token, "tok_abc");

        // Verify use_count incremented
        let row = sqlx::query("SELECT use_count FROM claude_credentials WHERE id = $1")
            .bind(cred.id)
            .fetch_one(&cp.pool)
            .await
            .expect("fetch");
        let use_count: i64 = row.try_get("use_count").expect("use_count");
        assert_eq!(use_count, 1, "use_count must increment to 1");

        // Verify last_used_at was updated (now > epoch default)
        let row2 = sqlx::query(
            "SELECT last_used_at > '1970-01-02' AS was_updated FROM claude_credentials WHERE id = $1",
        )
        .bind(cred.id)
        .fetch_one(&cp.pool)
        .await
        .expect("fetch");
        let was_updated: bool = row2.try_get("was_updated").expect("was_updated");
        assert!(was_updated, "last_used_at must be updated after checkout");
    }

    #[tokio::test]
    async fn checkout_round_robin() {
        let Some(url) = test_db_url() else {
            return;
        };
        let cp = fresh_pool(&url).await;

        sqlx::query(
            "INSERT INTO claude_credentials (account_name, token, last_used_at) VALUES \
             ('a@example.com', 'tok_a', '2020-01-01'), \
             ('b@example.com', 'tok_b', '2020-01-02')",
        )
        .execute(&cp.pool)
        .await
        .expect("insert");

        let first = cp.checkout().await.expect("first checkout").expect("row");
        let second = cp.checkout().await.expect("second checkout").expect("row");

        // First call should get the LRU row (a, older last_used_at)
        assert_eq!(first.account_name, "a@example.com");
        // Second call should get the other (b), since a's last_used_at was just updated
        assert_eq!(second.account_name, "b@example.com");
    }

    #[tokio::test]
    async fn checkout_skip_locked() {
        let Some(url) = test_db_url() else {
            return;
        };
        let cp = fresh_pool(&url).await;

        sqlx::query(
            "INSERT INTO claude_credentials (account_name, token) VALUES \
             ('a@example.com', 'tok_a'), \
             ('b@example.com', 'tok_b')",
        )
        .execute(&cp.pool)
        .await
        .expect("insert");

        // Spawn two concurrent checkouts — each should succeed without deadlock.
        let url2 = url.clone();
        let h1 = tokio::spawn(async move {
            let cp2 = CredentialPool::new(&url2).await.expect("connect");
            cp2.checkout().await.expect("checkout 1")
        });
        let h2 = tokio::spawn(async move {
            let cp3 = CredentialPool::new(&url).await.expect("connect");
            cp3.checkout().await.expect("checkout 2")
        });

        let (r1, r2) = tokio::join!(h1, h2);
        let c1 = r1.expect("task 1").expect("credential 1");
        let c2 = r2.expect("task 2").expect("credential 2");

        // Both checkouts succeeded with valid tokens — no deadlock
        assert!(!c1.token.is_empty());
        assert!(!c2.token.is_empty());
    }
}
