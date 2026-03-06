# Spec: sdlc-recap-command

## Summary

Add a new `/sdlc-recap` slash command that produces a state-aware session recap with forward motion. The command synthesizes what was accomplished in a session, identifies remaining work, and creates concrete artifacts (tasks, ponder entries, escalations) so no session ends without clear next steps.

## Problem

Currently, when a UAT fails or a long agent session ends, there is no structured way to:
- Summarize what was accomplished in machine-readable sdlc terms
- Classify remaining work into actionable categories
- Create concrete forward artifacts (ponder entries, tasks, escalations) automatically
- Ensure every session ends with exactly one `**Next:**` line

Agents either stall, produce ad-hoc summaries that don't create follow-up artifacts, or leave the project in an ambiguous state.

## Solution

A new `/sdlc-recap` command installed via `sdlc init` / `sdlc update` that:

1. **Gathers state** — reads actual sdlc state (`sdlc status --json`, milestone info, git history), not just conversation context
2. **Synthesizes** — produces: Working On / Completed / Remaining / Forward Motion sections
3. **Takes action** — creates real sdlc artifacts for each remaining item (tasks, escalations, ponder entries)
4. **Commits** — commits completed work to git
5. **Ends decisively** — exactly one `**Next:**` line, always

## Scope

This is a **template/skill-only feature** — no new Rust code, no new CLI commands, no new server endpoints.

Changes:
- New `crates/sdlc-cli/src/cmd/init/commands/sdlc_recap.rs` — four platform variants (Claude, Gemini, OpenCode, Agent Skills)
- Register in `commands/mod.rs` ALL_COMMANDS array
- Add `sdlc-recap` entry to `AGENTS.md` consumer command list in `build_sdlc_section_inner()` in `mod.rs`
- Add to `GUIDANCE_MD_CONTENT` command table in `templates.rs`

## Acceptance Criteria

1. `sdlc update` installs `~/.claude/commands/sdlc-recap.md` with correct frontmatter
2. `/sdlc-recap` command reads sdlc state, milestone info, and recent git log
3. Command produces Working On / Completed / Remaining / Forward Motion sections
4. For each "Complex" remaining item, command runs `sdlc ponder create`
5. For each "Fixable" remaining item, command runs `sdlc task add`
6. Command commits completed work with a `session:` prefix commit message
7. Every recap ends with exactly one `**Next:**` line
8. Gemini, OpenCode, and Agent Skills variants are installed alongside Claude variant
9. `cargo build --all` and `cargo clippy --all -- -D warnings` pass with no new warnings
10. `SDLC_NO_NPM=1 cargo test --all` passes

## Out of Scope

- Server endpoint for recap runs (parked as optional wave 2)
- Frontend "Run Recap" button
- New Rust structs or data types
- Database storage of recap results
