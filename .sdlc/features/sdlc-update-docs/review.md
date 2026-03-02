# Review: Document sdlc update as Update Mechanism

## Changes Implemented

### T1: README.md — "Updating" section

Added immediately after the Install section:

```markdown
## Updating

To upgrade the sdlc binary, re-run your install command (or `brew upgrade sdlc` if installed via Homebrew).

After upgrading the binary, run:

```bash
sdlc update
```

This refreshes your AI command scaffolding — the `/sdlc-*` slash commands installed in `~/.claude/commands/`, `~/.gemini/commands/`, etc. Run this after every sdlc binary upgrade to keep your AI tools in sync.
```

Section is correctly placed between Install and "Initialize a project".

### T2: `sdlc init` completion message — `crates/sdlc-cli/src/cmd/init/mod.rs` line 124

Before:
```rust
println!("Next: sdlc feature create <slug> --title \"...\"");
```

After:
```rust
println!("Next: sdlc ui    # then visit /setup to define Vision and Architecture");
```

Directs first-time users to the correct first step: opening the UI and configuring Vision/Architecture in Setup before creating features.

## Verification

- `SDLC_NO_NPM=1 cargo test --all` — all tests pass
- `cargo clippy --all -- -D warnings` — no new warnings
- README.md diff is additive only (8 lines added)
- init/mod.rs change is a single line, no logic changes

## Findings

None. Both changes are minimal and low-risk. No test changes required.
