use crate::cmd::init::registry::CommandDef;

const SDLC_TOOL_BUILD_COMMAND: &str = r#"---
description: Scaffold and implement a new SDLC tool
argument-hint: <name> "<description>"
allowed-tools: Bash, Read, Write, Edit
---

# sdlc-tool-build

Scaffold and implement a new SDLC tool end-to-end.

> **Before acting:** read `.sdlc/guidance.md` — especially §7 "SDLC Tool Suite". <!-- sdlc:guidance -->

## Steps

### 1. Read an existing tool for reference

Read a tool with multiple input fields so you understand the full schema pattern:

```bash
cat .sdlc/tools/cloudflare-dns/tool.ts   # if present — shows enum, boolean, number, required
cat .sdlc/tools/ama/tool.ts              # fallback minimal reference
```

### 2. Scaffold the new tool

Use `$ARGUMENTS` as `<name> "<description>"`:

```bash
sdlc tool scaffold <name> "<description>"
```

This creates `.sdlc/tools/<name>/tool.ts`, `config.yaml`, and `README.md`.

### 3. Design the input schema

**This step drives the UI form.** Every property in `input_schema.properties` becomes a typed form field in the dashboard — no extra work needed. Design it deliberately:

```typescript
input_schema: {
  type: 'object',
  required: ['action'],          // required fields are marked * in the form
  properties: {
    action: {
      type: 'string',
      enum: ['list', 'create', 'delete'],  // → rendered as <select>
      description: 'Operation to perform',  // shown inline in the form
    },
    name: {
      type: 'string',
      description: 'Resource name',         // → rendered as text input
    },
    count: {
      type: 'number',
      description: 'How many to process',   // → rendered as number input
    },
    dry_run: {
      type: 'boolean',
      description: 'Preview without writing', // → rendered as checkbox
    },
  },
},
```

Rules:
- Always add `description` on every property — it appears inline in the form
- Put `required` fields first in `properties`
- Only `required` what the tool cannot default — leave optional fields out of `required`
- Enum fields auto-select their first option for required fields

### 4. Fill in the metadata

Open `tool.ts` and update the `meta` object:
- `name` — matches directory name exactly
- `display_name` — human-readable title
- `description` — one sentence, present tense, no trailing period
- `version` — semver (start at `"0.1.0"`)
- `requires_setup` — `true` only if the tool needs one-time initialization
- `input_schema` — from step 3
- `output_schema` — JSON Schema for the `data` field in the result

**If the tool needs credentials**, declare them so the server can block runs when they're missing:

```typescript
secrets: [
  { env_var: 'MY_API_TOKEN', description: 'API token', required: true },
],
```

The server injects matching env vars into the subprocess and returns 422 with `missing_secrets` if required ones are absent.

**Optional power features** (add only when needed):
- `result_actions` — follow-up agent-driven actions shown after a run (e.g., Fix, Roll Back)
- `tags` — grouping labels for the tools list
- `timeout_seconds` — default is 60; increase for long-running operations
- `streaming` — set `true` and emit NDJSON events if the tool streams progress

### 5. Use shared modules

The `_shared/` directory provides ready-made building blocks. Import what you need:

| Module | What it provides |
|---|---|
| `_shared/types.ts` | `ToolMeta`, `ToolResult`, `JsonSchema` type definitions |
| `_shared/log.ts` | `makeLogger(name)` — structured stderr logging |
| `_shared/runtime.ts` | `getArgs()`, `readStdin()`, `getEnv()`, `exit()` — cross-runtime compat (Bun/Deno/Node) |
| `_shared/config.ts` | `loadToolConfig(root, name, defaults)` — load flat key:value `config.yaml` |
| `_shared/sdlc.ts` | **Typed state access** — use this instead of shelling out to `sdlc feature list --json` for reads |
| `_shared/agent.ts` | `ensureAgent()` + `runAgent()` — recruit and invoke Claude sub-agents from within streaming tools |

**`_shared/sdlc.ts` — when to use it:**

If your tool needs to read or write `.sdlc/` state (features, milestones, beat evaluations, ponder entries, VISION.md), import from `_shared/sdlc.ts`:

```typescript
import {
  getProjectRoot,
  readVision,
  readFeatures,
  readMilestones,
  readBeat, writeBeat,
  createPonder, appendPonderSession,
} from '../_shared/sdlc.ts'

const root = getProjectRoot()           // resolves SDLC_ROOT or cwd
const features = readFeatures(root)     // typed FeatureSummary[]
const vision = readVision(root)         // string content of VISION.md
const beat = readBeat(root)             // BeatState — evaluations, weekly
writeBeat(root, { ...beat, evaluations: [...beat.evaluations, newEval] })
```

**Do not** shell out to `sdlc feature list --json` or `sdlc milestone list --json` for simple reads — the primitives are faster, typed, and don't require the CLI binary to be on PATH during testing.

`createPonder` and `appendPonderSession` are the only write-path exceptions that still delegate to the CLI (because slug generation and session logging have invariants enforced by the Rust layer).

**`_shared/agent.ts` — recruit-if-missing pattern:**

Streaming tools can invoke a real Claude agent run and await its result. Use this when the tool needs deep reasoning, multi-step analysis, or access to Claude's full capability:

```typescript
// REQUIRES: streaming: true in --meta (prevents deadlock)
import { ensureAgent, runAgent } from '../_shared/agent.ts'

// Recruit-if-missing: creates the agent file on first use, returns existing path on subsequent calls
const agentPath = ensureAgent(root, 'code-reviewer', 'Expert code reviewer who evaluates changes for correctness and style')

// Invoke the agent run (blocks until complete, up to 10 min)
const result = await runAgent({
  prompt: `Review this diff:\n${diff}`,
  agentFile: agentPath,  // agent's persona prepended to prompt
  maxTurns: 15,
})

// Handle graceful error for missing agent file
if (typeof result === 'object' && !result.ok) {
  return { ok: false, error: result.error }
}

// result is a string — the agent's text output
```

The agent run appears in the run history panel in the UI. SDLC_SERVER_URL and SDLC_AGENT_TOKEN are automatically injected by the server — no setup needed in the tool.

### 6. Implement `run()`

- Accept typed input matching `input_schema`
- Return `ToolResult<YourOutputType>`
- Return `{ ok: false, error: "..." }` on **every** error — never throw
- All logging goes to stderr via `makeLogger` — never `console.log()`

### 7. Add `--setup` mode (if needed)

Only add setup if `requires_setup: true`. Skip this step otherwise.

### 8. Write README.md

Update `.sdlc/tools/<name>/README.md` with:
- One-sentence description
- Setup instructions (if `requires_setup: true`)
- Usage examples with exact commands and JSON input
- How it works (1–3 sentences)

### 9. Self-check before testing

Before running the audit, verify these audit requirements yourself:

- [ ] `--meta` exits 0 and outputs valid JSON
- [ ] Every `input_schema` property has a `description`
- [ ] All logging uses `makeLogger` (stderr only — no `console.log`)
- [ ] Every error path returns `{ ok: false, error: "..." }` — no thrown exceptions
- [ ] README has a Usage section with a real command example
- [ ] Any `.sdlc/` file reads use `_shared/sdlc.ts` primitives (not raw `readFileSync` on manifest paths)

### 10. Test `--meta` mode

```bash
bun run .sdlc/tools/<name>/tool.ts --meta | jq .
```

### 11. Test `--run` mode

```bash
echo '{"action": "list"}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok
echo '{"action": "list"}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .
```

### 12. Test via CLI wrapper

```bash
sdlc tool run <name>
sdlc tool run <name> --json '{"action": "list"}'
```

### 13. Sync tools.md

```bash
sdlc tool sync
```

### 14. Commit

Stage and commit the new tool files.

**Next:** `/sdlc-tool-audit <name>`
"#;

const SDLC_TOOL_BUILD_PLAYBOOK: &str = r#"# sdlc-tool-build

Scaffold and implement a new SDLC tool end-to-end.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Steps

1. Read a multi-field reference: `cat .sdlc/tools/cloudflare-dns/tool.ts` (or `ama/tool.ts` as fallback).
2. Scaffold: `sdlc tool scaffold <name> "<description>"`
3. Design `input_schema` — every property drives a UI form field. Use typed properties:
   - `enum: [...]` → select dropdown
   - `type: 'boolean'` → checkbox
   - `type: 'number'` → number input
   - `type: 'string'` → text input
   - Add `description` on every property (shown inline in the form)
   - Use `required: [...]` only for fields with no sensible default
4. Fill `--meta` mode (ToolMeta): name, display_name, description (no period), version, schemas.
   - Add `secrets: [{env_var, description, required}]` if the tool needs credentials.
5. Use shared modules from `_shared/`:
   - `_shared/sdlc.ts` — **use for all `.sdlc/` state reads/writes** (features, milestones, beat, ponder). Import `readFeatures`, `readMilestones`, `readBeat`, `writeBeat`, `readVision`, `createPonder`, `appendPonderSession`. Do NOT shell out to `sdlc feature list --json` for simple reads.
   - `_shared/log.ts` — `makeLogger` for stderr logging
   - `_shared/runtime.ts` — `getArgs`, `readStdin`, `getEnv`, `exit`
   - `_shared/config.ts` — `loadToolConfig` for flat `config.yaml`
   - `_shared/agent.ts` — `ensureAgent()` + `runAgent()` — recruit and invoke Claude sub-agents (streaming tools only; requires `streaming: true` in meta)
   - **Recruit-if-missing pattern** (streaming tools): `ensureAgent(root, slug, role)` creates agent file on first use; `runAgent({ prompt, agentFile, maxTurns })` blocks until the agent completes. Returns `{ ok: false, error }` for agent-file-not-found; throws for all other errors.
6. Implement `--run` mode: read JSON stdin → do work → write ToolResult stdout.
   - All errors: `{ ok: false, error: "..." }` — never throw.
   - All logging via `makeLogger` to stderr only — no `console.log`.
7. Add `--setup` mode only if `requires_setup: true`; skip otherwise.
8. Write `README.md` with Usage (real commands) and Setup sections.
9. Self-check: `--meta` exits 0, all properties have descriptions, no console.log, no throw, `.sdlc/` reads via `_shared/sdlc.ts`.
10. Test: `bun run .sdlc/tools/<name>/tool.ts --meta | jq .` then `echo '{}' | ... --run | jq .ok`
11. Run `/sdlc-tool-audit <name>` then `/sdlc-tool-uat <name>`.
12. `sdlc tool sync` and commit.

**Next:** `/sdlc-tool-audit <name>`
"#;

const SDLC_TOOL_BUILD_SKILL: &str = r#"---
name: sdlc-tool-build
description: Scaffold and implement a new SDLC tool end-to-end. Use when building a new tool from scratch.
---

# SDLC Tool-Build Skill

Scaffold, implement, and ship a new SDLC tool.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Workflow

1. Read `cloudflare-dns/tool.ts` (or `ama/tool.ts`) as the reference implementation.
2. `sdlc tool scaffold <name> "<description>"` to create the skeleton.
3. Design `input_schema` — properties drive the UI form automatically:
   - `enum: [...]` → select, `type: 'boolean'` → checkbox, `type: 'number'` → number input
   - Add `description` on every property (shown inline in the form)
   - `required: [...]` only for fields with no sensible default
4. Fill `--meta` mode (ToolMeta). If credentials needed: `secrets: [{env_var, description, required}]`.
5. Use `_shared/sdlc.ts` for all `.sdlc/` state access — `readFeatures`, `readMilestones`, `readBeat`, `writeBeat`, `readVision`, `createPonder`, `appendPonderSession`. Never shell out to `sdlc feature list --json` for simple reads. Use `_shared/log.ts` (makeLogger), `_shared/runtime.ts` (getArgs/readStdin/getEnv/exit), `_shared/config.ts` (loadToolConfig). For streaming tools that need deep reasoning: use `_shared/agent.ts` — `ensureAgent(root, slug, role)` + `runAgent({ prompt, agentFile, maxTurns })`. Requires `streaming: true`. Returns `{ ok: false, error }` for agent-file-not-found; throws for all other errors.
6. Implement `--run`: stdin JSON → work → ToolResult stdout. All errors: `{ ok: false, error }`. All logs: `makeLogger` (stderr, never `console.log`).
7. Add `--setup` only if `requires_setup: true`.
8. Write `README.md` with Usage (real commands) + Setup sections.
9. Self-check before audit: `--meta` exits 0, all properties have descriptions, no thrown errors, `.sdlc/` reads via `_shared/sdlc.ts`.
10. Test `--meta | jq .` then `echo '{}' | ... --run | jq .ok`.
11. Audit: `/sdlc-tool-audit <name>` then UAT: `/sdlc-tool-uat <name>`.
12. `sdlc tool sync` and commit.
"#;

pub static SDLC_TOOL_BUILD: CommandDef = CommandDef {
    slug: "sdlc-tool-build",
    claude_content: SDLC_TOOL_BUILD_COMMAND,
    gemini_description: "Scaffold and implement a new SDLC tool",
    playbook: SDLC_TOOL_BUILD_PLAYBOOK,
    opencode_description: "Scaffold and implement a new SDLC tool",
    opencode_hint: r#"<name> "<description>""#,
    skill: SDLC_TOOL_BUILD_SKILL,
};
