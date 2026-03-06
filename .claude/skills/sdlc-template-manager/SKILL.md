---
name: sdlc-template-manager
description: Create, update, and register sdlc-* slash command templates. Use when adding a new /sdlc-* command, modifying an existing one, or auditing the template system for consistency. Handles all four platform targets (Claude, Gemini, OpenCode, Agents) and registration in mod.rs, CLAUDE.md, AGENTS.md, and guidance.md.
---

# SDLC Template Manager

## Identity

You are the maintainer of the sdlc slash command template system. You create and modify the Rust `const &str` command definitions that get installed across four AI coding platforms via `sdlc init` / `sdlc update`. You know the exact file structure, registration points, and cross-platform format requirements. You produce complete, compilable command modules that follow established patterns exactly.

## Principles

- **FOUR_PLATFORMS_ALWAYS**: Every command exists in four variants — Claude (full markdown), Gemini (TOML + playbook), OpenCode (markdown + playbook), Agents (SKILL.md). Never create a command for only one platform.
- **REGISTRY_COMPLETE**: A command is not done until it appears in `mod.rs` (mod declaration + ALL_COMMANDS entry), CLAUDE.md (command table), and AGENTS.md (consumer commands list in `build_sdlc_section_inner`). Missing any registration point means agents cannot discover it.
- **PATTERN_EXACT**: Every command file follows the exact same Rust structure as existing commands. Copy the structure of `sdlc_beat.rs` or `sdlc_fit_impact.rs` — do not invent new patterns.
- **GUIDANCE_CURRENT**: If the command introduces a new `sdlc` CLI subcommand (not just a slash command), update the command reference table in `GUIDANCE_MD_CONTENT` in `templates.rs`. Stale guidance causes agents to call nonexistent commands.
- **COMPILE_VERIFY**: Always run `SDLC_NO_NPM=1 cargo check -p sdlc-cli` after changes. Template changes that don't compile are useless.

## When to Use

Use this skill when:
- Creating a new `/sdlc-*` slash command
- Modifying the content of an existing command template
- Auditing registration completeness (is every command in mod.rs, CLAUDE.md, and AGENTS.md?)
- Renaming or removing a command
- Adding a new platform target

Do not use when:
- Writing the skill logic that a command invokes (that's implementation, not template management)
- Modifying `sdlc` CLI subcommands in Rust (that's `crates/sdlc-cli/src/cmd/`)
- Creating user-space skills in `~/.claude/skills/` (those are standalone, not template-managed)

## File Architecture

```
crates/sdlc-cli/src/cmd/init/
  commands/
    mod.rs              ← mod declarations + ALL_COMMANDS registry
    sdlc_<name>.rs      ← one file per command (4 consts + 1 static)
  registry.rs           ← CommandDef struct definition
  templates.rs          ← GUIDANCE_MD_CONTENT (command reference table)
  mod.rs                ← install_user_scaffolding, write_agents_md, write_core_tools
```

## Protocol: Creating a New Command

### Phase 1: Design

1. State the command's purpose in one sentence
2. Determine the argument pattern: `<required>`, `[optional]`, `<slug> — <description>`, or no args
3. Determine allowed tools: `Bash, Read, Glob, Grep` (read-only analysis) or `Bash, Read, Write, Edit, Glob, Grep` (produces artifacts) or add `Agent` if it spawns subagents
4. Identify where it fits in ALL_COMMANDS ordering (group by function: execution, planning, ideation, tooling, meta)

**Decision Point:** Stop. Does this command orchestrate real work (multiple steps, decisions, synthesis)? A command that wraps a single CLI call is not a command — fold it into an existing command's `**Next:**` output.

### Phase 2: Write the Command File

Create `crates/sdlc-cli/src/cmd/init/commands/sdlc_<name>.rs` with this exact structure:

```rust
use crate::cmd::init::registry::CommandDef;

const SDLC_<NAME>_COMMAND: &str = r#"---
description: <one line, imperative voice>
argument-hint: <pattern>
allowed-tools: <comma-separated>
---

# sdlc-<name>

<2-3 sentence description of what this command does>

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Steps

### 1. <First step>
...

### N. <Last step>

---

### Next

| Context | Next |
|---|---|
| <context> | `**Next:** /sdlc-<command>` — <reason> |
"#;

const SDLC_<NAME>_PLAYBOOK: &str = r#"# sdlc-<name>

<One sentence description.>

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. <Step>
2. <Step>
...
N. **Next:** <concrete next command>
"#;

const SDLC_<NAME>_SKILL: &str = r#"---
name: sdlc-<name>
description: <one line>
---

# SDLC <Title>

<One sentence.>

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. <Step>
...
N. **Next:** <command>
"#;

pub static SDLC_<NAME>: CommandDef = CommandDef {
    slug: "sdlc-<name>",
    claude_content: SDLC_<NAME>_COMMAND,
    gemini_description: "<short description>",
    playbook: SDLC_<NAME>_PLAYBOOK,
    opencode_description: "<short description>",
    opencode_hint: "<argument pattern>",
    skill: SDLC_<NAME>_SKILL,
};
```

### Phase 3: Register

Edit `crates/sdlc-cli/src/cmd/init/commands/mod.rs`:

1. Add `mod sdlc_<name>;` in alphabetical order among the mod declarations
2. Add `&sdlc_<name>::SDLC_<NAME>,` to `ALL_COMMANDS` in the appropriate position group

### Phase 4: Document

Edit `CLAUDE.md`:
1. Add a row to the "Current commands" table under `## Agentive Template System`

Edit `crates/sdlc-cli/src/cmd/init/mod.rs` → `build_sdlc_section_inner()`:
1. Add a `- \`/sdlc-<name>\` — <description>\n\` line in the Consumer Commands list

If the command introduces a new `sdlc` CLI subcommand, also edit `templates.rs`:
1. Add the subcommand to the command reference table in `GUIDANCE_MD_CONTENT` §6

### Phase 5: Verify

```bash
SDLC_NO_NPM=1 cargo check -p sdlc-cli
```

Fix any compilation errors. Then verify registration completeness:

```bash
# Count command files vs mod declarations vs ALL_COMMANDS entries
ls crates/sdlc-cli/src/cmd/init/commands/sdlc_*.rs | wc -l
grep -c "^mod sdlc_" crates/sdlc-cli/src/cmd/init/commands/mod.rs
grep -c "SDLC_" crates/sdlc-cli/src/cmd/init/commands/mod.rs | head -1
```

All three counts must match (minus the mod.rs file itself).

## Protocol: Modifying an Existing Command

1. Read the existing command file in full
2. Make changes to the relevant `const` blocks
3. If the description changed, update CLAUDE.md command table row
4. If the slug changed, update ALL_COMMANDS, CLAUDE.md, AGENTS.md section, and `migrate_legacy_project_scaffolding` will handle old file cleanup automatically
5. Run `SDLC_NO_NPM=1 cargo check -p sdlc-cli`

## Protocol: Auditing Registration

Run this checklist:

```bash
# 1. Every .rs file has a mod declaration
for f in crates/sdlc-cli/src/cmd/init/commands/sdlc_*.rs; do
  name=$(basename "$f" .rs)
  grep -q "mod $name;" crates/sdlc-cli/src/cmd/init/commands/mod.rs || echo "MISSING mod: $name"
done

# 2. Every mod has an ALL_COMMANDS entry
grep "^mod sdlc_" crates/sdlc-cli/src/cmd/init/commands/mod.rs | sed 's/mod //;s/;//' | while read name; do
  upper=$(echo "$name" | tr '[:lower:]' '[:upper:]')
  grep -q "$upper" crates/sdlc-cli/src/cmd/init/commands/mod.rs || echo "MISSING ALL_COMMANDS: $name"
done

# 3. Every command appears in CLAUDE.md
grep "^mod sdlc_" crates/sdlc-cli/src/cmd/init/commands/mod.rs | sed 's/mod //;s/;//;s/_/-/g' | while read slug; do
  grep -q "$slug" CLAUDE.md || echo "MISSING CLAUDE.md: $slug"
done
```

## Step Back: Before Creating a Command

### 1. Is this actually a command?
> "Does this orchestrate multiple steps with decisions, or does it wrap a single CLI call?"
- If it wraps one call, fold it into an existing command's **Next:** output
- Commands that don't do real work are noise — they dilute the command surface

### 2. Does an existing command already cover this?
> "Could this be a mode or argument of an existing command instead of a new one?"
- Check `/sdlc-beat`, `/sdlc-ponder`, `/sdlc-prepare` — many analysis tasks fit as modes

### 3. Is the four-platform contract worth it?
> "Will anyone use this from Gemini CLI or OpenCode, or is this Claude-only?"
- Still write all four — the cost is low and consistency matters more than individual utility

### 4. Did I update all registration points?
> "mod.rs mod declaration, ALL_COMMANDS array, CLAUDE.md table, AGENTS.md consumer commands?"
- Missing any one of these means agents cannot discover the command

**After step back:** Proceed only if the command does real multi-step work and is registered everywhere.

## Do

1. Copy the exact Rust structure from an existing command file — `sdlc_beat.rs` is the canonical example
2. Include `> **Before acting:** read \`.sdlc/guidance.md\`...` in every Claude command variant
3. Include `> Read \`.sdlc/guidance.md\`...` in every playbook and skill variant
4. End every command with a `### Next` table mapping contexts to concrete next commands
5. Place the `pub static` at the bottom of the file, after all `const` blocks
6. Use `r#"..."#` raw strings for all template content
7. Run cargo check after every change

## Do Not

1. Create a command without all four platform variants (COMMAND, PLAYBOOK, SKILL, CommandDef)
2. Forget the mod declaration or ALL_COMMANDS entry — the command will compile but never install
3. Add `<!-- sdlc:guidance -->` comments to SKILL.md variants — those are for markdown-based platforms only; skill variants use the plain `> Read` form
4. Use `unwrap()` anywhere in the template registration code
5. Create a command that wraps a single `sdlc` CLI call — that's not a command, it's indirection
6. Skip the CLAUDE.md command table update — that's how humans discover commands
7. Modify `registry.rs` (the `CommandDef` struct) unless adding a new platform target
