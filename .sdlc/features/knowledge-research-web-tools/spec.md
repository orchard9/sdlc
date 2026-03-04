# Spec: Knowledge research — web-capable agent with rewritten prompt

## Problem

The `POST /api/knowledge/:slug/research` endpoint spawns an agent that only searches the local codebase using `Grep` and `Read` tools. This severely limits its utility: researching a topic that has no local codebase coverage (e.g. a new library, a competitor, an external standard) yields empty or shallow results.

Meanwhile, the `POST /api/knowledge/ask` endpoint already adds `WebSearch` and `WebFetch` to `sdlc_query_options`, proving the pattern works. The research endpoint simply never received the same upgrade.

## Goal

Upgrade `research_knowledge` so that the spawned agent can:
1. Search the web for authoritative, up-to-date external information.
2. Still integrate local codebase context when relevant.
3. Produce richer, more useful knowledge entries.

## Scope

- **In scope:** Modify `build_research_prompt` and `research_knowledge` in `crates/sdlc-server/src/routes/knowledge.rs`.
- **Out of scope:** Changes to the knowledge data model, CLI commands, SSE event names, or the frontend.

## Acceptance Criteria

1. `research_knowledge` passes `WebSearch` and `WebFetch` in `opts.allowed_tools` before calling `spawn_agent_run`, mirroring the pattern in `ask_knowledge`.
2. `build_research_prompt` is rewritten to instruct the agent to:
   - First search the web for external information on the topic.
   - Also search the local codebase for related context.
   - Synthesize both into `content.md`.
   - Update the entry summary via `sdlc knowledge update`.
   - Log the research session.
3. The function signature and calling convention remain unchanged (no new parameters, no API changes).
4. All existing tests pass (`SDLC_NO_NPM=1 cargo test --all`).
5. Clippy passes with no warnings (`cargo clippy --all -- -D warnings`).

## Design Notes

The implementation is a targeted two-line change to `research_knowledge` (push two tools onto `opts.allowed_tools`) plus a prompt rewrite in `build_research_prompt`. No new structs, no new routes, no database changes.

The rewritten prompt should mirror the structure used in `ask_knowledge` (numbered steps, explicit tool names, explicit output format) while being tailored for research synthesis rather than question-answering.
