# Tasks: Credential Pool Runs

## Task 1 — Add `mut` to `opts` parameter and inject token in `spawn_agent_run`

**File:** `crates/sdlc-server/src/routes/runs.rs`

Change the `opts: QueryOptions` parameter binding to `mut opts: QueryOptions`.

Immediately after the duplicate-run check (after the `// Lock dropped here` comment
and before `let run_id = generate_run_id()`), insert the credential injection block:

```rust
// Inject credential-pool token into QueryOptions before spawning.
{
    let token = checkout_from_pool(&app.credential_pool).await;
    if let Some(ref t) = token {
        opts.env
            .entry("CLAUDE_CODE_OAUTH_TOKEN".to_string())
            .or_insert_with(|| t.clone());
        for srv in &mut opts.mcp_servers {
            srv.env
                .entry("CLAUDE_CODE_OAUTH_TOKEN".to_string())
                .or_insert_with(|| t.clone());
        }
    }
}
```

## Task 2 — Add private `checkout_from_pool` helper in `runs.rs`

Add a private async helper near `checkout_claude_token`:

```rust
/// Check out a token from the pool. Returns `None` if pool is disabled,
/// empty, or a DB error occurs (errors are logged; never propagated).
async fn checkout_from_pool(
    pool: &Arc<std::sync::OnceLock<crate::credential_pool::OptionalCredentialPool>>,
) -> Option<String> {
    let pool = pool.get()?;
    match pool.checkout().await {
        Ok(Some(cred)) => Some(cred.token),
        Ok(None) => {
            warn!("credential pool empty — running with ambient auth");
            None
        }
        Err(e) => {
            error!(error = %e, "credential pool checkout failed — running with ambient auth");
            None
        }
    }
}
```

## Task 3 — Add a unit test for the injection logic

In the `#[cfg(test)]` block in `runs.rs`, add a test that:
1. Constructs a `QueryOptions` with a pre-set `env` entry and an MCP server without the token.
2. Simulates the injection logic (calling the same `entry().or_insert_with()` pattern).
3. Asserts that a pre-existing token is preserved and a missing token is filled.

This test does not require a live DB — it exercises the map-mutation logic only.
