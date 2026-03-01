# Design: playwright-mcp-setup

## Overview

Two targeted changes wire Playwright MCP into the sdlc milestone UAT flow:

1. A new `.mcp.json` file at the repository root registers `@microsoft/playwright-mcp` so any MCP-aware client can discover and start the server.
2. A localised change in `start_milestone_uat` (in `runs.rs`) extends the `QueryOptions` it builds to include both the Playwright MCP server registration and its seven browser tools in `allowed_tools`.

All other agent endpoints remain unchanged.

## Architecture

```
sdlc-server
└── routes/runs.rs
    ├── sdlc_query_options()          ← unchanged baseline
    └── start_milestone_uat()         ← extended opts (adds playwright)
            │
            └── spawn_agent_run(key, prompt, opts, ...)
                        │
                        └── Claude agent (UAT)
                                ├── mcp__sdlc__*  (existing sdlc tools)
                                └── mcp__playwright__*  (NEW browser tools)
                                            │
                                            └── McpServerConfig {
                                                  name: "playwright",
                                                  command: "npx",
                                                  args: ["@playwright/mcp@latest"]
                                                }
```

## File Changes

### 1. `.mcp.json` (new file, project root)

Standard MCP server configuration file used by Claude Code and compatible tooling. Format follows the MCP specification:

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

This file enables human developers to have the same Playwright MCP server available when they run Claude Code interactively in this repository — the UAT agent and the developer share a consistent tool surface.

### 2. `crates/sdlc-server/src/routes/runs.rs`

#### Before (start_milestone_uat)

```rust
pub async fn start_milestone_uat(...) -> ... {
    validate_slug(&slug)?;
    let key = format!("milestone-uat:{slug}");
    let opts = sdlc_query_options(app.root.clone(), 200);
    let prompt = format!(...);
    let label = format!("UAT: {slug}");
    spawn_agent_run(key, prompt, opts, &app, "milestone_uat", &label, None).await
}
```

#### After (start_milestone_uat)

```rust
pub async fn start_milestone_uat(...) -> ... {
    validate_slug(&slug)?;
    let key = format!("milestone-uat:{slug}");

    // Start from the standard sdlc options, then extend with Playwright MCP.
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

    let prompt = format!(...);
    let label = format!("UAT: {slug}");
    spawn_agent_run(key, prompt, opts, &app, "milestone_uat", &label, None).await
}
```

## Design Decisions

### Why extend rather than replace?

`sdlc_query_options` sets up the sdlc MCP server, file-system tools, and sensible defaults. Replacing it would duplicate maintenance surface. Extending it is a one-liner pattern already established by `sdlc_guideline_query_options`, which does exactly this (adds `WebSearch` and `WebFetch`).

### Why only UAT?

Playwright opens a real browser subprocess. For all other agent types (ponder, investigate, advisory, etc.) that would be surprising and resource-intensive. Scoping it to UAT keeps the footprint minimal and intentional.

### Why `npx @playwright/mcp@latest`?

The `@playwright/mcp` package is designed to be run with npx without a local install. Using `@latest` ensures the UAT agent always gets a working, up-to-date browser automation surface without requiring a pinned npm dependency in the project.

### `.mcp.json` location

The MCP specification and Claude Code both look for `.mcp.json` at the repository root. Placing it there makes it effective for both the server-spawned UAT agent (which sets `cwd` to the project root via `QueryOptions`) and for developers using Claude Code interactively.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| `npx @playwright/mcp@latest` not on PATH at runtime | This is the same requirement as any other npx tool; if npx is available (it always is alongside Node.js), the command works. The UAT agent will fail gracefully if npx is absent. |
| Playwright browsers not installed | UAT failures will be self-explanatory ("browser not found"). Users run `npx playwright install` as a one-time setup, same as any Playwright project. |
| Version drift (`@latest`) | Acceptable for UAT tooling; Playwright MCP's API is stable. Pin later if needed. |
| Breaking other endpoints | Change is 100% local to `start_milestone_uat`; other callers of `sdlc_query_options` are untouched. |

## Sequence: Milestone UAT with Playwright

```
User → POST /api/milestone/{slug}/uat
            │
            ├── validate_slug
            ├── build opts = sdlc_query_options() + playwright extension
            └── spawn_agent_run(...)
                        │
                        └── Claude agent starts
                                ├── mcp__sdlc__sdlc_get_directive (reads acceptance test)
                                ├── mcp__playwright__browser_navigate (opens app URL)
                                ├── mcp__playwright__browser_snapshot (reads DOM state)
                                ├── mcp__playwright__browser_click / browser_type (interacts)
                                ├── mcp__playwright__browser_take_screenshot (evidence)
                                ├── mcp__playwright__browser_console_messages (checks errors)
                                └── mcp__sdlc__sdlc_write_artifact (writes uat_results.md)
```
