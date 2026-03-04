# Tasks: Knowledge research — web-capable agent with rewritten prompt

## T1: Add WebSearch and WebFetch to research_knowledge opts

In `crates/sdlc-server/src/routes/knowledge.rs`, change the `let opts = ...` line in `research_knowledge` to `let mut opts = ...` and push `"WebSearch"` and `"WebFetch"` onto `opts.allowed_tools`.

## T2: Rewrite build_research_prompt

Replace the body of `build_research_prompt` with a new prompt that:
- Instructs the agent to use `WebSearch` and `WebFetch` for external sources first.
- Instructs the agent to use `Grep` and `Read` for local codebase context second.
- Asks the agent to synthesize both into `content.md`.
- Asks the agent to run `sdlc knowledge update <slug> --summary "..."` after writing content.
- Asks the agent to log the session with `sdlc knowledge session log <slug> --content "..."`.
- Uses numbered steps and explicit tool names, matching the style in `ask_knowledge`.

## T3: Build and verify

Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings` to confirm the changes compile and pass all checks.
