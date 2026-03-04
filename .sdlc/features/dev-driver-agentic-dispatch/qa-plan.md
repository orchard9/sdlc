# QA Plan: dev-driver-agentic-dispatch

## Test Strategy

This feature modifies the dispatch mechanism inside a TypeScript tool and adds a new Rust HTTP endpoint. Testing covers:

1. Rust unit tests for the new `agent_dispatch` endpoint
2. TypeScript integration: verify `_shared/agent.ts` sends the correct HTTP request
3. Behavioral regression: dev-driver's 5-level waterfall must behave identically to before (minus the lock file)

## Test Cases

### TC-1: `POST /api/tools/agent-dispatch` — valid request dispatches run

**Steps:**
- Start the SDLC server against a temp directory
- POST `{ "prompt": "/sdlc-next test-feature", "run_key": "dev-driver:feature:test-feature", "label": "test" }` to `/api/tools/agent-dispatch`

**Expected:** 202 response with `{ run_id: "...", run_key: "dev-driver:feature:test-feature", status: "started" }`

### TC-2: `POST /api/tools/agent-dispatch` — duplicate run_key returns 409

**Steps:**
- POST a valid agent-dispatch request
- POST the same request again (same run_key) before the first run completes

**Expected:** Second POST returns 409 Conflict with `{ error: "Agent already running for '...'" }`

### TC-3: `POST /api/tools/agent-dispatch` — missing prompt returns 400

**Steps:**
- POST `{ "run_key": "dev-driver:feature:x", "label": "test" }` (no prompt)

**Expected:** 400 Bad Request

### TC-4: `POST /api/tools/agent-dispatch` — empty prompt returns 400

**Steps:**
- POST `{ "prompt": "", "run_key": "dev-driver:feature:x", "label": "test" }`

**Expected:** 400 Bad Request

### TC-5: `POST /api/tools/agent-dispatch` — empty run_key returns 400

**Steps:**
- POST `{ "prompt": "/sdlc-next x", "run_key": "", "label": "test" }`

**Expected:** 400 Bad Request

### TC-6: `_shared/agent.ts` — 202 maps to `status: started`

**Steps:**
- Mock HTTP server returns 202 `{ run_id: "abc", run_key: "k", status: "started" }`
- Call `runAgentDispatch("/sdlc-next x", "k", "l")`

**Expected:** Returns `{ status: 'started', run_id: 'abc', run_key: 'k' }`

### TC-7: `_shared/agent.ts` — 409 maps to `status: conflict` (no throw)

**Steps:**
- Mock HTTP server returns 409
- Call `runAgentDispatch(...)`

**Expected:** Returns `{ status: 'conflict', run_id: '', run_key: 'k' }` — no exception thrown

### TC-8: `_shared/agent.ts` — 500 throws

**Steps:**
- Mock HTTP server returns 500
- Call `runAgentDispatch(...)`

**Expected:** Promise rejects with an Error

### TC-9: dev-driver — no lock file written after dispatch

**Steps:**
- Run dev-driver against a project with an actionable feature
- Check `.sdlc/.dev-driver.lock` does not exist

**Expected:** Lock file absent

### TC-10: dev-driver — `feature_advanced` result includes `run_id`

**Steps:**
- Run dev-driver against a project with an actionable feature in implementation phase
- Server responds with run_id

**Expected:** Output `{ action: "feature_advanced", slug: "...", run_id: "20260303-..." }`

### TC-11: dev-driver — Level 1 uses `hasActiveRuns` (no lock check)

**Steps:**
- Create a running agent run in the server
- Run dev-driver

**Expected:** dev-driver returns `{ action: "waiting", reason: "agent run in progress" }` immediately, without reading any lock file

### TC-12: cargo test passes

**Steps:**
- `SDLC_NO_NPM=1 cargo test --all`

**Expected:** All tests pass, no regressions

### TC-13: cargo clippy passes

**Steps:**
- `cargo clippy --all -- -D warnings`

**Expected:** No warnings

## Regression Checklist

- [ ] dev-driver --meta still returns valid JSON
- [ ] dev-driver Level 2 (quality check) still works
- [ ] dev-driver Level 3 (feature selection) still picks correct feature
- [ ] dev-driver Level 4 (wave detection) still picks correct milestone
- [ ] dev-driver Level 5 (idle) still returns correct output
- [ ] `GET /api/tools` still lists dev-driver correctly
- [ ] All other agent dispatch paths (ponder, investigation, feature run, etc.) are unaffected
