# Review: dev-driver-agentic-dispatch

## Summary

This review covers the full implementation of the agentic dispatch rewrite for
`dev-driver`. All four tasks were completed cleanly. The changes bring dev-driver
in line with the rest of the codebase by routing agent dispatch through the server
infrastructure rather than a detached subprocess.

---

## Files Changed

| File | Change |
|------|--------|
| `crates/sdlc-server/src/routes/tools.rs` | Added `AgentDispatchRequest` struct + `agent_dispatch` handler (~80 lines) |
| `crates/sdlc-server/src/lib.rs` | Registered `POST /api/tools/agent-dispatch` route before `{name}` wildcard |
| `.sdlc/tools/_shared/agent.ts` | Added `AgentDispatchResult` interface + `runAgentDispatch()` function |
| `.sdlc/tools/dev-driver/tool.ts` | Removed lock file subsystem + `spawnClaude`; rewrote Level 1–4 dispatch |
| `.sdlc/tools/dev-driver/README.md` | Updated Level 1/3/4 descriptions, replaced lock file section with dispatch section |

---

## Correctness Assessment

### New Rust endpoint: `POST /api/tools/agent-dispatch`

- Bearer token validation matches the pattern used by `agent-call` (same `extract_bearer_token` + `app.agent_token` check)
- Empty string validation for `prompt`, `run_key`, `label` — returns 400 with clear message
- Delegates duplicate detection to `spawn_agent_run`, which returns 409 when `agent_runs` map already contains the key — no additional duplicate check needed here
- Returns immediately after `spawn_agent_run` succeeds — does NOT wait for completion (correct fire-and-forget behavior)
- `run_id` is extracted from the `spawn_agent_run` response JSON and echoed back to the caller
- `max_turns` defaults to 40, capped at 100 by `body.max_turns.unwrap_or(40).min(100)`

**Finding:** Feature dispatch uses `maxTurns: 40` and wave dispatch uses `maxTurns: 100`. These are reasonable but not configurable per feature type. Acceptable for now; could be revisited if specific features need different caps. No action needed.

### New TypeScript helper: `runAgentDispatch()`

- Environment variable check throws a clear error with usage instructions — matches pattern in `runAgentViaServer`
- 409 Conflict is handled as a non-throwing return (`status: 'conflict'`) — correct; caller decides what to do
- Other non-2xx errors throw — correct
- Timeout: 10 seconds — appropriate for a lightweight HTTP dispatch call
- JSON parse failure throws with clear message

**Finding:** The `node:http`-based `fetch()` is used (native since Node 18+). The existing codebase uses `fetch` in `runAgentViaServer` with no issues. No problem here.

### Updated `dev-driver/tool.ts`

- Lock file subsystem fully removed: `LockFile` interface, `LOCK_TTL_MINS`, `lockPath`, `readLock`, `isLockActive`, `lockAgeMins`, `writeLock` — all gone
- `spawnClaude` function removed
- `spawn` import from `node:child_process` removed; `writeFileSync` removed
- Level 1 is now the existing `hasActiveRuns()` check (queries `sdlc run list --status running`) — correct placement
- Level 3 calls `runAgentDispatch` with `run_key: dev-driver:feature:<slug>` and returns `waiting` on 409
- Level 4 calls `runAgentDispatch` with `run_key: dev-driver:wave:<milestone>` and returns `waiting` on 409
- Output type updated: `lock_age_mins` removed; `run_id` added to `feature_advanced` and `wave_started`
- `output_schema` in `meta` updated to match (no `lock_age_mins`, `run_id` added)
- File header updated to describe new dispatch pattern

**Finding:** `readFileSync` and `existsSync` imports are still present (used by `hasSkipTag`). This is correct — they were not removed because `hasSkipTag` reads the tasks file. No issue.

---

## Quality Checks

- `SDLC_NO_NPM=1 cargo test --all` — all tests pass (no regressions)
- `cargo clippy --all -- -D warnings` — clean
- `node tool.ts --meta` — returns valid JSON with correct schema (no `lock_age_mins`, `run_id` present)
- Running `tool.ts --run` outside server context — fails with clear error about `SDLC_SERVER_URL`

---

## Findings Disposition

| Finding | Disposition |
|---------|-------------|
| `maxTurns` not configurable per dispatch type | Accept — reasonable defaults (40/100), revisit if needed |
| No unit tests added for `agent_dispatch` Rust handler | Track — see task below |

### Task: Add unit tests for `agent_dispatch` handler

The QA plan (TC-1 through TC-5) specifies unit tests for the new Rust endpoint
(missing prompt → 400, missing run_key → 400, duplicate key → 409). These weren't
added in this cycle. Adding a task to track this:
