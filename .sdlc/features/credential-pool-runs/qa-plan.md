# QA Plan: Credential Pool Runs

## Automated Tests

### Unit tests (no DB)

1. **`checkout_from_pool_disabled_returns_none`**
   - Construct `OptionalCredentialPool::Disabled` wrapped in `Arc<OnceLock>`.
   - Assert `checkout_from_pool` returns `None` without error.

2. **`inject_token_into_opts_env`**
   - Build `QueryOptions` with empty `env`.
   - Simulate injection of `"tok_abc"`.
   - Assert `opts.env["CLAUDE_CODE_OAUTH_TOKEN"] == "tok_abc"`.

3. **`inject_token_into_mcp_server_env`**
   - Build `QueryOptions` with one `McpServerConfig` having no env entry.
   - Simulate injection.
   - Assert `opts.mcp_servers[0].env["CLAUDE_CODE_OAUTH_TOKEN"] == "tok_abc"`.

4. **`existing_token_not_overwritten`**
   - Build `QueryOptions` with `CLAUDE_CODE_OAUTH_TOKEN = "caller_tok"` in both `env`
     and MCP server env.
   - Simulate injection of `"pool_tok"`.
   - Assert both values remain `"caller_tok"` (caller wins via `or_insert_with`).

5. **`no_injection_when_pool_returns_none`**
   - Pool returns `None` (empty).
   - Assert `opts.env` does not contain `CLAUDE_CODE_OAUTH_TOKEN`.

### Existing tests must still pass

- Run `SDLC_NO_NPM=1 cargo test --all` — all existing tests green.
- Run `cargo clippy --all -- -D warnings` — no new warnings.

## Integration smoke test (manual, optional)

If `TEST_DATABASE_URL` is available:
1. Seed one credential via `credential_pool.add("test@example.com", "tok_test")`.
2. Start the server with `DATABASE_URL` set.
3. Trigger a ponder run via `POST /api/ponder/:slug/chat`.
4. Observe logs: `"checked out Claude credential from pool"` should appear.
5. Confirm `use_count` incremented in the DB.

## Regression check

- Existing behavior when pool is disabled: no change — ambient auth used, no new
  log noise beyond the existing `credential pool disabled: DATABASE_URL not set` warning.
- `knowledge.rs` and `advisory.rs` callers: now automatically receive pool tokens
  without any call-site changes.
