use crate::cmd::init::registry::CommandDef;

const SDLC_TOOL_RUN_COMMAND: &str = r#"---
description: Run an SDLC tool and act on the JSON result
argument-hint: <tool-name> [args]
allowed-tools: Bash, Read
---

# sdlc-tool-run

Run an installed SDLC tool and act on the JSON result.

> **Before acting:** read `.sdlc/guidance.md` — especially §7 "SDLC Tool Suite". <!-- sdlc:guidance -->

## Steps

### 1. Check available tools

```bash
cat .sdlc/tools/tools.md
```

### 2. Run the tool

Use `$ARGUMENTS` as `<tool-name>` (and any extra args):

```bash
# Simple question
sdlc tool run <name> --question "..."

# Scoped run
sdlc tool run <name> --scope "..."

# Complex JSON input
sdlc tool run <name> --json '{"key": "val"}'
```

### 3. Parse and act on the result

The tool outputs `{ ok, data, error?, duration_ms }`.

- If `ok: false` — explain the error and suggest a fix
- If `ok: true` — describe the findings and recommend next steps based on the data

**Next:** `/sdlc-tool-audit <name>` if the tool output reveals quality issues
"#;

const SDLC_TOOL_RUN_PLAYBOOK: &str = r#"# sdlc-tool-run

Run an installed SDLC tool and act on its JSON result.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Steps

1. Check available tools: `cat .sdlc/tools/tools.md`
2. Run the tool using `$ARGUMENTS` as `<tool-name>` (plus any extra args):
   - Simple question: `sdlc tool run <name> --question "..."`
   - Scoped run: `sdlc tool run <name> --scope "..."`
   - Complex input: `sdlc tool run <name> --input '{"key":"val"}'`
3. Parse the JSON result `{ ok, data, error?, duration_ms }`.
4. If `ok: false` — explain the error and suggest a fix.
   If `ok: true` — describe the findings and recommend next steps.

**Next:** `/sdlc-tool-audit <name>` if the tool output reveals quality issues
"#;

const SDLC_TOOL_RUN_SKILL: &str = r#"---
name: sdlc-tool-run
description: Run an installed SDLC tool and act on its JSON result. Use when an agent needs to invoke a tool and interpret the output.
---

# SDLC Tool-Run Skill

Run an SDLC tool and act on the result.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Workflow

1. Check available tools: `cat .sdlc/tools/tools.md`
2. Run: `sdlc tool run <name> --question "..."` (or `--scope` / `--input` for complex input).
3. Parse `{ ok, data, error?, duration_ms }`.
4. `ok: false` → explain error. `ok: true` → act on findings.
5. End: `**Next:** /sdlc-tool-audit <name>` if issues found.
"#;

pub static SDLC_TOOL_RUN: CommandDef = CommandDef {
    slug: "sdlc-tool-run",
    claude_content: SDLC_TOOL_RUN_COMMAND,
    gemini_description: "Run an SDLC tool and act on the JSON result",
    playbook: SDLC_TOOL_RUN_PLAYBOOK,
    opencode_description: "Run an SDLC tool and act on the JSON result",
    opencode_hint: "<tool-name> [args]",
    skill: SDLC_TOOL_RUN_SKILL,
};
