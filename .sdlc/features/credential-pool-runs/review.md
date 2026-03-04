# Code Review: Credential Pool Runs

## Change Summary

Modified `crates/sdlc-server/src/routes/runs.rs`:

1. Added `mut` to the `opts: QueryOptions` binding in `spawn_agent_run`.
2. Added a credential-injection block immediately after the duplicate-run check:
   - Calls new private `checkout_from_pool` helper.
   - If a token is returned, injects `CLAUDE_CODE_OAUTH_TOKEN` into `opts.env`
     (subprocess) and each `opts.mcp_servers[*].env` (MCP child processes).
   - Uses `entry().or_insert_with()` to preserve tokens set by callers.
3. Added private `checkout_from_pool` helper (async, returns `Option<String>`,
   never propagates errors — logs at `warn!`/`error!` level).
4. Added 6 unit tests covering all injection branches.

## Findings

### Finding 1: All callers gain automatic token injection without call-site changes — ACCEPT

`knowledge.rs` (4 call sites) and `advisory.rs` (1 call site) previously called
`spawn_agent_run` without injecting a pool credential. They now automatically receive
pool tokens. No call-site changes required. This is the primary goal.

### Finding 2: `entry().or_insert_with()` preserves caller-provided tokens — ACCEPT

The ~20 call sites in `runs.rs` that already call `checkout_claude_token` before
building opts will pass an already-populated `CLAUDE_CODE_OAUTH_TOKEN` into
`sdlc_query_options`. The injection block uses `entry().or_insert_with()`, which is
a no-op when the key already exists, so those callers continue to work correctly.
There is a minor redundancy (double checkout avoided because the key is already set),
but correctness is maintained.

### Finding 3: Double checkout for callers that already call checkout_claude_token — TRACK

Callers that already call `checkout_claude_token` will trigger a second checkout
inside `spawn_agent_run`. Both checkouts update `last_used_at` and `use_count`.
This is functionally harmless (the second checkout increments `use_count` again and
may select a different credential from the pool, though the token embedded by the
first checkout is the one actually used via `or_insert_with`).

The correct long-term fix is to remove the explicit `checkout_claude_token` calls at
each call site and rely solely on the injection in `spawn_agent_run`. This is a
separate refactor — not required for correctness, and out of scope for this feature.

Action: `sdlc task add credential-pool-runs "[follow-up] Remove redundant checkout_claude_token calls from spawn_agent_run callers"`

### Finding 4: No error propagation from checkout — ACCEPT

`checkout_from_pool` swallows DB errors and returns `None`, allowing the run to
proceed with ambient auth. This matches the spec requirement and is consistent with
the existing `checkout_claude_token` implementation. DB errors are logged at
`error!` level so they are visible in production traces.

### Finding 5: Tests cover all branches — ACCEPT

Six tests added:
- Token injected into `opts.env` ✓
- Token injected into MCP server env ✓
- Existing caller token not overwritten ✓
- No injection when pool returns None ✓
- `checkout_from_pool` returns None when pool is Disabled ✓
- `checkout_from_pool` returns None when OnceLock uninitialised ✓

All 49 server tests pass. Clippy clean.

## Verdict: APPROVE with one tracked follow-up task
