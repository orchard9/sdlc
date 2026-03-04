# QA Plan: credential-pool-core

## Test Strategy

Tests live in `crates/sdlc-server/src/credential_pool.rs` under `#[cfg(test)]`.

### Unit Tests (no live DB — always run)

| Test | What it verifies |
|---|---|
| `disabled_pool_returns_none` | `OptionalCredentialPool::Disabled.checkout()` returns `Ok(None)` without panic or DB call |

### Integration Tests (gated on `TEST_DATABASE_URL` env var)

These tests require a live Postgres instance. They are skipped automatically when
`TEST_DATABASE_URL` is not set, so `SDLC_NO_NPM=1 cargo test --all` always passes.

| Test | What it verifies |
|---|---|
| `schema_creates_table` | `initialize_schema()` is idempotent — call twice, no error |
| `checkout_empty_returns_none` | Empty `claude_credentials` table yields `Ok(None)` |
| `checkout_single_row` | One active row is returned; `last_used_at` > epoch; `use_count` = 1 |
| `checkout_round_robin` | Two rows; call checkout twice; each call returns the row not returned by the previous call |
| `checkout_skip_locked` | Two concurrent `tokio::spawn` tasks each get a distinct credential (SKIP LOCKED working) |

## Test Invocation

```bash
# Unit tests only (no DB required):
SDLC_NO_NPM=1 cargo test --all

# Full integration tests (requires Postgres):
TEST_DATABASE_URL="postgres://..." SDLC_NO_NPM=1 cargo test --all
```

## Lint & Clippy

```bash
cargo clippy --all -- -D warnings
```

Zero warnings is the pass bar.

## Quality Gates

- All unit tests pass: required
- All integration tests pass (when `TEST_DATABASE_URL` is set): required before merge
- Clippy clean: required
- No `unwrap()` in library/server code: required (enforced by code review)
