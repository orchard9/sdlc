# Code Review: tool-agent-invocation

## Summary

This feature adds the ability for SDLC tools to invoke Claude agent runs from within their execution — the recruit-if-missing pattern. It ships three pieces: a new `POST /api/tools/agent-call` server endpoint, `SDLC_AGENT_TOKEN` injection into tool subprocess environments, and a refactored `_shared/agent.ts` that exposes the `runAgent()` API.

Overall the implementation is clean and follows project conventions. Three findings are documented below — one fix applied inline, two tracked as tasks.

---

## Findings

### F1 — Route was mis-wired (FIXED)

**Severity:** Blocker

`lib.rs` registered `/api/tools/agent-call` against `routes::runs::agent_call` — the dev-driver fire-and-forget handler that does not validate `SDLC_AGENT_TOKEN`. The spec requires `routes::tools::agent_call` (blocking, token-validated). Fixed during implementation.

**Resolution:** Route corrected in `crates/sdlc-server/src/lib.rs`.

---

### F2 — Pre-existing compile error in `auth_config.rs` (FIXED)

**Severity:** Blocker (build-time)

`crates/sdlc-core/src/auth_config.rs` referenced `rand::thread_rng()` but `rand` was not in `sdlc-core/Cargo.toml`. This caused the entire workspace to fail to compile. Replaced with a `/dev/urandom` + base62 implementation matching the pattern already used in `state.rs`'s `generate_agent_token()`.

**Resolution:** `auth_config.rs` `generate_token()` now uses `std::fs::File::open("/dev/urandom")` + `read_exact`. No new dependency added.

---

### F3 — `SdlcError` new variants not exhaustively handled in server (FIXED)

**Severity:** Blocker (compile-time)

`AuthTokenExists` and `AuthTokenNotFound` variants were added to `sdlc-core/src/error.rs` by the `auth-named-tokens` feature but were absent from the `IntoResponse` match in `sdlc-server/src/error.rs`. Added to the correct arms (`CONFLICT` and `NOT_FOUND` respectively).

**Resolution:** `crates/sdlc-server/src/error.rs` updated.

---

### F4 — `runAgent` vs `runAgentCli` naming conflict (FIXED)

**Severity:** Blocker (API incompatibility)

The prior `_shared/agent.ts` exported a synchronous `runAgent(agentPath, prompt, opts?)` function. The spec requires an async `runAgent({ prompt, agentFile?, maxTurns? }): Promise<string>`. These are incompatible — same export name, different signatures. Resolved by renaming the synchronous variant to `runAgentCli` (which more accurately describes what it does) and updating the only consumer (`beat/tool.ts`) to import `runAgentCli as runAgent` for backward compatibility.

**Resolution:** `_shared/agent.ts` refactored; `beat/tool.ts` import updated.

---

### F5 — `runAgent` should return `{ ok: false }` not throw for agent-file-not-found (IMPLEMENTED PER SPEC)

The spec's acceptance criteria say callers must be able to distinguish "agent file not found" from hard errors (server down, invalid token). Implemented: when the server returns HTTP 400 with an error message containing `agent_file`, `runAgent` returns `{ ok: false, error: 'agent file not found' }`. All other errors propagate as thrown exceptions. `RunAgentError` interface exported for caller type-checking.

---

## Spec Compliance

| Acceptance Criterion | Status |
|---|---|
| `_shared/agent.ts` exports `runAgent()` | PASS |
| `runAgent` sends `SDLC_AGENT_TOKEN` as `Authorization: Bearer` | PASS |
| `/api/tools/agent-call` returns 401 on missing/wrong token | PASS — 4 unit tests added |
| `SDLC_SERVER_URL` and `SDLC_AGENT_TOKEN` injected for every tool run | PASS — both `run_tool` and `setup_tool` handlers inject them |
| Agent-file-not-found handled gracefully (no throw) | PASS |
| Recruit-if-missing pattern documented | PASS — COMMAND, PLAYBOOK, SKILL variants all updated |
| Build passes, no new `unwrap()` in library code | PASS — all tests pass, clippy clean |

---

## Test Coverage

- `extract_bearer_token_parses_valid_header` — unit test
- `extract_bearer_token_returns_none_for_missing_header` — unit test
- `agent_call_returns_401_for_missing_token` — integration-style unit test
- `agent_call_returns_401_for_wrong_token` — integration-style unit test
- All 49 workspace tests pass, clippy reports zero warnings

---

## Verdict

APPROVE — all spec requirements met, all blockers fixed inline, no deferred debt.
