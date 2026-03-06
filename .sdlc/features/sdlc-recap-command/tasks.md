# Tasks: sdlc-recap-command

## T1 — Create `sdlc_recap.rs` command module

Create `crates/sdlc-cli/src/cmd/init/commands/sdlc_recap.rs` with:
- `SDLC_RECAP_COMMAND` const — full Claude Code slash command (frontmatter + 5-step body)
- `SDLC_RECAP_PLAYBOOK` const — compact Gemini/OpenCode playbook variant
- `SDLC_RECAP_SKILL` const — Agent Skills SKILL.md with frontmatter
- `pub static SDLC_RECAP: CommandDef` — registers all three variants

## T2 — Register in `commands/mod.rs`

In `crates/sdlc-cli/src/cmd/init/commands/mod.rs`:
- Add `mod sdlc_recap;`
- Add `&sdlc_recap::SDLC_RECAP` to the `ALL_COMMANDS` array (near the end, after `sdlc_convo_mine`)

## T3 — Add to AGENTS.md consumer commands

In `build_sdlc_section_inner()` in `crates/sdlc-cli/src/cmd/init/mod.rs`:
- Add `/sdlc-recap [slug]` entry to the Consumer Commands list with description

## T4 — Add to GUIDANCE_MD_CONTENT command table

In `crates/sdlc-cli/src/cmd/init/templates.rs`:
- Add `sdlc-recap` entry to the command reference table in `GUIDANCE_MD_CONTENT`

## T5 — Build and lint verification

Run:
```bash
SDLC_NO_NPM=1 cargo build --all 2>&1
cargo clippy --all -- -D warnings 2>&1
SDLC_NO_NPM=1 cargo test --all 2>&1
```
Fix any errors or warnings before marking complete.
