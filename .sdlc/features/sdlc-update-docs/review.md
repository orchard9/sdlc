# Review: Document sdlc update as Update Mechanism

## Summary

Two targeted changes to address undocumented `sdlc update` command and a misleading `sdlc init` completion message.

## Changes Reviewed

### 1. README.md — `### Updating` section

**Location:** `README.md` line 58, inserted after the `### Install` subsection within `## Quickstart`

**Change:** Added an 11-line "Updating" section with:
- Instructions to re-run the install command or `brew upgrade sdlc`
- The `sdlc update` command
- Explanation of what `sdlc update` does (refreshes AI command scaffolding in `~/.claude/commands/`, etc.)
- Reminder to run after every binary upgrade

**Assessment:** Correct placement, accurate content, appropriate scope. The section is concise and actionable. No redundancy with existing CLI reference (line 324 lists `sdlc update` without explanation — the new section provides the upgrade context that makes `update` discoverable).

**Finding:** None.

### 2. `crates/sdlc-cli/src/cmd/init/mod.rs` line 127 — Completion message

**Before:**
```rust
println!("Next: sdlc feature create <slug> --title \"...\"");
```

**After:**
```rust
println!("Next: sdlc ui    # then visit /setup to define Vision and Architecture");
```

**Assessment:** Correct fix. The old message sent first-time users directly to feature creation, skipping the Vision and Architecture setup that makes the AI agent tools useful. The new message directs users to `sdlc ui` and `/setup`, which is the right first step for a new project.

**Finding:** None.

## Quality Checks

- `SDLC_NO_NPM=1 cargo build --all` — **passed** (clean, no new warnings)
- `SDLC_NO_NPM=1 cargo clippy --all -- -D warnings` — **passed** (zero new warnings)
- `SDLC_NO_NPM=1 cargo test --all` — **passed** (all tests pass, none broken)

## Verdict

**APPROVED.** Both changes are correct, minimal, and well-scoped. The documentation gap is filled and the init UX flows to the right first step.
