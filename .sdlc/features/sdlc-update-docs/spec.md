# Spec: Document sdlc update as Update Mechanism

## Problem

`sdlc update` exists and works — it refreshes the agent command scaffolding in `~/.claude/commands/`, `~/.gemini/commands/`, `~/.opencode/command/`, and `~/.agents/skills/` — but it is completely undocumented in any user-facing location. Users who upgrade the sdlc binary have no way to know they should run `sdlc update` to sync their AI command templates.

Additionally, the `sdlc init` completion message tells users to run `sdlc feature create` as their next step, which is wrong for first-time users who should open the UI and go to Setup first.

## Changes

### README.md — Updating section

**File:** `README.md`

Add an "Updating" section immediately after the Install section (~8 lines):

```markdown
## Updating

To upgrade the sdlc binary, re-run your install command (or `brew upgrade sdlc` if installed via Homebrew).

After upgrading the binary, run:

```bash
sdlc update
```

This refreshes your AI command scaffolding — the `/sdlc-*` slash commands installed in `~/.claude/commands/`, `~/.gemini/commands/`, etc. Run this after every sdlc binary upgrade to keep your AI tools in sync.
```

### `crates/sdlc-cli/src/cmd/init/mod.rs` — Completion message

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`, line 119

Before:
```rust
println!("Next: sdlc feature create <slug> --title \"...\"");
```

After:
```rust
println!("Next: sdlc ui    # then visit /setup to define Vision and Architecture");
```

This directs first-time users to open the UI and set up their project's Vision and Architecture before creating features — the correct first step for a new project.

## Scope

- **Files:** `README.md` and `crates/sdlc-cli/src/cmd/init/mod.rs`
- **Changes:** 1 new README section (~8 lines) + 1 line change in init.rs completion message
- **Existing users:** No impact — additive documentation; init message change is informational only
- **Tests:** No test changes needed
