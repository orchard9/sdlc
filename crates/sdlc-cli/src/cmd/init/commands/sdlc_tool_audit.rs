use crate::cmd::init::registry::CommandDef;

const SDLC_TOOL_AUDIT_COMMAND: &str = r#"---
description: Audit an SDLC tool against the full quality contract
argument-hint: <tool-name>
allowed-tools: Bash, Read
---

# sdlc-tool-audit

Audit an SDLC tool against the full quality contract.

> **Before acting:** read `.sdlc/guidance.md` — especially §7 "SDLC Tool Suite". <!-- sdlc:guidance -->

Use `$ARGUMENTS` as `<tool-name>`.

## Checklist

Read `.sdlc/tools/<name>/tool.ts` and verify each item. Mark ✓ or ✗.

### Metadata (5 checks)

- [ ] `name` matches the directory name exactly (e.g. `"quality-check"` in `quality-check/`)
- [ ] `display_name` is human-readable and title-cased
- [ ] `description` is one sentence, present tense, no trailing period
- [ ] `version` is valid semver (e.g. `"0.1.0"`)
- [ ] `input_schema` and `output_schema` are defined

### Protocol (4 checks)

- [ ] `--meta` mode: exits 0 and outputs valid `ToolMeta` JSON
- [ ] `--run` mode: reads JSON from stdin before doing any work
- [ ] `--run` mode: exits 1 when `ok: false`
- [ ] `--setup` mode: handled gracefully (or explicitly absent with a comment)

### Error handling (4 checks)

- [ ] Errors return `{ ok: false, error: "..." }` — never throw unhandled exceptions
- [ ] All `catch` branches log the error and return an error result
- [ ] No bare `process.exit()` calls in library functions (only in the CLI entrypoint)
- [ ] All log output goes to stderr, not stdout

### Logging (2 checks)

- [ ] Uses `makeLogger` from `_shared/log.ts`
- [ ] No `console.log()` calls for logs (only `console.error()` via logger)

### Documentation (3 checks)

- [ ] `README.md` exists and has Usage section
- [ ] `README.md` has Setup section (or "Setup required: No" note)
- [ ] Instruction header in `tool.ts` has WHAT IT DOES, WHAT IT READS, WHAT IT WRITES, EXTENDING

## Commands

```bash
# Read the tool
cat .sdlc/tools/<name>/tool.ts

# Test --meta mode
bun run .sdlc/tools/<name>/tool.ts --meta | jq .

# Verify exit code
echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run; echo "exit: $?"
```

**Next:** `/sdlc-tool-uat <name>` after all checks pass
"#;

const SDLC_TOOL_AUDIT_PLAYBOOK: &str = r#"# sdlc-tool-audit

Audit an SDLC tool against the full quality contract (18-item checklist).

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Checklist

Read `.sdlc/tools/<name>/tool.ts` and mark ✓ or ✗ for each item.

### Metadata (5)
- [ ] `name` matches the directory name exactly
- [ ] `display_name` is human-readable and title-cased
- [ ] `description` is one sentence, present tense, no trailing period
- [ ] `version` is valid semver (e.g. `"0.1.0"`)
- [ ] `input_schema` and `output_schema` are defined

### Protocol (4)
- [ ] `--meta` mode: exits 0 and outputs valid ToolMeta JSON
- [ ] `--run` mode: reads JSON from stdin before doing any work
- [ ] `--run` mode: exits 1 when `ok: false`
- [ ] `--setup` mode: handled gracefully (or explicitly absent with a comment)

### Error handling (4)
- [ ] Errors return `{ ok: false, error: "..." }` — never throw unhandled exceptions
- [ ] All `catch` branches log the error and return an error result
- [ ] No bare `process.exit()` calls in library functions (only in CLI entrypoint)
- [ ] All log output goes to stderr, not stdout

### Logging (2)
- [ ] Uses `makeLogger` from `_shared/log.ts`
- [ ] No `console.log()` calls for logs (only `console.error()` via logger)

### Documentation (3)
- [ ] `README.md` exists and has Usage section
- [ ] `README.md` has Setup section (or "Setup required: No" note)
- [ ] Instruction header in `tool.ts` has WHAT IT DOES, WHAT IT READS, WHAT IT WRITES, EXTENDING

**Next:** `/sdlc-tool-uat <name>` after all 18 checks pass
"#;

const SDLC_TOOL_AUDIT_SKILL: &str = r#"---
name: sdlc-tool-audit
description: Audit an SDLC tool against the full quality contract (18-item checklist). Use when verifying tool correctness before shipping.
---

# SDLC Tool-Audit Skill

Audit an SDLC tool against 18 quality checks in 5 categories.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Workflow

1. Read `.sdlc/tools/<name>/tool.ts`.
2. Check all 18 items: Metadata (5), Protocol (4), Error handling (4), Logging (2), Documentation (3).
3. Mark ✓/✗ for each. Report failing items with suggested fixes.
4. End: `**Next:** /sdlc-tool-uat <name>` when all pass.
"#;

pub static SDLC_TOOL_AUDIT: CommandDef = CommandDef {
    slug: "sdlc-tool-audit",
    claude_content: SDLC_TOOL_AUDIT_COMMAND,
    gemini_description: "Audit an SDLC tool against the full quality contract",
    playbook: SDLC_TOOL_AUDIT_PLAYBOOK,
    opencode_description: "Audit an SDLC tool against the full quality contract",
    opencode_hint: "<tool-name>",
    skill: SDLC_TOOL_AUDIT_SKILL,
};
