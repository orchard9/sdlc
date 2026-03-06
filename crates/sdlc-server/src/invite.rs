//! OTP invite system for hub mode.
//!
//! Admins create invites (email + 6-digit OTP). External users verify the OTP
//! to receive a signed session cookie. Supports in-memory (default) and
//! PostgreSQL (when `DATABASE_URL` is set) backends.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InviteRecord {
    pub id: String,
    pub email: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: InviteStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InviteStatus {
    Pending,
    Used,
    Revoked,
}

// ---------------------------------------------------------------------------
// InviteStore enum (Memory | Postgres)
// ---------------------------------------------------------------------------

pub enum InviteStore {
    Memory(MemoryInviteStore),
    Postgres(PgInviteStore),
}

impl InviteStore {
    /// Build from environment: Postgres if `DATABASE_URL` is set, else Memory.
    pub async fn from_env() -> Self {
        if let Ok(url) = std::env::var("DATABASE_URL") {
            match sqlx::PgPool::connect(&url).await {
                Ok(pool) => {
                    tracing::info!("invite store: postgres");
                    return Self::Postgres(PgInviteStore { pool });
                }
                Err(e) => {
                    tracing::warn!(error = %e, "invite store: postgres connect failed, falling back to memory");
                }
            }
        }
        tracing::info!("invite store: memory");
        Self::Memory(MemoryInviteStore::new())
    }

    pub async fn create(
        &self,
        email: &str,
        created_by: &str,
    ) -> Result<(InviteRecord, String), String> {
        match self {
            Self::Memory(s) => s.create(email, created_by).await,
            Self::Postgres(s) => s.create(email, created_by).await,
        }
    }

    pub async fn verify(&self, email: &str, otp: &str) -> Result<InviteRecord, InviteError> {
        match self {
            Self::Memory(s) => s.verify(email, otp).await,
            Self::Postgres(s) => s.verify(email, otp).await,
        }
    }

    pub async fn list(&self) -> Result<Vec<InviteRecord>, String> {
        match self {
            Self::Memory(s) => s.list().await,
            Self::Postgres(s) => s.list().await,
        }
    }

    pub async fn revoke(&self, id: &str) -> Result<(), String> {
        match self {
            Self::Memory(s) => s.revoke(id).await,
            Self::Postgres(s) => s.revoke(id).await,
        }
    }
}

#[derive(Debug)]
pub enum InviteError {
    RateLimited,
    InvalidOrExpired,
}

// ---------------------------------------------------------------------------
// OTP helpers
// ---------------------------------------------------------------------------

fn generate_otp() -> String {
    use rand::Rng;
    let code: u32 = rand::thread_rng().gen_range(0..1_000_000);
    format!("{code:06}")
}

fn hash_otp(otp: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(otp.as_bytes());
    hasher.finalize().to_vec()
}

// ---------------------------------------------------------------------------
// MemoryInviteStore
// ---------------------------------------------------------------------------

struct MemoryInviteEntry {
    record: InviteRecord,
    otp_hash: Vec<u8>,
}

pub struct MemoryInviteStore {
    invites: RwLock<HashMap<String, MemoryInviteEntry>>,
    attempts: RwLock<HashMap<String, Vec<DateTime<Utc>>>>,
}

impl MemoryInviteStore {
    fn new() -> Self {
        Self {
            invites: RwLock::new(HashMap::new()),
            attempts: RwLock::new(HashMap::new()),
        }
    }

    async fn create(
        &self,
        email: &str,
        created_by: &str,
    ) -> Result<(InviteRecord, String), String> {
        let otp = generate_otp();
        let now = Utc::now();
        let record = InviteRecord {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.to_lowercase(),
            created_by: created_by.to_string(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(48),
            status: InviteStatus::Pending,
            used_at: None,
        };
        let entry = MemoryInviteEntry {
            record: record.clone(),
            otp_hash: hash_otp(&otp),
        };
        self.invites.write().await.insert(record.id.clone(), entry);
        Ok((record, otp))
    }

    async fn verify(&self, email: &str, otp: &str) -> Result<InviteRecord, InviteError> {
        let email_lower = email.to_lowercase();

        // Rate limit: max 5 attempts per email per hour
        {
            let mut attempts = self.attempts.write().await;
            let entry = attempts.entry(email_lower.clone()).or_default();
            let cutoff = Utc::now() - chrono::Duration::hours(1);
            entry.retain(|t| *t > cutoff);
            if entry.len() >= 5 {
                return Err(InviteError::RateLimited);
            }
            entry.push(Utc::now());
        }

        let expected_hash = hash_otp(otp);
        let now = Utc::now();

        let mut invites = self.invites.write().await;
        // Find the matching pending invite for this email
        let matching_id = invites
            .iter()
            .find(|(_, e)| {
                e.record.email == email_lower
                    && e.record.status == InviteStatus::Pending
                    && e.record.expires_at > now
                    && e.otp_hash == expected_hash
            })
            .map(|(id, _)| id.clone());

        let Some(id) = matching_id else {
            return Err(InviteError::InvalidOrExpired);
        };

        let entry = invites.get_mut(&id).unwrap();
        entry.record.status = InviteStatus::Used;
        entry.record.used_at = Some(now);
        Ok(entry.record.clone())
    }

    async fn list(&self) -> Result<Vec<InviteRecord>, String> {
        let invites = self.invites.read().await;
        let mut records: Vec<InviteRecord> = invites.values().map(|e| e.record.clone()).collect();
        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(records)
    }

    async fn revoke(&self, id: &str) -> Result<(), String> {
        let mut invites = self.invites.write().await;
        let entry = invites
            .get_mut(id)
            .ok_or_else(|| format!("invite {id} not found"))?;
        entry.record.status = InviteStatus::Revoked;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PgInviteStore
// ---------------------------------------------------------------------------

pub struct PgInviteStore {
    pool: sqlx::PgPool,
}

type InviteRow = (
    String,
    String,
    String,
    DateTime<Utc>,
    DateTime<Utc>,
    String,
    Option<DateTime<Utc>>,
);

fn row_to_record(row: InviteRow) -> InviteRecord {
    InviteRecord {
        id: row.0,
        email: row.1,
        created_by: row.2,
        created_at: row.3,
        expires_at: row.4,
        status: match row.5.as_str() {
            "used" => InviteStatus::Used,
            "revoked" => InviteStatus::Revoked,
            _ => InviteStatus::Pending,
        },
        used_at: row.6,
    }
}

impl PgInviteStore {
    async fn create(
        &self,
        email: &str,
        created_by: &str,
    ) -> Result<(InviteRecord, String), String> {
        let otp = generate_otp();
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let email_lower = email.to_lowercase();
        let expires_at = now + chrono::Duration::hours(48);
        let otp_hash_bytes = hash_otp(&otp);

        sqlx::query(
            "INSERT INTO otp_invites (id, email, otp_hash, created_by, created_at, expires_at, status)
             VALUES ($1, $2, $3, $4, $5, $6, 'pending')",
        )
        .bind(&id)
        .bind(&email_lower)
        .bind(&otp_hash_bytes)
        .bind(created_by)
        .bind(now)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("insert invite: {e}"))?;

        let record = InviteRecord {
            id,
            email: email_lower,
            created_by: created_by.to_string(),
            created_at: now,
            expires_at,
            status: InviteStatus::Pending,
            used_at: None,
        };
        Ok((record, otp))
    }

    async fn verify(&self, email: &str, otp: &str) -> Result<InviteRecord, InviteError> {
        let email_lower = email.to_lowercase();

        // Record attempt
        sqlx::query("INSERT INTO otp_attempts (email, attempted_at) VALUES ($1, NOW())")
            .bind(&email_lower)
            .execute(&self.pool)
            .await
            .ok();

        // Rate limit check: count attempts in last hour
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM otp_attempts WHERE email = $1 AND attempted_at > NOW() - INTERVAL '1 hour'",
        )
        .bind(&email_lower)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| InviteError::InvalidOrExpired)?;

        if row.0 > 5 {
            return Err(InviteError::RateLimited);
        }

        let expected_hash = hash_otp(otp);

        // Find matching pending invite
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM otp_invites WHERE email = $1 AND status = 'pending' AND expires_at > NOW() AND otp_hash = $2 LIMIT 1",
        )
        .bind(&email_lower)
        .bind(&expected_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| InviteError::InvalidOrExpired)?;

        let Some((id,)) = result else {
            return Err(InviteError::InvalidOrExpired);
        };

        // Mark as used
        sqlx::query("UPDATE otp_invites SET status = 'used', used_at = NOW() WHERE id = $1")
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|_| InviteError::InvalidOrExpired)?;

        // Re-fetch the record
        let row: InviteRow = sqlx::query_as(
            "SELECT id, email, created_by, created_at, expires_at, status, used_at FROM otp_invites WHERE id = $1",
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| InviteError::InvalidOrExpired)?;

        Ok(row_to_record(row))
    }

    async fn list(&self) -> Result<Vec<InviteRecord>, String> {
        let rows: Vec<InviteRow> = sqlx::query_as(
            "SELECT id, email, created_by, created_at, expires_at, status, used_at FROM otp_invites ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("list invites: {e}"))?;

        Ok(rows.into_iter().map(row_to_record).collect())
    }

    async fn revoke(&self, id: &str) -> Result<(), String> {
        let result = sqlx::query("UPDATE otp_invites SET status = 'revoked' WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("revoke invite: {e}"))?;

        if result.rows_affected() == 0 {
            return Err(format!("invite {id} not found"));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_otp_format() {
        for _ in 0..100 {
            let otp = generate_otp();
            assert_eq!(otp.len(), 6, "OTP should be 6 digits: {otp}");
            assert!(
                otp.chars().all(|c| c.is_ascii_digit()),
                "OTP should be all digits: {otp}"
            );
        }
    }

    #[tokio::test]
    async fn test_memory_create_and_verify() {
        let store = MemoryInviteStore::new();
        let (record, otp) = store
            .create("Test@Example.com", "admin@test.com")
            .await
            .unwrap();
        assert_eq!(record.email, "test@example.com");
        assert_eq!(record.status, InviteStatus::Pending);

        let verified = store.verify("test@example.com", &otp).await.unwrap();
        assert_eq!(verified.status, InviteStatus::Used);
        assert!(verified.used_at.is_some());
    }

    #[tokio::test]
    async fn test_memory_used_cannot_reverify() {
        let store = MemoryInviteStore::new();
        let (_, otp) = store
            .create("user@test.com", "admin@test.com")
            .await
            .unwrap();
        store.verify("user@test.com", &otp).await.unwrap();

        // Second verify should fail
        let result = store.verify("user@test.com", &otp).await;
        assert!(matches!(result, Err(InviteError::InvalidOrExpired)));
    }

    #[tokio::test]
    async fn test_memory_expired_rejected() {
        let store = MemoryInviteStore::new();
        let (record, otp) = store
            .create("user@test.com", "admin@test.com")
            .await
            .unwrap();

        // Manually expire the invite
        {
            let mut invites = store.invites.write().await;
            let entry = invites.get_mut(&record.id).unwrap();
            entry.record.expires_at = Utc::now() - chrono::Duration::hours(1);
        }

        let result = store.verify("user@test.com", &otp).await;
        assert!(matches!(result, Err(InviteError::InvalidOrExpired)));
    }

    #[tokio::test]
    async fn test_memory_rate_limit() {
        let store = MemoryInviteStore::new();
        let (_, _otp) = store
            .create("user@test.com", "admin@test.com")
            .await
            .unwrap();

        // Make 5 failed attempts
        for _ in 0..5 {
            let _ = store.verify("user@test.com", "000000").await;
        }

        // 6th attempt should be rate limited
        let result = store.verify("user@test.com", "000000").await;
        assert!(matches!(result, Err(InviteError::RateLimited)));
    }

    #[tokio::test]
    async fn test_memory_revoke() {
        let store = MemoryInviteStore::new();
        let (record, otp) = store
            .create("user@test.com", "admin@test.com")
            .await
            .unwrap();
        store.revoke(&record.id).await.unwrap();

        let result = store.verify("user@test.com", &otp).await;
        assert!(matches!(result, Err(InviteError::InvalidOrExpired)));
    }

    #[tokio::test]
    async fn test_memory_wrong_otp() {
        let store = MemoryInviteStore::new();
        let (_, _otp) = store
            .create("user@test.com", "admin@test.com")
            .await
            .unwrap();

        let result = store.verify("user@test.com", "999999").await;
        assert!(matches!(result, Err(InviteError::InvalidOrExpired)));
    }
}
