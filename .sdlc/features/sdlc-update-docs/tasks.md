# Tasks: Document sdlc update as Update Mechanism

## T1 — Add "Updating" section to README.md

**File:** `README.md`

Add an "Updating" section immediately after the "Install" section. Content per spec:

```markdown
## Updating

To upgrade the sdlc binary, re-run your install command (or `brew upgrade sdlc` if installed via Homebrew).

After upgrading the binary, run:

```bash
sdlc update
```

This refreshes your AI command scaffolding — the `/sdlc-*` slash commands installed in `~/.claude/commands/`, `~/.gemini/commands/`, etc. Run this after every sdlc binary upgrade to keep your AI tools in sync.
```

**Acceptance:** README.md contains an `## Updating` section with the `sdlc update` invocation and explanation.

---

## T2 — Fix `sdlc init` completion message

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Find the line that prints the "Next:" hint at the end of `sdlc init` and change it from:

```
Next: sdlc feature create <slug> --title "..."
```

to:

```
Next: sdlc ui    # then visit /setup to define Vision and Architecture
```

**Acceptance:** Running `sdlc init` (or inspecting the source) shows the new message directing users to `sdlc ui` and `/setup`.
