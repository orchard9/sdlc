# Feature Specification: playwright-mcp-setup

## Summary

Register `@microsoft/playwright-mcp` as a Model Context Protocol (MCP) server for the sdlc project, and include Playwright MCP browser tools in the allowed tools list for milestone UAT agent runs.

## Background

The sdlc project drives feature lifecycles through AI agents that operate via Claude's tool system. Milestone UAT (`sdlc milestone uat`) is currently executed by a Claude agent, but that agent has no browser automation tools available. Adding Playwright MCP gives the UAT agent the ability to interact with real browser UI during acceptance tests — navigating pages, clicking elements, filling forms, taking screenshots, and reading console output.

## Problem Statement

1. There is no `.mcp.json` in the project root, so the `@microsoft/playwright-mcp` server is not registered anywhere the tooling can discover it.
2. The `start_milestone_uat` function in `crates/sdlc-server/src/routes/runs.rs` calls `sdlc_query_options` which does not include any Playwright MCP tools in `allowed_tools`. The UAT agent therefore cannot interact with a browser.

## Deliverables

### Deliverable 1: `.mcp.json` in the project root

Create `/Users/jordanwashburn/Workspace/orchard9/sdlc/.mcp.json` with the following content:

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

This file is the standard MCP server configuration that Claude Code and compatible AI CLIs use to discover and launch MCP servers.

### Deliverable 2: Playwright tools in UAT allowed list

In `crates/sdlc-server/src/routes/runs.rs`, update the `start_milestone_uat` function to build a custom `QueryOptions` that extends `sdlc_query_options` with the following additional allowed tools:

- `mcp__playwright__browser_navigate`
- `mcp__playwright__browser_click`
- `mcp__playwright__browser_type`
- `mcp__playwright__browser_snapshot`
- `mcp__playwright__browser_take_screenshot`
- `mcp__playwright__browser_console_messages`
- `mcp__playwright__browser_wait_for`

Additionally, the Playwright MCP server itself must be registered in `mcp_servers` within the UAT query options so Claude can discover and invoke it.

## Acceptance Criteria

1. `.mcp.json` exists at the project root with valid JSON matching the specified format.
2. `cargo build --all` succeeds with no errors after the Rust change.
3. `start_milestone_uat` in `runs.rs` uses an `opts` that includes all seven `mcp__playwright__*` tools in `allowed_tools`.
4. The Playwright MCP server (`playwright` / `npx @playwright/mcp@latest`) appears in the `mcp_servers` list for UAT runs.
5. All other endpoints that call `sdlc_query_options` are unaffected — the change is scoped only to the UAT function.

## Non-Goals

- Installing or verifying the Playwright npm package at build time.
- Adding Playwright tools to any endpoint other than `start_milestone_uat`.
- Modifying any frontend code.
- Writing automated tests for Playwright integration itself.

## Out of Scope

- Playwright browser installation (done separately via `npx playwright install`).
- Changing any other agent endpoint's tool allowlist.
