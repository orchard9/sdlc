# QA Plan: sdlc-recap-command

## Scope

This feature adds a new `/sdlc-recap` slash command via the `sdlc init` / `sdlc update` template system. No Rust business logic is added — the QA focus is on template correctness, proper registration in the command registry, and build integrity.

## QA Checklist

### 1. Build verification

- [ ] `SDLC_NO_NPM=1 cargo build --all` succeeds with no errors
- [ ] `cargo clippy --all -- -D warnings` produces no new warnings
- [ ] `SDLC_NO_NPM=1 cargo test --all` passes all tests

### 2. Command file generation

Run `sdlc update` (or `sdlc init` in a temp dir) and verify:
- [ ] `~/.claude/commands/sdlc-recap.md` is created with correct frontmatter (`description`, `argument-hint`, `allowed-tools`)
- [ ] `~/.gemini/commands/sdlc-recap.toml` is created with `description` and `prompt` fields
- [ ] `~/.opencode/command/sdlc-recap.md` is created with correct frontmatter
- [ ] `~/.agents/skills/sdlc-recap/SKILL.md` is created with SKILL.md frontmatter (`name`, `description`)

### 3. Claude command content verification

Inspect `~/.claude/commands/sdlc-recap.md`:
- [ ] Has `<!-- sdlc:guidance -->` marker in step 1
- [ ] Contains Step 1 (gather state: `sdlc status --json`, git log)
- [ ] Contains Step 2 (synthesize: Working On / Completed / Remaining / Forward Motion)
- [ ] Contains Step 3 (forward artifacts: `sdlc ponder create`, `sdlc task add`)
- [ ] Contains Step 4 (git commit with `session:` prefix)
- [ ] Ends with `**Next:**` rule table

### 4. AGENTS.md inclusion

After `sdlc init` / `sdlc update`:
- [ ] `AGENTS.md` includes `/sdlc-recap` in the Consumer Commands list

### 5. Guidance table inclusion

After `sdlc init` / `sdlc update`:
- [ ] `.sdlc/guidance.md` contains `sdlc-recap` in the command table

### 6. Legacy migration

- [ ] `migrate_legacy_project_scaffolding()` includes `sdlc-recap.md` in the file list to remove from project-level `.claude/commands/`

### 7. Idempotence

- [ ] Running `sdlc update` twice does not produce errors or duplicate entries

## Pass Criteria

All checklist items above must be checked. Build must be clean. No existing tests may regress.
