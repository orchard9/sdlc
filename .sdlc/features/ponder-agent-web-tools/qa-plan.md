# QA Plan: ponder-agent-web-tools

## Approach

This is a server-side Rust change with no new routes or data structures. QA focuses on:
1. Code correctness — the right tools appear in `QueryOptions`
2. Compilation and lint — no warnings or errors
3. Test suite integrity — no regressions

All checks are automatable via `cargo` commands.

## Test Cases

### TC-1 — Build succeeds without errors or warnings

```bash
SDLC_NO_NPM=1 cargo build -p sdlc-server 2>&1 | grep -E "^error|^warning\[" | wc -l
```

**Pass:** output is `0` (no errors, no warnings)

### TC-2 — Full test suite passes

```bash
SDLC_NO_NPM=1 cargo test --all 2>&1 | tail -20
```

**Pass:** all tests pass, `test result: ok` shown for each crate

### TC-3 — Clippy clean

```bash
SDLC_NO_NPM=1 cargo clippy --all -- -D warnings 2>&1 | grep -c "^error"
```

**Pass:** output is `0`

### TC-4 — `sdlc_ponder_query_options` exists and includes expected tools

Code inspection to confirm:

- `sdlc_ponder_query_options` function exists in `crates/sdlc-server/src/routes/runs.rs`
- It calls `sdlc_query_options` as the base
- It adds `WebSearch` and `WebFetch` to `allowed_tools`
- It pushes a Playwright `McpServerConfig` with command `npx` and args `["@playwright/mcp@latest"]`
- It adds all 7 Playwright tool names to `allowed_tools`

```bash
grep -n "sdlc_ponder_query_options\|WebSearch\|WebFetch\|playwright\|mcp__playwright" \
  crates/sdlc-server/src/routes/runs.rs
```

**Pass:** all of the above are present in the output

### TC-5 — `start_ponder_chat` calls `sdlc_ponder_query_options`

```bash
grep -A2 "let mut opts" crates/sdlc-server/src/routes/runs.rs | grep -A1 "start_ponder_chat"
```

Alternative: code inspection confirming `start_ponder_chat` calls `sdlc_ponder_query_options` rather than `sdlc_query_options`.

**Pass:** `sdlc_ponder_query_options` is called in `start_ponder_chat`

### TC-6 — Ponder prompt mentions web tools

```bash
grep -c "WebSearch\|WebFetch\|mcp__playwright" crates/sdlc-server/src/routes/runs.rs
```

**Pass:** count is > 0 in context of the ponder prompt string

### TC-7 — No regression in ponder session logging behavior (code review)

The session logging instructions (two-step Write → `sdlc ponder session log` protocol) must remain intact in the prompt. Confirm the relevant sections are unchanged.

**Pass:** the Step 3 — Log the session and Step 4 — Update status sections appear verbatim in the prompt

## Regression Scope

Since only `start_ponder_chat` and the new `sdlc_ponder_query_options` function are changed:

- All other `spawn_agent_run` callers are unaffected
- `sdlc_query_options` is unchanged
- `sdlc_guideline_query_options` is unchanged
- `start_milestone_uat` is unchanged
- All ponder data operations (session log, capture, status update) are unchanged

The risk surface is minimal.
