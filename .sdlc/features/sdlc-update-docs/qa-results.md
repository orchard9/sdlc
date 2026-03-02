# QA Results: Document sdlc update as Update Mechanism

## T1: README.md Updating section

- PASS: `## Updating` section exists at line 58 of `README.md`
- PASS: Section is placed immediately after the Install section (before "Initialize a project")
- PASS: `sdlc update` appears in a code block within the section
- PASS: Section explains refreshing AI scaffolding in `~/.claude/commands/`, `~/.gemini/commands/`, etc.
- PASS: Section mentions running after every binary upgrade

## T2: `sdlc init` completion message

- PASS: `crates/sdlc-cli/src/cmd/init/mod.rs` line 125 prints `"Next: sdlc ui    # then visit /setup to define Vision and Architecture"`
- PASS: Old `"Next: sdlc feature create <slug> --title \"...\""` message is absent

## Regression

- PASS: `SDLC_NO_NPM=1 cargo test --all` — all tests pass, no failures
- PASS: `cargo clippy --all -- -D warnings` — no new warnings, clean build

## Verdict

All checks pass. Ready for merge.
