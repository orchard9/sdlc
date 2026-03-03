# Design: ponder-agent-web-tools

## Overview

This is a targeted server-side change to `crates/sdlc-server/src/routes/runs.rs`. The implementation adds web research capabilities to ponder agent sessions by extending `QueryOptions` before spawning the agent. No new types, routes, or data structures are required.

## Current Architecture

```
start_ponder_chat()
    │
    ├── sdlc_query_options(root, 100)
    │     ├── PermissionMode::BypassPermissions
    │     ├── MCP: sdlc server (Bash, Read, Write, Edit, Glob, Grep, sdlc_* tools)
    │     └── allowed_tools: [Bash, Read, Write, Edit, Glob, Grep, sdlc_* MCP tools]
    │
    ├── opts.allowed_tools.push("mcp__sdlc__sdlc_ponder_chat")
    │
    └── spawn_agent_run(run_key, prompt, opts, ...)
```

## Target Architecture

```
start_ponder_chat()
    │
    ├── sdlc_ponder_query_options(root, 100)   ← new helper
    │     ├── sdlc_query_options (base)
    │     ├── WebSearch
    │     ├── WebFetch
    │     ├── MCP: playwright (npx @playwright/mcp@latest)
    │     └── Playwright tools: navigate, click, type, snapshot, screenshot,
    │                           console_messages, wait_for
    │
    ├── opts.allowed_tools.push("mcp__sdlc__sdlc_ponder_chat")
    │
    └── spawn_agent_run(run_key, prompt, opts, ...)
```

## Helper Function Pattern

The existing codebase has `sdlc_guideline_query_options` as a precedent for extending base options:

```rust
/// Build query options for guideline investigations — extends sdlc_query_options with
/// WebSearch and WebFetch for the Prior Art Mapper perspective.
pub(crate) fn sdlc_guideline_query_options(
    root: std::path::PathBuf,
    max_turns: u32,
) -> QueryOptions {
    let mut opts = sdlc_query_options(root, max_turns);
    opts.allowed_tools.push("WebSearch".into());
    opts.allowed_tools.push("WebFetch".into());
    opts
}
```

Following the same pattern, we add:

```rust
/// Build query options for ponder sessions — extends sdlc_query_options with
/// WebSearch, WebFetch, and Playwright MCP so agents can research prior art,
/// fetch documentation, and interact with real browser UIs during ideation.
pub(crate) fn sdlc_ponder_query_options(
    root: std::path::PathBuf,
    max_turns: u32,
) -> QueryOptions {
    let mut opts = sdlc_query_options(root, max_turns);
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
    opts
}
```

## Prompt Update

The ponder session prompt in `start_ponder_chat` should mention web tools so the agent knows to use them when appropriate. A brief addition after the "Run the session" heading is sufficient:

```
## Step 2 — Run the session

You have web research tools available when the idea benefits from external context:
- `WebSearch` — search for prior art, competitors, specifications, research papers
- `WebFetch` — fetch and read a specific URL (docs, APIs, blog posts)
- Playwright browser tools — navigate and interact with live products, capture screenshots

Use these when they strengthen the analysis. Do not use them gratuitously.
```

## Unchanged

- Session logging protocol (two-step Write → `sdlc ponder session log`)
- Status update logic
- Route signatures
- SSE event names and payload shapes
- All other agent run types (feature runs, UAT, investigation, advisory)
- `sdlc_query_options` (base options unchanged)

## File Changes

| File | Change |
|---|---|
| `crates/sdlc-server/src/routes/runs.rs` | Add `sdlc_ponder_query_options` helper; update `start_ponder_chat` to call it; add web tool mention to prompt |

No other files need changes.
