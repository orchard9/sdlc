# QA Plan: Standard Agents Scaffolding

## Q1: Build and clippy pass
- `SDLC_NO_NPM=1 cargo test --all` — no new test failures
- `cargo clippy --all -- -D warnings` — no warnings

## Q2: Init creates agent files
- Run `sdlc init` on a temp directory
- Verify `.claude/agents/knowledge-librarian.md` exists with correct frontmatter
- Verify `.claude/agents/cto-cpo-lens.md` exists with correct frontmatter

## Q3: Write-if-missing semantics
- Create `.claude/agents/knowledge-librarian.md` with custom content
- Run `sdlc init` — verify custom content is preserved (not overwritten)
- Run `sdlc update` — verify custom content is still preserved

## Q4: Update also creates missing agents
- Run `sdlc init` on a temp directory
- Delete `.claude/agents/cto-cpo-lens.md`
- Run `sdlc update` — verify it recreates the missing agent file

## Q5: Agent frontmatter valid
- Both agent files must have valid YAML frontmatter with `model`, `description`, `tools` fields
