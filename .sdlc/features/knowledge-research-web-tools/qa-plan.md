# QA Plan: Knowledge research — web-capable agent with rewritten prompt

## Automated checks

1. `SDLC_NO_NPM=1 cargo test --all` — all tests pass.
2. `cargo clippy --all -- -D warnings` — zero warnings.

## Code review checks

3. Confirm `research_knowledge` has `let mut opts = sdlc_query_options(...)` and pushes `"WebSearch"` and `"WebFetch"` onto `opts.allowed_tools`.
4. Confirm `build_research_prompt` instructs web search before local search.
5. Confirm the prompt includes explicit steps for: web search, local search, write content.md, update summary, log session.
6. Confirm function signature of `build_research_prompt` is unchanged.
7. Confirm no new routes, structs, or SSE variants were added.

## Manual smoke test (optional, requires running server)

8. Start `sdlc ui` and navigate to a knowledge entry.
9. Trigger research on a topic with no local codebase coverage.
10. Verify the agent run completes and `content.md` is populated with web-sourced content.
