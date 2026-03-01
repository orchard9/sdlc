# Code Review: playwright-mcp-setup

## Summary

Two files changed, one file created. All changes are minimal and well-scoped.

## Files Changed

### 1. `.mcp.json` (new file)

```json
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": ["@playwright/mcp@latest"]
    }
  }
}
```

**Review:** Correct format. Valid JSON. `npx @playwright/mcp@latest` is the documented way to run the Playwright MCP server without a local install. No issues.

### 2. `crates/sdlc-server/src/routes/runs.rs` — `start_milestone_uat`

The diff extends UAT opts to include Playwright:

```rust
let mut opts = sdlc_query_options(app.root.clone(), 200);
opts.mcp_servers.push(McpServerConfig {
    name: "playwright".into(),
    command: "npx".into(),
    args: vec!["@playwright/mcp@latest".into()],
    env: HashMap::new(),
});
opts.allowed_tools.extend([
    "mcp__playwright__browser_navigate".into(),
    "mcp__playwright__browser_click".into(),
    "mcp__playwright__browser_type".into(),
    "mcp__playwright__browser_snapshot".into(),
    "mcp__playwright__browser_take_screenshot".into(),
    "mcp__playwright__browser_console_messages".into(),
    "mcp__playwright__browser_wait_for".into(),
]);
```

**Review:**
- Pattern is consistent with `sdlc_guideline_query_options` — extend, don't replace. Correct.
- All seven tool names from the spec are present. Verified.
- `McpServerConfig` struct is used exactly as elsewhere in the file. Correct.
- `HashMap::new()` for `env` matches the baseline pattern. Correct.
- Change is scoped to `start_milestone_uat` only. No other endpoints affected. Verified.
- `cargo build --all` passes. Verified.

### 3. `CLAUDE.md` — Playwright MCP section added

Documents the Playwright MCP tools available in UAT, the prerequisite step (`npx playwright install`), and the extension pattern. Accurate and complete.

## Correctness Check

| Criterion | Status |
|---|---|
| `.mcp.json` valid JSON | PASS |
| `.mcp.json` format matches spec | PASS |
| All 7 Playwright tools in `allowed_tools` | PASS |
| `playwright` MCP server in `opts.mcp_servers` | PASS |
| Build passes (`SDLC_NO_NPM=1 cargo build --all`) | PASS |
| No other endpoints modified | PASS |
| `sdlc_query_options` unchanged | PASS |
| CLAUDE.md documents the new capability | PASS |

## Issues Found

None. The implementation is clean, minimal, and consistent with established patterns.

## Approval Recommendation

Approve. Ready for audit.
