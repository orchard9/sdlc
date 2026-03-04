# Spec: credential-pool-core

## Summary

Implement a PostgreSQL-backed `CredentialPool` struct in `crates/sdlc-server/src/credential_pool.rs`
that stores Claude OAuth tokens in a `claude_credentials` table and checks them out via
round-robin using `SELECT FOR UPDATE SKIP LOCKED`. The pool degrades gracefully when
`DATABASE_URL` is absent or the database is unreachable.

## Problem

Cluster pods running `sdlc-server` need Claude OAuth tokens for agent runs, but tokens
cannot live on ephemeral pod filesystems. Without a shared credential store, every pod
restart or new pod requires manual credential setup. The credential pool solves this by
centralizing tokens in Postgres, shared by all pods.

## Scope

This feature covers the core data layer only:
- The `CredentialPool` struct and `ClaudeCredential` value type
- Schema initialization (`CREATE TABLE IF NOT EXISTS`)
- Round-robin checkout transaction (`SELECT FOR UPDATE SKIP LOCKED`)
- Graceful no-op wrapper (`OptionalCredentialPool`) for when `DATABASE_URL` is absent
- Unit/integration tests using `cargo test` (no live DB required for unit tests)

Out of scope for this feature:
- Injection of checked-out tokens into agent run subprocesses (handled by `credential-pool-runs`)
- Helm/ExternalSecret wiring (handled by `credential-pool-helm`)
- REST API endpoints for managing credentials

## Schema

```sql
CREATE TABLE IF NOT EXISTS claude_credentials (
    id           BIGSERIAL PRIMARY KEY,
    account_name TEXT        NOT NULL,
    token        TEXT        NOT NULL,
    is_active    BOOLEAN     NOT NULL DEFAULT true,
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01',
    use_count    BIGINT      NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS claude_credentials_lru_idx
    ON claude_credentials (last_used_at ASC)
    WHERE is_active;
```

## Checkout Algorithm

```
BEGIN TRANSACTION
SELECT id, account_name, token
  FROM claude_credentials
  WHERE is_active
  ORDER BY last_used_at ASC
  LIMIT 1
  FOR UPDATE SKIP LOCKED
→ if no row: ROLLBACK, return Ok(None)
UPDATE claude_credentials SET last_used_at = NOW(), use_count = use_count + 1 WHERE id = $1
COMMIT
return Ok(Some(ClaudeCredential { id, account_name, token }))
```

The `SKIP LOCKED` clause ensures concurrent callers each acquire a different row without
blocking one another. There is no retry — callers that find all rows locked receive `None`
and fall back to ambient auth.

## Public API

```rust
/// Value returned from a successful checkout.
pub struct ClaudeCredential {
    pub id: i64,
    pub account_name: String,
    pub token: String,
}

/// PostgreSQL-backed credential pool.
pub struct CredentialPool { /* private */ }

impl CredentialPool {
    /// Connect and return a pool. Returns Err if the database is unreachable.
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error>;

    /// CREATE TABLE IF NOT EXISTS. Call once at startup.
    pub async fn initialize_schema(&self) -> Result<(), sqlx::Error>;

    /// Round-robin checkout. Returns None if no active credentials exist.
    pub async fn checkout(&self) -> Result<Option<ClaudeCredential>, sqlx::Error>;
}

/// Wrapper that holds an optional pool. When DATABASE_URL is absent the
/// pool is None and checkout() always returns Ok(None) without error.
pub enum OptionalCredentialPool {
    Active(CredentialPool),
    Disabled,
}

impl OptionalCredentialPool {
    /// Construct from environment. Logs a warning if DATABASE_URL is absent.
    pub async fn from_env() -> Self;

    /// Forward checkout or return Ok(None) if disabled.
    pub async fn checkout(&self) -> Result<Option<ClaudeCredential>, sqlx::Error>;
}
```

## Graceful Degradation

- If `DATABASE_URL` env var is absent: `OptionalCredentialPool::from_env()` returns
  `OptionalCredentialPool::Disabled`. No connection is attempted. Log: `"credential pool
  disabled: DATABASE_URL not set"`.
- If `DATABASE_URL` is present but connection fails: `from_env()` logs a warning and returns
  `OptionalCredentialPool::Disabled`. The server continues booting normally.
- If `checkout()` is called on a `Disabled` pool: returns `Ok(None)` immediately.
- If `checkout()` finds no active credentials: returns `Ok(None)`.

## Non-Goals

- Token expiry, refresh, or rotation
- Per-project token isolation
- Credential insertion API (out-of-band Postgres INSERT is the intended path)
- Any REST endpoint in this feature

## Test Plan (high-level)

- Unit test: `OptionalCredentialPool::Disabled` returns `Ok(None)` from checkout
- Integration test (requires live Postgres, opt-in via `TEST_DATABASE_URL`): schema init,
  single checkout, round-robin with two rows, concurrent checkout
- All tests runnable with `SDLC_NO_NPM=1 cargo test --all` (unit tests only without live DB)
