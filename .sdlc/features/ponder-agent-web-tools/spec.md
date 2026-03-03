# Spec: ponder-agent-web-tools

## Problem

Ponder sessions are ideation workspaces where agents explore ideas, recruit thought partners, and build understanding before committing to milestones. Currently, ponder agent sessions are confined to local filesystem tools (`Bash`, `Read`, `Write`, `Edit`, `Glob`, `Grep`) and the sdlc MCP tools. Agents cannot search the web for prior art, fetch documentation pages, or perform browser-based research during a session.

This creates a gap: when an idea requires understanding the competitive landscape, researching existing solutions, or exploring industry patterns, the ponder agent must either work from stale knowledge or guess. This limits the quality of ponder output and makes sessions less valuable.

## Solution

Extend ponder agent sessions with three web-access capabilities:

1. **WebSearch** — allows the agent to search the web and retrieve current, up-to-date information about a topic (competitors, standards, prior art, research papers)
2. **WebFetch** — allows the agent to fetch and read a specific web page (documentation, specifications, blog posts, API references)
3. **Playwright MCP** — allows the agent to navigate a real browser, interact with web UIs, capture screenshots, and read accessibility trees; useful when a ponder session involves evaluating existing products or validating how a competitor's UX works

These tools are additive — they do not change the existing ponder session flow. The agent continues to follow the load → session → log → update lifecycle. Web tools simply expand what can be researched during Step 2.

## Scope

### In scope

- Add `WebSearch` to the `allowed_tools` list in `start_ponder_chat` in `crates/sdlc-server/src/routes/runs.rs`
- Add `WebFetch` to the `allowed_tools` list in `start_ponder_chat`
- Add Playwright MCP server config and Playwright tools to ponder sessions (following the same pattern as `start_milestone_uat`)
- Update the ponder session prompt to mention that the agent can search the web and use the browser for research
- No changes to `sdlc_query_options` (base query options remain unchanged — Playwright stays UAT-only at the base level; ponder extends it independently)

### Out of scope

- Investigation sessions (separate feature if needed)
- Advisory runs
- Feature agent runs
- Changing the session logging protocol
- Adding new ponder CLI commands

## Acceptance Criteria

1. `start_ponder_chat` in `runs.rs` includes `WebSearch` and `WebFetch` in `opts.allowed_tools`
2. `start_ponder_chat` pushes the Playwright MCP server config (`npx @playwright/mcp@latest`) into `opts.mcp_servers`
3. `start_ponder_chat` includes all Playwright tool names in `opts.allowed_tools` (same set as UAT: navigate, click, type, snapshot, screenshot, console_messages, wait_for)
4. The ponder session prompt includes a brief mention that the agent may use web search, web fetch, and browser tools when relevant to the research
5. Existing ponder session behavior is unchanged — no regressions in session logging, status updates, or the two-step log procedure
6. The server compiles without warnings
7. `SDLC_NO_NPM=1 cargo test --all` passes

## Implementation Notes

The implementation is a small, targeted change to `start_ponder_chat` in `crates/sdlc-server/src/routes/runs.rs`:

```rust
// After: let mut opts = sdlc_query_options(app.root.clone(), 100);
// Add:
opts.allowed_tools.push("WebSearch".into());
opts.allowed_tools.push("WebFetch".into());
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

A helper function `sdlc_ponder_query_options` can be extracted to keep `start_ponder_chat` readable (following the `sdlc_guideline_query_options` pattern).
