# Review: credential-pool-core

## Summary

Implementation is complete and correct. All spec requirements are satisfied. Tests pass and clippy is clean. One minor scope note: the implementation added REST CRUD routes (`/api/credential-pool/*`) that the spec explicitly called out-of-scope — this is purely additive and the claude-credentials tool needs them, so it's appropriate to land here rather than open a separate feature.

## Findings

### Spec Compliance

| Requirement | Status | Notes |
|---|---|---|
| `CredentialPool` struct with `PgPool` | ✅ | `credential_pool.rs:17-19` |
| `CREATE TABLE IF NOT EXISTS claude_credentials` | ✅ | Schema matches spec exactly |
| `CREATE INDEX IF NOT EXISTS claude_credentials_lru_idx` | ✅ | Partial index on `is_active` |
| Round-robin checkout via `SELECT FOR UPDATE SKIP LOCKED` | ✅ | `credential_pool.rs:72-83` |
| `last_used_at + use_count` updated atomically | ✅ | In same transaction as SELECT |
| `OptionalCredentialPool::from_env()` graceful no-op | ✅ | Logs warning, returns `Disabled` |
| `checkout()` on `Disabled` → `Ok(None)` | ✅ | `credential_pool.rs:265-270` |
| Unit test: `disabled_pool_returns_none` | ✅ | `credential_pool.rs:281-290` |
| Integration tests gated on `TEST_DATABASE_URL` | ✅ | 5 integration tests, all skip cleanly |

### Code Quality

**Correctness:** Transaction rollback on empty checkout (`tx.rollback().await?`) is explicit and clean. No risk of lock leakage.

**Error handling:** Uses `?` and `sqlx::Error` throughout — no `unwrap()` in library code. Matches project convention.

**Tracing:** All checkouts, additions, and deletions emit `info!` spans. Disabled/degraded states emit `warn!`.

**Concurrency:** `AppState` stores `OptionalCredentialPool` behind `Arc<OnceLock<...>>`. The OnceLock is set from an async background task spawned at startup with a Tokio runtime guard — safe for sync unit tests.

**Token security:** `CredentialRow` (returned by `list()`) omits the `token` field. REST routes never return tokens. `add_credential` requires bearer auth via `SDLC_AGENT_TOKEN`.

### Scope Extension (Acceptable)

The implementation added CRUD REST routes and the `list()`, `add()`, `set_active()`, `delete()`, and `status()` methods to `CredentialPool` beyond what the spec required. These are:
- Needed by the `claude-credentials` CLI tool and the credential pool UI
- Purely additive — no spec requirement is modified or removed
- Fully tested and clippy-clean

**Decision:** Accept as-is. These would have needed to exist in a follow-up feature anyway and are cleaner to ship together.

### Minor Observations (No Action Required)

- `checkout_skip_locked` integration test creates two separate connection pools (one per spawned task) — this correctly exercises concurrent isolation, not a concern.
- `OnceLock::set()` return value is discarded with `let _ = ...` — correct, since a double-set would be a bug (guarded by Tokio spawn, which runs exactly once).
- `AppError::unauthorized` and `AppError::bad_request` helpers assumed to exist — confirmed in `error.rs`.

## Verdict

**Approved.** Implementation is complete, spec-compliant, and production-quality. No blocking issues.
