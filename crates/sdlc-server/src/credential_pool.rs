use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

/// A single Claude credential from the pool.
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
        Ok(Self { pool })
    }

    /// Create the `claude_credentials` table if it does not exist.
    pub async fn initialize_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS claude_credentials (
                id           BIGSERIAL PRIMARY KEY,
                account_name TEXT      NOT NULL,
                token        TEXT      NOT NULL,
                is_active    BOOLEAN   NOT NULL DEFAULT true,
                last_used_at TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01',
                use_count    BIGINT    NOT NULL DEFAULT 0
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

        Ok(())
    }

    /// Check out the least-recently-used active credential using SELECT FOR UPDATE SKIP LOCKED.
    /// Updates `last_used_at` and `use_count` atomically in the same transaction.
    /// Returns `None` if no active credentials exist.
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

        Ok(Some(ClaudeCredential {
            id,
            account_name,
            token,
        }))
    }
}
