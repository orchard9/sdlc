# QA Plan: _shared/agent.ts — Agent Invocation from Within Tools

## Approach

Primary verification is Rust unit tests (no network, no agent spawning). The TypeScript `_shared/agent.ts` module is verified by inspection and compile-time checks (Bun type-check). No browser automation required — this is a server-side and shared-lib feature.

## Test Cases

### TC-1: `agent_token` is populated in `AppState`

**Method**: Rust unit test  
**Command**: `SDLC_NO_NPM=1 cargo test -p sdlc-server agent_token`  
**Assert**: `AppState::new(dir.path()).agent_token` is non-empty and 32 chars long

---

### TC-2: `SDLC_AGENT_TOKEN` is injected into tool subprocess env

**Method**: Rust unit test (mocked subprocess env inspection)  
**Command**: `SDLC_NO_NPM=1 cargo test -p sdlc-server inject_agent_token`  
**Assert**: The `extra_env` map passed to `tool_runner::run_tool` contains `SDLC_AGENT_TOKEN` and `SDLC_SERVER_URL` keys with non-empty values

---

### TC-3: `/api/tools/agent-call` returns 401 with no Authorization header

**Method**: Rust unit test (axum handler called directly without spawning agent)  
**Command**: `SDLC_NO_NPM=1 cargo test -p sdlc-server agent_call_401_missing`  
**Assert**: Response status is 401

---

### TC-4: `/api/tools/agent-call` returns 401 with wrong token

**Method**: Rust unit test  
**Command**: `SDLC_NO_NPM=1 cargo test -p sdlc-server agent_call_401_wrong`  
**Assert**: Response status is 401

---

### TC-5: `extract_bearer_token` parses valid `Authorization` header

**Method**: Rust unit test  
**Command**: `SDLC_NO_NPM=1 cargo test -p sdlc-server extract_bearer_token`  
**Assert**: Returns `Some("abc123")` for `"Bearer abc123"`, `None` for missing/malformed headers

---

### TC-6: `/api/tools/agent-call` route is registered before `{name}` wildcard

**Method**: Rust integration test (router construction)  
**Command**: `SDLC_NO_NPM=1 cargo test -p sdlc-server agent_call_route_resolves`  
**Assert**: `POST /api/tools/agent-call` with a valid body (but wrong token) returns 401, not 404 — confirming the route exists and is not swallowed by the `{name}` wildcard

---

### TC-7: `_shared/agent.ts` type-checks cleanly

**Method**: Bun type-check  
**Command**: `cd .sdlc/tools && bun x tsc --noEmit --strict _shared/agent.ts` (if bun is available; otherwise skip)  
**Assert**: Zero TypeScript errors

---

### TC-8: `_shared/agent.ts` throws when env vars are absent

**Method**: Code inspection  
**Assert**: `runAgent` checks for `!serverUrl || !token` and throws a descriptive `Error` — confirmed by reading the file

---

### TC-9: Build and lint pass

**Method**: CI commands  
**Commands**:
```bash
SDLC_NO_NPM=1 cargo build --all 2>&1 | tail -5
SDLC_NO_NPM=1 cargo test --all 2>&1 | tail -20
cargo clippy --all -- -D warnings 2>&1 | tail -10
```
**Assert**: All three commands exit 0

---

## Pass Criteria

All nine test cases pass. No regressions in existing tool tests (`list_tools`, `run_tool`, `setup_tool`, `create_tool`). No new `unwrap()` in library or server code.
