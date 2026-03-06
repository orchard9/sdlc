# Design: sdlc-recap-command

## Overview

The `/sdlc-recap` command is a new slash command installed via `sdlc init` / `sdlc update`. It is a pure template — no Rust code changes needed. The implementation is a `CommandDef` struct registered in `ALL_COMMANDS` in `commands/mod.rs`.

## File Structure

```
crates/sdlc-cli/src/cmd/init/commands/
  sdlc_recap.rs          ← NEW — CommandDef with 4 platform variants
  mod.rs                 ← MODIFIED — add mod + entry in ALL_COMMANDS
crates/sdlc-cli/src/cmd/init/
  mod.rs                 ← MODIFIED — add /sdlc-recap to AGENTS.md consumer commands
  templates.rs           ← MODIFIED — add sdlc-recap to GUIDANCE_MD_CONTENT command table
```

## CommandDef Structure

```rust
pub static SDLC_RECAP: CommandDef = CommandDef {
    slug: "sdlc-recap",
    claude_content: SDLC_RECAP_COMMAND,      // full Claude Code slash command
    gemini_description: "...",
    playbook: SDLC_RECAP_PLAYBOOK,           // shared Gemini + OpenCode body
    opencode_description: "...",
    opencode_hint: "[feature-slug | milestone-slug]",
    skill: SDLC_RECAP_SKILL,                 // Agent Skills SKILL.md
};
```

## Claude Command Design (SDLC_RECAP_COMMAND)

The Claude variant is the full command spec. Structure:

```
---
description: Produce a state-aware session recap with forward motion — synthesize progress, classify remaining work, and create sdlc artifacts
argument-hint: [feature-slug | milestone-slug]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-recap

[ethos + before-acting note]

## Step 1 — Gather state
  - sdlc status --json
  - sdlc milestone info <active-milestone> --json (if applicable)
  - git log --oneline -20
  - git diff --stat HEAD~5

## Step 2 — Synthesize
  ### Working On
  ### Completed
  ### Remaining (with classifier: Fixable | Needs input | Complex)

## Step 3 — Create forward artifacts
  For Complex → sdlc ponder create
  For Needs input → sdlc task add with [escalate] marker
  For Fixable → sdlc task add

## Step 4 — Commit completed work
  git add + git commit -m "session: <summary>"

## Step 5 — Output
  Exactly one **Next:** line
```

## Command Logic (Forward Motion Rules)

```
for each Remaining item:
  classify as:
    Fixable    → concrete, bounded, can be done in 1-2 sessions
    Needs input → requires human decision or external info
    Complex    → needs design thinking, multi-session, uncertain scope

  action:
    Fixable    → sdlc task add <feature-slug> "<description>"
    Needs input → sdlc task add <feature-slug> "[escalate] <description>"
    Complex    → sdlc ponder create "<problem-as-question>" --brief "<context>"
```

## Next-line Rules

```
if ponder sessions created → **Next:** /sdlc-ponder <first-ponder-slug>
elif tasks created         → **Next:** /sdlc-run <first-task-feature>
elif clean session         → **Next:** /sdlc-status
```

## Playbook (Gemini + OpenCode variant)

Compact 6-step version of the above — same steps, no detailed code examples. Used for `SDLC_RECAP_PLAYBOOK`.

## Skill (Agent Skills variant)

Minimal SKILL.md with frontmatter (name, description) and condensed 5-bullet workflow.

## AGENTS.md Addition

In `build_sdlc_section_inner()` in `mod.rs`, add to the Consumer Commands list:

```
- `/sdlc-recap [slug]` — state-aware session recap with forward motion — synthesizes progress, classifies remaining work, and creates tasks/ponder entries
```

## GUIDANCE_MD_CONTENT Addition

In `templates.rs`, add to the command table in `GUIDANCE_MD_CONTENT`:

```
| `/sdlc-recap [slug]` | Session recap with forward motion — synthesizes progress, classifies remaining work, creates tasks/ponder entries |
```

## No Breaking Changes

- No Rust types changed
- No CLI argument changes
- No server routes added
- Pure addition — existing commands unaffected
- `ALL_COMMANDS` grows by one entry; install and migrate functions handle it automatically via the registry pattern
