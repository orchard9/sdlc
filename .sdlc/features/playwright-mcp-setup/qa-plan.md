# QA Plan: playwright-mcp-setup

## Scope

This QA plan covers the two deliverables:
1. `.mcp.json` registration file at the project root.
2. Playwright MCP tool integration in `start_milestone_uat`.

## Test Scenarios

### QA-1: `.mcp.json` exists and is valid JSON

**Steps:**
1. Check that `/Users/jordanwashburn/Workspace/orchard9/sdlc/.mcp.json` exists.
2. Parse it as JSON (`python3 -m json.tool .mcp.json` or equivalent).
3. Verify it contains a `mcpServers.playwright` key with `command: "npx"` and `args: ["@playwright/mcp@latest"]`.

**Pass criteria:** File exists, parses without error, and contains the exact specified structure.

### QA-2: Rust build succeeds

**Steps:**
1. Run `SDLC_NO_NPM=1 cargo build --all 2>&1 | tail -20`.
2. Confirm output ends with `Finished` with no errors.

**Pass criteria:** Zero compilation errors.

### QA-3: `start_milestone_uat` contains all seven Playwright tools

**Steps:**
1. Grep `crates/sdlc-server/src/routes/runs.rs` for each tool name:
   - `mcp__playwright__browser_navigate`
   - `mcp__playwright__browser_click`
   - `mcp__playwright__browser_type`
   - `mcp__playwright__browser_snapshot`
   - `mcp__playwright__browser_take_screenshot`
   - `mcp__playwright__browser_console_messages`
   - `mcp__playwright__browser_wait_for`
2. Confirm each appears at least once in the file.

**Pass criteria:** All seven tool names found.

### QA-4: Playwright MCP server registered in UAT opts

**Steps:**
1. Read the `start_milestone_uat` function in `runs.rs`.
2. Confirm it pushes a `McpServerConfig` with `name: "playwright"`, `command: "npx"`, and `args: ["@playwright/mcp@latest"]` to `opts.mcp_servers`.

**Pass criteria:** The Playwright MCP server configuration is present inside `start_milestone_uat`.

### QA-5: Other endpoints unaffected

**Steps:**
1. Read `sdlc_query_options` function — confirm it has no Playwright tools.
2. Check `start_run`, `start_milestone_prepare`, `start_milestone_run_wave`, and a sample investigation endpoint — confirm none have Playwright in their tool lists.

**Pass criteria:** Playwright tools appear only inside `start_milestone_uat`, not in `sdlc_query_options` or any other endpoint.

### QA-6: Clippy passes

**Steps:**
1. Run `SDLC_NO_NPM=1 cargo clippy --all -- -D warnings 2>&1 | tail -30`.
2. Confirm no warnings or errors.

**Pass criteria:** Zero clippy warnings.

## Exit Criteria

All six QA scenarios pass before the feature is marked for review.
