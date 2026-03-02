# Spec: /sdlc-knowledge Slash Command

## Summary

Add a `/sdlc-knowledge` slash command across all four AI CLI platforms (Claude Code, Gemini CLI, OpenCode, generic Agents) by creating `SDLC_KNOWLEDGE_COMMAND`, `SDLC_KNOWLEDGE_PLAYBOOK`, and `SDLC_KNOWLEDGE_SKILL` constants in `crates/sdlc-cli/src/cmd/init/commands/sdlc_knowledge.rs`. Register the command in `commands/mod.rs`. Update `GUIDANCE_MD_CONTENT` in `templates.rs` to add `sdlc knowledge *` commands to section 6 CLI reference table.

## Background

The knowledge base (`sdlc knowledge *` CLI commands) is a complete system for managing structured project knowledge. However, agents have no dedicated slash command for interacting with it. The command needs to expose the full lifecycle: catalog overview, query/synthesis, init, research, and maintenance.

## Command Behavior

The command takes an optional argument:

| Invocation | Behavior |
|---|---|
| `/sdlc-knowledge` (no arg) | Show catalog overview — entry count by category, last maintained date, catalog class list |
| `/sdlc-knowledge <topic>` | Query mode — search knowledge base for topic, synthesize answer with citations from matching entries |
| `/sdlc-knowledge init` | Run `sdlc knowledge librarian init` to seed knowledge base from existing workspaces (ponders, investigations, guidelines) |
| `/sdlc-knowledge research <topic>` | Active research — find, synthesize, add new entries, then search again to confirm indexing |
| `/sdlc-knowledge maintain` | Run a maintenance pass: check for stale entries, orphaned entries, catalog gaps, cross-reference integrity |

## CLI Commands Used

The command relies on existing `sdlc knowledge *` subcommands:

```bash
sdlc knowledge status            # overall status (entry count, catalog classes, last maintained)
sdlc knowledge list              # list all entries
sdlc knowledge list --code-prefix <code>  # list by catalog class
sdlc knowledge search <query>    # full-text search
sdlc knowledge show <slug>       # show a full entry with content
sdlc knowledge add --title "..." --code <code> --content "..."  # add entry
sdlc knowledge catalog show      # show catalog taxonomy
sdlc knowledge librarian init    # seed from workspaces
```

## Files to Create or Modify

| File | Action |
|---|---|
| `crates/sdlc-cli/src/cmd/init/commands/sdlc_knowledge.rs` | Create — three constants + CommandDef |
| `crates/sdlc-cli/src/cmd/init/commands/mod.rs` | Add `mod sdlc_knowledge;` and add `&sdlc_knowledge::SDLC_KNOWLEDGE` to `ALL_COMMANDS` |
| `crates/sdlc-cli/src/cmd/init/templates.rs` | Update `GUIDANCE_MD_CONTENT` section 6 to add `sdlc knowledge *` rows |

## Acceptance Criteria

1. `cargo build --all` passes after changes
2. `SDLC_NO_NPM=1 cargo test --all` passes
3. `cargo clippy --all -- -D warnings` passes
4. `sdlc update` (or `sdlc init`) installs `sdlc-knowledge.md` to `~/.claude/commands/`, `sdlc-knowledge.toml` to `~/.gemini/commands/`, `sdlc-knowledge.md` to `~/.opencode/command/`, and `sdlc-knowledge/SKILL.md` to `~/.agents/skills/`
5. Section 6 of `.sdlc/guidance.md` contains rows for `sdlc knowledge status`, `sdlc knowledge list`, `sdlc knowledge search`, `sdlc knowledge show`, `sdlc knowledge add`, `sdlc knowledge catalog show`, and `sdlc knowledge librarian init`
6. Command content correctly describes all five invocation modes
7. Command follows the pattern of adjacent commands (e.g., `sdlc-guideline.rs`, `sdlc-suggest.rs`) — three constants (`COMMAND`, `PLAYBOOK`, `SKILL`) + one `pub static CommandDef`
