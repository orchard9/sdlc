# Tasks: /sdlc-knowledge Slash Command

## T1 — Create `sdlc_knowledge.rs` command file

Create `/Users/jordanwashburn/Workspace/orchard9/sdlc/crates/sdlc-cli/src/cmd/init/commands/sdlc_knowledge.rs` with:
- `SDLC_KNOWLEDGE_COMMAND` — Claude Code markdown with frontmatter (description, argument-hint, allowed-tools) and full five-mode playbook
- `SDLC_KNOWLEDGE_PLAYBOOK` — condensed Gemini + OpenCode body
- `SDLC_KNOWLEDGE_SKILL` — minimal agents SKILL.md with name/description frontmatter
- `pub static SDLC_KNOWLEDGE: CommandDef` referencing all three constants

## T2 — Register command in `commands/mod.rs`

Edit `/Users/jordanwashburn/Workspace/orchard9/sdlc/crates/sdlc-cli/src/cmd/init/commands/mod.rs`:
- Add `mod sdlc_knowledge;`
- Add `&sdlc_knowledge::SDLC_KNOWLEDGE` to `ALL_COMMANDS` after the guideline entry

## T3 — Update `GUIDANCE_MD_CONTENT` in `templates.rs`

Edit `/Users/jordanwashburn/Workspace/orchard9/sdlc/crates/sdlc-cli/src/cmd/init/templates.rs`:
- In section 6 "Using sdlc", add rows for `sdlc knowledge status`, `sdlc knowledge list`, `sdlc knowledge search`, `sdlc knowledge show`, `sdlc knowledge add`, `sdlc knowledge catalog show`, `sdlc knowledge librarian init`

## T4 — Build and test

Run:
```bash
SDLC_NO_NPM=1 cargo build --all
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

All must pass with no errors or warnings.
