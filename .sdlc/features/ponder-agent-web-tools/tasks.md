# Tasks: ponder-agent-web-tools

## T1 — Add `sdlc_ponder_query_options` helper to runs.rs

**File:** `crates/sdlc-server/src/routes/runs.rs`

Add a new `pub(crate)` function `sdlc_ponder_query_options` that extends `sdlc_query_options` with WebSearch, WebFetch, and Playwright MCP (following the `sdlc_guideline_query_options` pattern):

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

Place it immediately after `sdlc_guideline_query_options` for discoverability.

## T2 — Update `start_ponder_chat` to use `sdlc_ponder_query_options`

**File:** `crates/sdlc-server/src/routes/runs.rs`

Change the single line that builds query options in `start_ponder_chat`:

```rust
// Before:
let mut opts = sdlc_query_options(app.root.clone(), 100);

// After:
let mut opts = sdlc_ponder_query_options(app.root.clone(), 100);
```

The rest of `start_ponder_chat` remains unchanged.

## T3 — Add web tools mention to the ponder session prompt

**File:** `crates/sdlc-server/src/routes/runs.rs`

Update the prompt in `start_ponder_chat` to inform the agent about available web tools. Insert a paragraph at the start of the "Step 2 — Run the session" section:

```
## Step 2 — Run the session\n\
\n\
You have web research tools available when the idea benefits from external context:\n\
- `WebSearch` — search for prior art, competitors, specifications, research papers\n\
- `WebFetch` — fetch and read a specific URL (docs, APIs, blog posts)\n\
- Playwright browser tools (`mcp__playwright__browser_*`) — navigate live products, \
  capture screenshots, read accessibility trees\n\
\n\
Use these tools when they would strengthen the analysis. Do not use them gratuitously.\n\
\n\
```

## T4 — Verify compilation and tests pass

```bash
SDLC_NO_NPM=1 cargo build -p sdlc-server 2>&1
SDLC_NO_NPM=1 cargo test --all 2>&1
cargo clippy --all -- -D warnings 2>&1
```

All must pass without errors or warnings.
