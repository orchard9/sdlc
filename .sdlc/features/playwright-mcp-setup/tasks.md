# Tasks: playwright-mcp-setup

## Task List

### T1: Create `.mcp.json` at project root

**What:** Create the file `/Users/jordanwashburn/Workspace/orchard9/sdlc/.mcp.json` with the MCP server registration for `@microsoft/playwright-mcp`.

**Content:**
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

**Done when:** The file exists, contains valid JSON, and matches the specified format exactly.

---

### T2: Extend `start_milestone_uat` in `runs.rs` with Playwright MCP

**What:** In `crates/sdlc-server/src/routes/runs.rs`, update the `start_milestone_uat` function to:
1. Build a mutable `opts` from `sdlc_query_options(app.root.clone(), 200)`
2. Push a new `McpServerConfig { name: "playwright", command: "npx", args: ["@playwright/mcp@latest"], env: HashMap::new() }` to `opts.mcp_servers`
3. Extend `opts.allowed_tools` with the seven `mcp__playwright__*` tool names

**Done when:** `SDLC_NO_NPM=1 cargo build --all` succeeds and `start_milestone_uat` contains all seven Playwright tool names in its `allowed_tools`.

---

### T3: Verify the build and validate output

**What:** Run `SDLC_NO_NPM=1 cargo build --all 2>&1 | tail -20` and confirm zero errors. Optionally grep `runs.rs` to confirm all seven tool names are present.

**Done when:** Build output shows no errors.
