# Review: ponder-agent-web-tools

## Summary

This feature adds WebSearch, WebFetch, and Playwright MCP tools to ponder agent sessions. The implementation is a targeted change to `crates/sdlc-server/src/routes/runs.rs`.

## Changes Reviewed

### 1. New helper: `sdlc_ponder_query_options` (lines 621–647)

```rust
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

**Assessment:** Correct. Follows the established `sdlc_guideline_query_options` pattern exactly. Placed immediately before `sdlc_query_options` for discoverability. `pub(crate)` visibility is appropriate — consistent with the other helpers. The Playwright config matches the `start_milestone_uat` config byte-for-byte.

### 2. Updated `start_ponder_chat` to call `sdlc_ponder_query_options`

```rust
// Before:
let mut opts = sdlc_query_options(app.root.clone(), 100);
// After:
let mut opts = sdlc_ponder_query_options(app.root.clone(), 100);
```

**Assessment:** Correct. Single-line change. The `sdlc_ponder_chat` tool is still added after (unchanged). The `max_turns: 100` is unchanged. All downstream logic is unaffected.

### 3. Prompt update — web tools mention in Step 2

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
```

**Assessment:** Correct. The instruction is well-placed and appropriately restrained ("when they would strengthen the analysis"). The existing Step 3 (session logging) and Step 4 (status update) sections are intact and unchanged.

## Checklist

- [x] `sdlc_ponder_query_options` exists and has correct signature
- [x] `WebSearch` added to `allowed_tools`
- [x] `WebFetch` added to `allowed_tools`
- [x] Playwright MCP server config present (`npx @playwright/mcp@latest`)
- [x] All 7 Playwright tool names present
- [x] `start_ponder_chat` calls `sdlc_ponder_query_options`
- [x] Ponder prompt mentions web tools in Step 2
- [x] Session logging protocol (two-step Write + `sdlc ponder session log`) unchanged
- [x] `sdlc_query_options` base function unchanged
- [x] `sdlc_guideline_query_options` unchanged
- [x] `start_milestone_uat` unchanged
- [x] `SDLC_NO_NPM=1 cargo build -p sdlc-server` — PASS
- [x] `SDLC_NO_NPM=1 cargo test --all` — PASS
- [x] `SDLC_NO_NPM=1 cargo clippy --all -- -D warnings` — PASS (zero errors, zero warnings)

## Findings

No findings. The implementation is minimal, follows established patterns, and passes all quality gates.

## Verdict

**APPROVED** — ready for audit.
