# Design: credential-pool-core

## Module Location

`crates/sdlc-server/src/credential_pool.rs` — already scaffolded, to be expanded.
Exposed via `pub mod credential_pool;` in `crates/sdlc-server/src/lib.rs` (or `main.rs`).

## Data Types

```
┌──────────────────────────────────────────────────────┐
│ ClaudeCredential                                     │
│   id:           i64                                  │
│   account_name: String                               │
│   token:        String                               │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│ CredentialPool                                       │
│   pool: sqlx::PgPool  (max_connections = 5)          │
│                                                      │
│  + new(database_url) -> Result<Self, sqlx::Error>    │
│  + initialize_schema()  -> Result<(), sqlx::Error>   │
│  + checkout()           -> Result<Option<CC>, Error> │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│ OptionalCredentialPool (enum)                        │
│   Active(CredentialPool)                             │
│   Disabled                                           │
│                                                      │
│  + from_env() -> Self   (async, reads DATABASE_URL)  │
│  + checkout() -> Result<Option<CC>, sqlx::Error>     │
└──────────────────────────────────────────────────────┘
```

## Sequence: Startup

```
main()
  │
  └─► OptionalCredentialPool::from_env()
          │
          ├─ DATABASE_URL absent?
          │     └─► warn!("credential pool disabled: DATABASE_URL not set")
          │         return Disabled
          │
          └─ DATABASE_URL present
                │
                ├─ CredentialPool::new(url) → Ok(pool)?
                │     └─► pool.initialize_schema()
                │           └─► info!("credential pool ready")
                │               return Active(pool)
                │
                └─ Err(e)
                      └─► warn!("credential pool unavailable: {e}")
                          return Disabled
```

## Sequence: Checkout (Active pool)

```
caller ──► pool.checkout()
              │
              └─► BEGIN TRANSACTION
                     │
                     └─► SELECT id, account_name, token
                             FROM claude_credentials
                             WHERE is_active
                             ORDER BY last_used_at ASC
                             LIMIT 1
                             FOR UPDATE SKIP LOCKED
                             │
                             ├─ No row (empty or all locked)?
                             │    └─► ROLLBACK; return Ok(None)
                             │
                             └─ Row found
                                  └─► UPDATE SET last_used_at=NOW(), use_count+=1
                                      COMMIT
                                      return Ok(Some(ClaudeCredential { id, account_name, token }))
```

## Sequence: Checkout (Disabled pool)

```
caller ──► pool.checkout()
              └─► return Ok(None)   // immediate, no DB call
```

## Database Schema

```sql
CREATE TABLE IF NOT EXISTS claude_credentials (
    id           BIGSERIAL   PRIMARY KEY,
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

The `is_active` partial index lets the query planner use the index without scanning
inactive rows. `last_used_at` defaulting to epoch ensures new rows are always
checked out first (LRU order).

## Integration with AppState

`OptionalCredentialPool` is constructed once during server startup and held in `AppState`:

```rust
pub struct AppState {
    // ... existing fields ...
    pub credential_pool: Arc<OptionalCredentialPool>,
}
```

`Arc<>` allows clone-free sharing across request handlers and agent run tasks.

## Error Handling

- `CredentialPool::new` propagates `sqlx::Error` — caller decides to degrade.
- `CredentialPool::checkout` propagates `sqlx::Error` on transient DB errors.
- `OptionalCredentialPool::Disabled::checkout` always returns `Ok(None)` — never errors.
- Callers of `OptionalCredentialPool::checkout` must handle the `Err` variant (transient
  DB failure during an otherwise-active pool). Recommended: log warn, continue without token.

## Dependencies

- `sqlx` 0.8 with `runtime-tokio-rustls` + `postgres` + `chrono` features — already present
  in `crates/sdlc-server/Cargo.toml`.
- No new dependencies required.

## File Changes

| File | Change |
|---|---|
| `crates/sdlc-server/src/credential_pool.rs` | Expand scaffold: add `OptionalCredentialPool`, tracing calls, tests |
| `crates/sdlc-server/src/lib.rs` or `main.rs` | `pub mod credential_pool;` if not already declared |

## Testing Strategy

Tests live in `credential_pool.rs` under `#[cfg(test)]`.

**Unit tests (no live DB):**
- `disabled_pool_returns_none` — `OptionalCredentialPool::Disabled` checkout returns `Ok(None)`

**Integration tests (live DB, gated on `TEST_DATABASE_URL`):**
- `schema_creates_table` — `initialize_schema()` is idempotent
- `checkout_empty_returns_none` — empty table returns `Ok(None)`
- `checkout_single_row` — row returned, `last_used_at` updated, `use_count` incremented
- `checkout_round_robin` — two rows, two calls, each returns the other
- `checkout_skip_locked` — two concurrent checkouts each get distinct rows
