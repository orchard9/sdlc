# Design: Credential Pool Runs

## Summary

Modify `spawn_agent_run` in `crates/sdlc-server/src/routes/runs.rs` to perform a
credential-pool checkout at the start of every run and inject the resulting token as
`CLAUDE_CODE_OAUTH_TOKEN` into the Claude subprocess — covering both the top-level
`QueryOptions.env` (passed to the Claude binary itself) and each MCP server's env
(used by the sdlc MCP child process).

## Code Path

```
spawn_agent_run(key, prompt, mut opts, app, ...) {
    // 1. Checkout
    let token: Option<String> = checkout_from_pool(&app.credential_pool).await;

    // 2. Inject into Claude subprocess env
    if let Some(ref t) = token {
        opts.env
            .entry("CLAUDE_CODE_OAUTH_TOKEN".to_string())
            .or_insert_with(|| t.clone());
    }

    // 3. Inject into every MCP server env
    if let Some(ref t) = token {
        for srv in &mut opts.mcp_servers {
            srv.env
                .entry("CLAUDE_CODE_OAUTH_TOKEN".to_string())
                .or_insert_with(|| t.clone());
        }
    }

    // 4. Spawn the task (unchanged below this point)
    ...
}
```

Using `entry(...).or_insert_with(...)` means:
- If the caller already embedded the token (via `sdlc_query_options` + `checkout_claude_token`),
  the existing value is preserved — no double checkout, no overwrite.
- If the caller did not embed a token (e.g. `knowledge.rs`, `advisory.rs`), the pool
  checkout fills the gap.

## Checkout Helper

A private helper replaces the existing public `checkout_claude_token`:

```rust
async fn checkout_from_pool(
    pool: &Arc<OnceLock<OptionalCredentialPool>>,
) -> Option<String> {
    let pool = pool.get()?;
    match pool.checkout().await {
        Ok(Some(cred)) => Some(cred.token),
        Ok(None) => {
            warn!("credential pool has no active Claude credentials — running with ambient auth");
            None
        }
        Err(e) => {
            error!(error = %e, "credential pool checkout failed — running with ambient auth");
            None
        }
    }
}
```

The existing `checkout_claude_token(app: &AppState)` helper is kept as-is (it is
`pub(crate)` and used by callers building opts). It becomes redundant over time but
removing it in this feature would break all existing call sites and is out of scope.

## Signature Change

`spawn_agent_run` gains `mut` binding for `opts` (currently immutable):

```rust
pub(crate) async fn spawn_agent_run(
    key: String,
    prompt: String,
    mut opts: QueryOptions,   // <-- mut added
    app: &AppState,
    ...
```

No callers need to change — `mut` is a local binding, not part of the type.

## Fields Confirmed

- `QueryOptions.env: HashMap<String, String>` — subprocess env, confirmed in
  `crates/claude-agent/src/types.rs:460`.
- `McpServerConfig.env: HashMap<String, String>` — MCP child env, confirmed in
  `crates/claude-agent/src/types.rs:533`.
- `process.rs` iterates both maps and passes them via `cmd.env(k, v)` (line 39) and
  the MCP server spawn (line 345).

## Edge Cases

| Scenario | Behaviour |
|---|---|
| Pool disabled | `pool.get()` returns `None` via `OnceLock`; no injection |
| Pool initializing (not yet set) | Same — `OnceLock::get()` is `None`; no injection |
| Pool active, no credentials | `checkout()` returns `Ok(None)`; warn logged; no injection |
| Pool active, DB error | `checkout()` returns `Err(e)`; error logged; no injection |
| Caller already set token | `entry().or_insert_with()` preserves existing value |
| Two concurrent runs | Each gets its own checkout; LRU+SKIP LOCKED in the DB ensures no collision |

## What Does NOT Change

- `sdlc_query_options`, `sdlc_ponder_query_options`, `sdlc_guideline_query_options` —
  unchanged; callers that already inject the token continue to work.
- `checkout_claude_token` — kept as-is to avoid breaking call sites.
- The credential pool itself (`credential_pool.rs`) — no changes.
- `AppState`, `state.rs` — no changes.
- All callers of `spawn_agent_run` — no signature changes required.

## Files Changed

- `crates/sdlc-server/src/routes/runs.rs` — one `mut` keyword and ~15 lines of
  injection logic inside `spawn_agent_run`.
