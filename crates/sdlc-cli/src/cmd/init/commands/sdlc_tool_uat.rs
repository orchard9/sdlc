use crate::cmd::init::registry::CommandDef;

const SDLC_TOOL_UAT_COMMAND: &str = r#"---
description: Run UAT scenarios for an SDLC tool
argument-hint: <tool-name>
allowed-tools: Bash
---

# sdlc-tool-uat

Run UAT scenarios for an SDLC tool.

> **Before acting:** read `.sdlc/guidance.md` — especially §7 "SDLC Tool Suite". <!-- sdlc:guidance -->

Use `$ARGUMENTS` as `<tool-name>`.

## Scenarios

Run each scenario and record the verdict (PASS / FAIL / SKIP).

### 1. Metadata

```bash
bun run .sdlc/tools/<name>/tool.ts --meta | jq .
```

Verify: `name`, `display_name`, `description`, `version`, `input_schema`, `output_schema` all present.

### 2. Happy path

```bash
echo '{"question":"test"}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok
```

Expected: `true`

### 3. Empty input (optional fields)

```bash
echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok
```

Expected: `true` — tools must handle missing optional inputs gracefully.

### 4. CLI wrapper

```bash
sdlc tool run <name> --question "test"
```

Expected: JSON output on stdout, exit 0.

### 5. Error path

Supply invalid or missing required input and verify the tool returns an error result:

```bash
echo '{"invalid_key": true}' | bun run .sdlc/tools/<name>/tool.ts --run | jq '{ok, error}'
```

Expected: `{ "ok": false, "error": "..." }` (not a crash).

### 6. Logging format

```bash
echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run 2>&1 >/dev/null | head -5
```

Expected: lines match `[sdlc-tool:<name>] (INFO|WARN|ERROR|DEBUG):`.

### 7. Discovery

```bash
sdlc tool list
```

Expected: `<name>` appears in the output.

**Next:** `sdlc tool sync` to regenerate `tools.md`
"#;

const SDLC_TOOL_UAT_PLAYBOOK: &str = r#"# sdlc-tool-uat

Run UAT scenarios for an SDLC tool. Record PASS / FAIL / SKIP for each.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Scenarios

Use `$ARGUMENTS` as `<name>`.

### 1. Metadata
`bun run .sdlc/tools/<name>/tool.ts --meta | jq .`
Verify: `name`, `display_name`, `description`, `version`, `input_schema`, `output_schema` all present.

### 2. Happy path
`echo '{"question":"test"}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok`
Expected: `true`

### 3. Empty input (optional fields)
`echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok`
Expected: `true` — tools must handle missing optional inputs gracefully.

### 4. CLI wrapper
`sdlc tool run <name> --question "test"`
Expected: JSON output on stdout, exit 0.

### 5. Error path
`echo '{"invalid_key": true}' | bun run .sdlc/tools/<name>/tool.ts --run | jq '{ok, error}'`
Expected: `{ "ok": false, "error": "..." }` (not a crash).

### 6. Logging format
`echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run 2>&1 >/dev/null | head -5`
Expected: lines match `[sdlc-tool:<name>] (INFO|WARN|ERROR|DEBUG):`.

### 7. Discovery
`sdlc tool list`
Expected: `<name>` appears in the output.

**Next:** `sdlc tool sync` to regenerate `tools.md`
"#;

const SDLC_TOOL_UAT_SKILL: &str = r#"---
name: sdlc-tool-uat
description: Run 7 UAT scenarios for an SDLC tool and record PASS/FAIL/SKIP. Use when validating a tool before shipping.
---

# SDLC Tool-UAT Skill

Run 7 UAT scenarios for an SDLC tool.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Workflow

Record PASS / FAIL / SKIP for each scenario:
1. `--meta` — all required fields present
2. Happy path `--run` — `ok: true`
3. Empty input — `ok: true` (optional fields handled)
4. CLI wrapper `sdlc tool run` — JSON out, exit 0
5. Error path — `ok: false` with error message (no crash)
6. Logging format — lines match `[sdlc-tool:<name>] LEVEL:`
7. Discovery — `sdlc tool list` shows the tool

End: `**Next:** sdlc tool sync` if all pass.
"#;

pub static SDLC_TOOL_UAT: CommandDef = CommandDef {
    slug: "sdlc-tool-uat",
    claude_content: SDLC_TOOL_UAT_COMMAND,
    gemini_description: "Run UAT scenarios for an SDLC tool",
    playbook: SDLC_TOOL_UAT_PLAYBOOK,
    opencode_description: "Run UAT scenarios for an SDLC tool",
    opencode_hint: "<tool-name>",
    skill: SDLC_TOOL_UAT_SKILL,
};
