# Security Audit: Credential Pool Runs

## Scope

Changes to `crates/sdlc-server/src/routes/runs.rs`:
- `spawn_agent_run` — added `checkout_from_pool` call and env injection.
- New private `checkout_from_pool` helper.
- 6 new unit tests.

## Threat Surface

The change handles OAuth tokens sourced from a PostgreSQL-backed credential pool and
injects them into subprocess environment variables passed to the Claude binary and
its MCP server child processes.

## Findings

### F1: Token exposure via env vars — ACCEPT with rationale

**Risk:** Subprocess environment variables are readable by other processes on the
same machine with sufficient privileges (`/proc/<pid>/environ` on Linux). Claude and
its MCP children are already launched as the server process's uid, so they inherit
the server's ambient trust level. The risk is identical to the existing pattern
(callers already embed tokens in MCP server env via `sdlc_query_options`). The
change does not worsen this surface.

**Rationale:** The Claude SDK requires the token via env var — this is the
only supported injection mechanism. Process isolation is a host-level control, not
an application-level control.

**Action:** Accept. No change required.

### F2: Token not logged — PASS

The `checkout_from_pool` helper logs only `account_name` (via `info!` in
`CredentialPool::checkout`), never the token value. The injection block handles an
`Option<String>` and does not log it. No token leak via logs.

### F3: Error handling swallows errors — ACCEPT

DB errors in `checkout_from_pool` are logged at `error!` level and return `None`,
allowing the run to proceed with ambient auth. This is the correct resilience
posture: a pool failure does not prevent agent runs from executing. An alternative
(fail-closed: abort the run on checkout error) would be more secure but would break
production runs whenever the pool DB is temporarily unreachable.

**Action:** Accept. The current design matches the stated fallback contract.

### F4: `entry().or_insert_with()` vs direct overwrite — PASS

Using `or_insert_with` preserves tokens set by callers, which prevents a
(hypothetical) downgrade where a more-specific caller-provided token is replaced by
a less-specific pool token. This is the correct behaviour.

### F5: Double checkout side effect — LOW RISK

Callers that already call `checkout_claude_token` trigger a second checkout inside
`spawn_agent_run`. The second checkout updates `last_used_at` and `use_count` in the
DB but uses a different credential (LRU selects the next one). The second checkout's
token is discarded via `or_insert_with`. Net effect: a credential that was not used
by the actual agent run has its `use_count` incremented.

This skews LRU ordering slightly but does not cause incorrect auth, token exposure,
or service degradation. A follow-up task was logged in the review to remove the
redundant call sites.

**Action:** Accept as low risk. Tracked for follow-up.

### F6: No secret in test code — PASS

Unit tests use literal strings (`"tok_pool"`, `"caller_tok"`) — not real credentials.
No hardcoded secrets.

## Summary

| Finding | Severity | Action |
|---|---|---|
| F1: Token in env vars | Low (pre-existing) | Accept |
| F2: Token not logged | N/A (pass) | None |
| F3: Error swallowed | Low | Accept |
| F4: or_insert_with correct | N/A (pass) | None |
| F5: Double checkout | Low | Track follow-up |
| F6: No secrets in tests | N/A (pass) | None |

No blocking findings. Feature is safe to merge.
