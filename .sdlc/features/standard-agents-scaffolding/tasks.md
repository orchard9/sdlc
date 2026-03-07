# Tasks: Standard Agents Scaffolding

## T1: Add standard agent template constants
- Add `STANDARD_AGENT_KNOWLEDGE_LIBRARIAN` and `STANDARD_AGENT_CTO_CPO_LENS` const strings to `crates/sdlc-cli/src/cmd/init/mod.rs`
- Knowledge-librarian: generic version (no catalog YAML placeholder)
- CTO/CPO lens: frontmatter + role description matching existing agent

## T2: Add `write_standard_agents()` function
- New `pub fn write_standard_agents(root: &Path) -> anyhow::Result<()>` in `init/mod.rs`
- Creates `.claude/agents/` dir via `io::ensure_dir`
- Writes both agent files via `io::write_if_missing`
- Prints created/exists status for each

## T3: Integrate into `sdlc init`
- Call `write_standard_agents(root)?` in `run()` between step 6 (AGENTS.md) and step 7 (user scaffolding)
- Add section header print: `println!("\nInstalling standard agents:");`

## T4: Integrate into `sdlc update`
- Import `write_standard_agents` in `crates/sdlc-cli/src/cmd/update.rs`
- Call after `write_agents_md` with same section header

## T5: Update specialize template
- Add note to specialize command template about pre-existing standard agents
- "Standard agents (knowledge-librarian, cto-cpo-lens) are pre-installed. Design project-specific agents to complement them."

## T6: Test
- Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings`
- Verify no regressions
