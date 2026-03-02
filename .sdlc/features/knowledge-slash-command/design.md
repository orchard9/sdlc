# Design: /sdlc-knowledge Slash Command

## Overview

This is a pure additive change to the init system. No new Rust structs, no new CLI subcommands, no server routes. The work is entirely in:

1. A new command file following the established pattern
2. Registration in the command registry
3. A guidance.md section 6 update

## Architecture

The command follows the exact same pattern as every other command in `crates/sdlc-cli/src/cmd/init/commands/`:

```
sdlc_knowledge.rs
  ├── SDLC_KNOWLEDGE_COMMAND   — Claude Code markdown (frontmatter + playbook)
  ├── SDLC_KNOWLEDGE_PLAYBOOK  — shared Gemini + OpenCode body
  ├── SDLC_KNOWLEDGE_SKILL     — minimal agents SKILL.md
  └── pub static SDLC_KNOWLEDGE: CommandDef { ... }
```

The `CommandDef` struct (in `registry.rs`) has these fields:
- `slug`: `"sdlc-knowledge"`
- `claude_content`: the full Claude Code markdown with frontmatter
- `gemini_description`: short one-liner for TOML `description`
- `playbook`: shared Gemini + OpenCode body text
- `opencode_description`: short one-liner for OpenCode frontmatter
- `opencode_hint`: `"[<topic> | init | research <topic> | maintain]"`
- `skill`: the SKILL.md content

## Command Content Design

### Claude Code command (`SDLC_KNOWLEDGE_COMMAND`)

Frontmatter:
```
description: Query and manage the project knowledge base — catalog overview, topic synthesis, init, research, and maintenance
argument-hint: [<topic> | init | research <topic> | maintain]
allowed-tools: Bash, Read, Write, Glob, Grep, WebSearch, WebFetch
```

Playbook sections:
1. **Dispatch table** — routes `$ARGUMENTS` to one of five modes
2. **Mode: no argument** — run `sdlc knowledge status`, `sdlc knowledge catalog show`, `sdlc knowledge list` to give a catalog overview
3. **Mode: `<topic>` query** — run `sdlc knowledge search <topic>`, show matching entries with `sdlc knowledge show <slug>`, synthesize answer with citations
4. **Mode: `init`** — run `sdlc knowledge librarian init`, report counts
5. **Mode: `research <topic>`** — search first, then use Grep/Read/WebSearch to find new knowledge, add entries via `sdlc knowledge add`, confirm with a second search
6. **Mode: `maintain`** — check status, list entries, scan for stale or orphaned entries, report findings
7. **Next step** — always end with `**Next:** /sdlc-knowledge <topic>` or `/sdlc-ponder`

### Gemini/OpenCode playbook (`SDLC_KNOWLEDGE_PLAYBOOK`)

Concise numbered list. Same five modes but condensed to bullet points.

### Agents skill (`SDLC_KNOWLEDGE_SKILL`)

Minimal SKILL.md with name/description frontmatter and a brief workflow list.

## Registration

`commands/mod.rs`:
- Add `mod sdlc_knowledge;`
- Add `&sdlc_knowledge::SDLC_KNOWLEDGE` to `ALL_COMMANDS` slice — insert after `&sdlc_guideline::SDLC_GUIDELINE` (knowledge is in the workspace/investigation family)

## GUIDANCE_MD_CONTENT Update

Add to section 6 table (after the existing `sdlc escalate resolve` row, before the closing paragraph):

| Action | Command |
|---|---|
| Knowledge base status | `sdlc knowledge status` |
| List knowledge entries | `sdlc knowledge list [--code-prefix <code>]` |
| Search knowledge base | `sdlc knowledge search <query>` |
| Show knowledge entry | `sdlc knowledge show <slug>` |
| Add knowledge entry | `sdlc knowledge add --title "..." --code <code> --content "..."` |
| Show catalog taxonomy | `sdlc knowledge catalog show` |
| Seed from workspaces | `sdlc knowledge librarian init` |

## No Breaking Changes

- All existing tests pass untouched
- `ALL_COMMANDS` grows by one entry — `migrate_legacy_project_scaffolding` and all `write_user_*` functions derive their file lists from `ALL_COMMANDS`, so they automatically handle the new command
- Section 6 of guidance.md is always overwritten by `sdlc init`/`sdlc update` — safe to extend
