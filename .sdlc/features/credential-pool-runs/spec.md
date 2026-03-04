# Spec: Credential Pool Runs

## Overview

Every call to `spawn_agent_run` should inject a checked-out OAuth token into the
Claude subprocess when a credential pool is available. Currently, some callers
explicitly call `checkout_claude_token` and pass the result into `sdlc_query_options`,
while others (notably `knowledge.rs` and `advisory.rs`) do not — leaving those agent
runs with no pool credential.

The fix centralises checkout inside `spawn_agent_run` so no caller can accidentally
miss the injection.

## Current Behaviour

`spawn_agent_run` accepts a fully-constructed `QueryOptions`. The credential token —
if any — is embedded in `opts.mcp_servers[sdlc].env["CLAUDE_CODE_OAUTH_TOKEN"]` by
the caller via `sdlc_query_options(root, turns, checkout_claude_token(&app).await)`.

Callers that construct opts via `sdlc_query_options` and explicitly call
`checkout_claude_token` work correctly. Callers that omit the checkout (or use a
different opts builder) silently fall back to ambient auth.

## Proposed Behaviour

`spawn_agent_run` will:

1. Call `app.credential_pool.checkout()` at the start of each run.
2. If checkout returns `Some(token)`, inject `CLAUDE_CODE_OAUTH_TOKEN` into the env
   of every MCP server listed in `opts.mcp_servers`. This covers the `sdlc` MCP
   server (where the env is already plumbed) and any future MCP servers.
3. Additionally, set the env var on the `QueryOptions` top-level `env` field if the
   claude-agent library exposes one — ensuring the token reaches the Claude subprocess
   itself, not just the MCP child process.
4. If checkout returns `None` (pool disabled, pool empty, or checkout error), proceed
   without injecting — retaining ambient credential behaviour unchanged.
5. Callers that already call `checkout_claude_token` before building opts will have
   the token set twice — the inner injection in `spawn_agent_run` is idempotent (it
   only inserts when the key is absent, or always overwrites — TBD in design).

## Env Var Name

`CLAUDE_CODE_OAUTH_TOKEN` — matches the existing convention used by
`sdlc_query_options`.

## Fallback Behaviour

- Pool disabled (`OptionalCredentialPool::Disabled`): `checkout()` returns `Ok(None)`;
  no injection; ambient auth used.
- Pool active but empty: `checkout()` returns `Ok(None)` with a warning; no injection.
- Pool active but DB error: `checkout()` returns `Err(e)`; log the error at `error!`
  level; no injection; proceed with ambient auth.

## Error Handling

Checkout errors must not abort the run. The contract is:
- `Ok(Some(cred))` → inject token, proceed.
- `Ok(None)` → skip injection, proceed.
- `Err(e)` → log at `error!` level, skip injection, proceed.

No `unwrap()` — use pattern matching on the `Result`.

## Scope Exclusion

The `QueryOptions` struct is owned by `claude-agent` and its fields may not include a
top-level `env` map. In that case, injection is limited to `opts.mcp_servers[*].env`.
The design step will confirm which fields are available.
