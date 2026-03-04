# Design: Knowledge research — web-capable agent with rewritten prompt

## Overview

This is a targeted two-part change to `crates/sdlc-server/src/routes/knowledge.rs`:

1. **Tool injection** — add `WebSearch` and `WebFetch` to the allowed tools list before spawning the research agent.
2. **Prompt rewrite** — update `build_research_prompt` to instruct the agent to use those web tools effectively.

No new structs, routes, data model changes, or frontend changes are needed.

## Change 1: Tool injection in `research_knowledge`

Current code:
```rust
let opts = sdlc_query_options(app.root.clone(), 20);
```

After change:
```rust
let mut opts = sdlc_query_options(app.root.clone(), 20);
opts.allowed_tools.push("WebSearch".into());
opts.allowed_tools.push("WebFetch".into());
```

This mirrors the identical pattern used by `ask_knowledge` (line ~720 in `knowledge.rs`).

## Change 2: Rewritten `build_research_prompt`

The current prompt only instructs the agent to search the local codebase. The rewritten prompt will:

1. Search the web first (`WebSearch`, `WebFetch`) for authoritative external sources.
2. Search the local codebase second (`Grep`, `Read`) for internal context and existing usage.
3. Synthesize findings into `content.md` with a clear structure (external findings, local context, summary).
4. Update the entry summary with `sdlc knowledge update <slug> --summary "<one-line summary>"`.
5. Log the session with `sdlc knowledge session log <slug> --content "<brief summary>"`.

The prompt will use numbered steps with explicit tool names, consistent with the `ask_knowledge` style.

## Affected Files

- `crates/sdlc-server/src/routes/knowledge.rs` — the only file changed

## Test Coverage

Existing integration tests cover the research endpoint. No new test fixtures are needed because the prompt and tool list are runtime strings — not compiled logic. Manual verification via the running server is the appropriate QA method.
