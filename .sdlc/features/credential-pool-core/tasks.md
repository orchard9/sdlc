# Tasks: credential-pool-core

## Task List

- [ ] T1: Expand `credential_pool.rs` — add `OptionalCredentialPool` enum with `from_env()` and `checkout()`, add tracing calls to `CredentialPool::new` and `initialize_schema`, ensure `ClaudeCredential` and `CredentialPool` public API matches spec
- [ ] T2: Declare `pub mod credential_pool` in `crates/sdlc-server/src/lib.rs` (or `main.rs`) if not already present
- [ ] T3: Add unit test `disabled_pool_returns_none` — verifies `OptionalCredentialPool::Disabled.checkout()` returns `Ok(None)` without any DB call
- [ ] T4: Add integration tests gated on `TEST_DATABASE_URL` — `schema_creates_table`, `checkout_empty_returns_none`, `checkout_single_row` (verifies `last_used_at` update and `use_count` increment), `checkout_round_robin` (two rows, alternating), `checkout_skip_locked` (concurrent checkouts get distinct rows)
- [ ] T5: Run `SDLC_NO_NPM=1 cargo test --all` and confirm all tests pass (unit tests only without live DB); run `cargo clippy --all -- -D warnings` with zero warnings
- [ ] T6: Commit all code changes
