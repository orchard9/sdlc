# QA Plan: Document sdlc update as Update Mechanism

## Checks

### T1: README.md Updating section
- "Updating" section exists in `README.md` immediately after the Install section
- Section contains `sdlc update` command in a code block
- Section explains that this refreshes AI command scaffolding in `~/.claude/commands/`, `~/.gemini/commands/`, etc.
- Section mentions running after every binary upgrade

### T2: `sdlc init` completion message
- `crates/sdlc-cli/src/cmd/init/mod.rs` no longer prints `sdlc feature create` as the next step
- Completion message now points to `sdlc ui` with instruction to visit `/setup`

### Regression
- `SDLC_NO_NPM=1 cargo test --all` passes with no failures
- `cargo clippy --all -- -D warnings` reports no new warnings
