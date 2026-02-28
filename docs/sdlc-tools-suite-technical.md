# SDLC Tools Suite — Technical Specification

**Status:** Design
**Date:** 2026-02-27
**References:** `docs/sdlc-tools-suite.md` (product spec), `crates/sdlc-cli/src/cmd/init.rs`, `crates/sdlc-core/src/paths.rs`

---

## Overview

SDLC Tools are lightweight TypeScript scripts installed per-project by `sdlc init`/`sdlc update`. Each tool speaks a simple JSON stdin/stdout protocol, runs with Bun (or Deno/Node fallback), and is exposed three ways: CLI via `sdlc tool run`, REST via `/api/tools/<name>/run`, and MCP-compatible JSON stream via `/api/tools/<name>/mcp`. The installed tool manifest at `.sdlc/tools.md` is the agent-readable help menu — regenerated on every `sdlc init`/`sdlc update`/`sdlc tool sync`.

---

## 1. Filesystem Layout

```
<project-root>/
└── .sdlc/
    ├── tools/
    │   ├── tools.md                  # generated help menu — agents read this
    │   ├── ama/
    │   │   ├── tool.ts               # tool entrypoint (written by sdlc init, owned by sdlc)
    │   │   ├── config.yaml           # user-editable config (written once by sdlc init)
    │   │   └── index/                # persistent index state (gitignored by default)
    │   │       └── chunks.json
    │   └── quality-check/
    │       ├── tool.ts
    │       └── config.yaml
    └── guidance.md                   # updated to include tools pointer
```

`.sdlc/tools/tools.md` is **always overwritten** by `sdlc init`/`sdlc update`/`sdlc tool sync` — it is managed content, not user-editable. The pattern mirrors `.sdlc/guidance.md`.

`.sdlc/tools/*/config.yaml` is written **once** (on first init, not overwritten) — users can edit it.

`.sdlc/tools/*/tool.ts` is **always overwritten** — it is versioned with the sdlc binary like all other managed templates.

---

## 2. Tool Script Contract

Every tool is a TypeScript file (`tool.ts`) that follows this protocol:

### 2a. Invocation Modes

```
# Metadata query (for help menu generation)
bun run .sdlc/tools/ama/tool.ts --meta
# → writes JSON to stdout: { name, description, input_schema, output_schema, version }

# Direct run (CLI and server both use this)
echo '{"question":"where is JWT validation?"}' | bun run .sdlc/tools/ama/tool.ts --run
# → writes JSON to stdout: { ok: true, data: { answer, sources } }
# → on error: { ok: false, error: "message" }

# Index/setup (run once before first use)
bun run .sdlc/tools/ama/tool.ts --setup --root /path/to/project
# → indexes code, returns { ok: true, data: { files_indexed: N } }
```

### 2b. Standard TypeScript Shape

```typescript
// Every tool.ts must export these two symbols (used when imported, not when run standalone):
export const meta: ToolMeta = { ... }
export async function run(input: unknown, ctx: ToolContext): Promise<ToolResult> { ... }

// And handle CLI modes when run as a script:
if (import.meta.main) {          // Bun/Deno idiom for "if run directly"
  const mode = Deno.args[0] ?? Bun.argv[2] ?? '--run'
  if (mode === '--meta') { console.log(JSON.stringify(meta)); Deno.exit(0) }
  if (mode === '--run') {
    const input = JSON.parse(await readStdin())
    const result = await run(input, { root: process.cwd() })
    console.log(JSON.stringify(result))
  }
}
```

### 2c. ToolMeta Schema

```typescript
interface ToolMeta {
  name: string                    // e.g. "ama"
  display_name: string            // e.g. "Ask Me Anything"
  description: string             // one sentence
  version: string                 // semver, mirrors sdlc binary version
  input_schema: JsonSchema        // JSON Schema for input object
  output_schema: JsonSchema       // JSON Schema for output object
  requires_setup: boolean         // true if --setup must be run before first --run
  setup_description?: string      // what setup does (for help menu)
}
```

### 2d. ToolResult Schema

```typescript
interface ToolResult {
  ok: boolean
  data?: unknown       // tool-specific shape, matches output_schema
  error?: string       // present only when ok = false
  duration_ms?: number // optional perf metric
}
```

### 2e. ToolContext

```typescript
interface ToolContext {
  root: string         // absolute path to project root
  config: ToolConfig   // parsed from .sdlc/tools/<name>/config.yaml
}
```

---

## 3. Runtime Detection

The Rust CLI and server detect the available runtime with this priority:

```rust
// crates/sdlc-core/src/tool_runner.rs  (new file)
pub fn detect_runtime() -> Option<&'static str> {
    for rt in &["bun", "deno", "node"] {
        if which::which(rt).is_ok() { return Some(rt) }
    }
    None
}

pub fn runtime_args(runtime: &str, script: &Path, mode: &str) -> Vec<String> {
    match runtime {
        "bun" => vec!["run".into(), script.to_str().unwrap().into(), mode.into()],
        "deno" => vec![
            "run".into(),
            "--allow-read".into(), "--allow-run".into(), "--allow-net".into(),
            "--allow-write".into(),
            script.to_str().unwrap().into(), mode.into()
        ],
        "node" => vec![
            "--input-type=module".into(),   // node doesn't run .ts natively; need tsx shim
            // NOTE: node fallback requires `npx tsx` wrapper — see tool_runner.rs
        ],
        _ => unreachable!(),
    }
}
```

If no runtime is detected, `sdlc tool run` prints a clear error:
```
Error: no supported runtime found. Install bun (https://bun.sh) to use SDLC tools.
```

---

## 4. `tools.md` — Agent-Readable Help Menu

`.sdlc/tools/tools.md` is regenerated by calling `--meta` on each installed tool and formatting the results. This is the **single file agents are told to read** when they need to know what tools are available.

### Format

```markdown
# SDLC Tools

Project-specific tools installed by sdlc. Use `sdlc tool run <name>` to invoke.

Generated: 2026-02-27T10:00:00Z · sdlc 0.3.0

---

## ama — Ask Me Anything

Answer questions about the codebase. Returns relevant file excerpts with line numbers.

**Run:** `sdlc tool run ama --question "<your question>"`
**Setup required:** Yes — run `sdlc tool run ama --setup` once before first use.

**Input:**
```json
{ "question": "string — the question to answer" }
```

**Output:**
```json
{
  "answer": "string — synthesized answer with file references",
  "sources": [{ "path": "string", "lines": [number, number], "excerpt": "string" }]
}
```

---

## quality-check — Dev Quality Check

Run the project's quality gates from .sdlc/config.yaml on demand.

**Run:** `sdlc tool run quality-check`
**Run (scoped):** `sdlc tool run quality-check --scope implement_task`

**Input:**
```json
{ "scope": "string? — gate action name to filter by (optional, all gates if omitted)" }
```

**Output:**
```json
{
  "passed": number,
  "failed": number,
  "checks": [{ "name": "string", "status": "passed|failed", "output": "string", "duration_ms": number }]
}
```

---

## Adding a Custom Tool

Create `.sdlc/tools/<name>/tool.ts` following the tool contract in `docs/sdlc-tools-suite-technical.md`.
Then run `sdlc tool sync` to regenerate this file.
```

---

## 5. Rust Implementation — Exact Files to Change

### 5a. `crates/sdlc-core/src/paths.rs`

Add after the existing `ROADMAP_DIR` constant block:

```rust
pub const TOOLS_DIR: &str = ".sdlc/tools";
pub const TOOLS_MANIFEST: &str = ".sdlc/tools/tools.md";

pub fn tools_dir(root: &Path) -> PathBuf { root.join(TOOLS_DIR) }
pub fn tool_dir(root: &Path, name: &str) -> PathBuf { tools_dir(root).join(name) }
pub fn tool_script(root: &Path, name: &str) -> PathBuf { tool_dir(root, name).join("tool.ts") }
pub fn tool_config(root: &Path, name: &str) -> PathBuf { tool_dir(root, name).join("config.yaml") }
pub fn tool_index_dir(root: &Path, name: &str) -> PathBuf { tool_dir(root, name).join("index") }
pub fn tools_manifest_path(root: &Path) -> PathBuf { root.join(TOOLS_MANIFEST) }
```

### 5b. `crates/sdlc-core/src/tool_runner.rs` (new file)

```rust
//! Runtime detection and subprocess invocation for SDLC tool scripts.

use std::path::Path;
use std::process::{Command, Stdio};
use crate::error::{Result, SdlcError};

pub enum Runtime { Bun, Deno, Node }

pub fn detect_runtime() -> Option<Runtime> {
    if which::which("bun").is_ok() { return Some(Runtime::Bun) }
    if which::which("deno").is_ok() { return Some(Runtime::Deno) }
    if which::which("npx").is_ok() { return Some(Runtime::Node) }
    None
}

pub fn run_tool(script: &Path, mode: &str, stdin_json: Option<&str>, root: &Path) -> Result<String> {
    let runtime = detect_runtime().ok_or(SdlcError::NoToolRuntime)?;

    let mut cmd = match runtime {
        Runtime::Bun => {
            let mut c = Command::new("bun");
            c.args(["run", script.to_str().unwrap(), mode]);
            c
        }
        Runtime::Deno => {
            let mut c = Command::new("deno");
            c.args(["run", "--allow-read", "--allow-run", "--allow-write",
                    "--allow-env", "--allow-net",
                    script.to_str().unwrap(), mode]);
            c
        }
        Runtime::Node => {
            let mut c = Command::new("npx");
            c.args(["--yes", "tsx", script.to_str().unwrap(), mode]);
            c
        }
    };

    cmd.current_dir(root);
    cmd.env("SDLC_ROOT", root);

    if stdin_json.is_some() {
        cmd.stdin(Stdio::piped());
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());  // let tool stderr flow through for debugging

    let mut child = cmd.spawn().map_err(|e| SdlcError::ToolSpawnFailed(e.to_string()))?;

    if let Some(json) = stdin_json {
        use std::io::Write;
        child.stdin.as_mut().unwrap().write_all(json.as_bytes())?;
    }

    let output = child.wait_with_output().map_err(|e| SdlcError::ToolSpawnFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(SdlcError::ToolFailed(
            String::from_utf8_lossy(&output.stdout).to_string()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

Add to `SdlcError` in `crates/sdlc-core/src/error.rs`:
```rust
NoToolRuntime,           // no bun/deno/node found
ToolSpawnFailed(String), // subprocess failed to start
ToolFailed(String),      // tool exited non-zero
```

### 5c. `crates/sdlc-cli/src/cmd/init.rs`

**In `run()`**, add step 11 after the existing step 10 (scaffold_platform):

```rust
// 11. Write / refresh core SDLC tools
write_core_tools(root)?;
```

**In `write_guidance_md()` content** (`GUIDANCE_MD_CONTENT` const), add a new section §7:

```
## 7. SDLC Tool Suite

This project has SDLC tools installed in `.sdlc/tools/`. These are callable by agents
and humans during any phase of the feature lifecycle.

**Available tools:** Read `.sdlc/tools/tools.md` for the full help menu.

Quick reference:
| Tool           | Command                             | Purpose                        |
|----------------|-------------------------------------|--------------------------------|
| ama            | `sdlc tool run ama --question "..."` | Answer codebase questions      |
| quality-check  | `sdlc tool run quality-check`        | Run project quality gates      |

Run `sdlc tool list` to see all installed tools.
```

**In `build_sdlc_section_inner()`**, add after the "Directive Interface" section:

```rust
"### Tool Suite\n\n\
> **Available tools:** `.sdlc/tools/tools.md` — installed SDLC tools for this project. \
Run `sdlc tool list` to see them. <!-- sdlc:tools -->\n\n\
- `sdlc tool list` — show all installed tools\n\
- `sdlc tool run <name> [--question \"...\"]` — run a tool\n\
- `sdlc tool sync` — regenerate the tools help menu\n\n"
```

**New function `write_core_tools()`**:

```rust
fn write_core_tools(root: &Path) -> anyhow::Result<()> {
    let tools_dir = root.join(paths::TOOLS_DIR);
    io::ensure_dir(&tools_dir)?;

    // tool.ts files — always overwritten (managed content)
    let tools: &[(&str, &str, &str)] = &[
        ("ama",           AMA_TOOL_SCRIPT,           AMA_TOOL_CONFIG),
        ("quality-check", QUALITY_CHECK_TOOL_SCRIPT, QUALITY_CHECK_TOOL_CONFIG),
    ];

    for (name, script, default_config) in tools {
        let tool_dir = tools_dir.join(name);
        io::ensure_dir(&tool_dir)?;

        // tool.ts: always overwrite
        let script_path = tool_dir.join("tool.ts");
        let existed = script_path.exists();
        io::atomic_write(&script_path, script.as_bytes())?;
        if existed { println!("  updated: .sdlc/tools/{name}/tool.ts"); }
        else { println!("  created: .sdlc/tools/{name}/tool.ts"); }

        // config.yaml: write only if missing (user-editable)
        let config_path = tool_dir.join("config.yaml");
        if io::write_if_missing(&config_path, default_config.as_bytes())? {
            println!("  created: .sdlc/tools/{name}/config.yaml");
        }
    }

    // Regenerate tools.md help menu
    write_tools_manifest(root)?;

    // Add .sdlc/tools/*/index/ to .gitignore if not already present
    ensure_tools_gitignore(root)?;

    Ok(())
}
```

**Embedded tool scripts** (const strings, embedded at compile time):

```rust
const AMA_TOOL_SCRIPT: &str = include_str!("../../../assets/tools/ama.ts");
const QUALITY_CHECK_TOOL_SCRIPT: &str = include_str!("../../../assets/tools/quality-check.ts");
const AMA_TOOL_CONFIG: &str = include_str!("../../../assets/tools/ama-config.yaml");
const QUALITY_CHECK_TOOL_CONFIG: &str = include_str!("../../../assets/tools/quality-check-config.yaml");
```

Tool scripts live in `crates/sdlc-cli/assets/tools/` and are baked into the binary at compile time. This is the same approach used for platform scripts (`MASQ_DEPLOY_SCRIPT` etc.).

**`write_tools_manifest()`**:

```rust
fn write_tools_manifest(root: &Path) -> anyhow::Result<()> {
    // For init/update: generate the manifest from static metadata (no subprocess)
    // because the user may not have bun/deno yet. The manifest is pre-built from
    // the embedded tool scripts' hardcoded meta blocks.
    let manifest = build_static_tools_manifest();
    let path = paths::tools_manifest_path(root);
    io::atomic_write(&path, manifest.as_bytes())?;
    println!("  updated: .sdlc/tools/tools.md");
    Ok(())
}
```

### 5d. `crates/sdlc-cli/src/cmd/tool.rs` (new file)

```rust
//! `sdlc tool` subcommands.

use clap::Subcommand;
use std::path::Path;

#[derive(Subcommand)]
pub enum ToolCommand {
    /// List installed tools
    List,
    /// Run a tool
    Run {
        /// Tool name (e.g. ama, quality-check)
        name: String,
        /// JSON input (reads from stdin if not provided)
        #[arg(long)]
        json: Option<String>,
        // convenience flags forwarded as JSON:
        #[arg(long)] question: Option<String>,
        #[arg(long)] scope: Option<String>,
        #[arg(long)] setup: bool,
    },
    /// Regenerate .sdlc/tools/tools.md
    Sync,
    /// Show detailed info about a tool
    Info { name: String },
}

pub fn run(cmd: ToolCommand, root: &Path) -> anyhow::Result<()> {
    match cmd {
        ToolCommand::List => list_tools(root),
        ToolCommand::Run { name, json, question, scope, setup } => {
            let mode = if setup { "--setup" } else { "--run" };
            let input_json = build_input_json(json, question, scope);
            run_tool(root, &name, mode, input_json.as_deref())
        }
        ToolCommand::Sync => sync_manifest(root),
        ToolCommand::Info { name } => tool_info(root, &name),
    }
}
```

### 5e. `crates/sdlc-server/src/routes/tools.rs` (new file)

```rust
//! REST routes for SDLC tool invocation.
//!
//! GET  /api/tools          → list installed tools (reads tools.md metadata)
//! GET  /api/tools/:name    → tool metadata (--meta output)
//! POST /api/tools/:name/run → run a tool (body = JSON input, streams output via SSE)
//! GET  /api/tools/:name/mcp → MCP-compatible streaming endpoint

// Routes added to lib.rs build_router():
// .route("/api/tools", get(routes::tools::list_tools))
// .route("/api/tools/{name}", get(routes::tools::get_tool))
// .route("/api/tools/{name}/run", post(routes::tools::run_tool))
```

---

## 6. Core Tool Scripts

### 6a. AMA (`crates/sdlc-cli/assets/tools/ama.ts`)

```typescript
/**
 * AMA — Ask Me Anything
 * Searches the project codebase and returns relevant excerpts for a question.
 *
 * Protocol:
 *   --meta    → writes ToolMeta JSON to stdout
 *   --run     → reads JSON from stdin, writes ToolResult JSON to stdout
 *   --setup   → indexes the codebase, writes ToolResult JSON to stdout
 */

import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { join, relative } from "node:path";
import { globSync } from "node:fs"; // Bun/Node 22+ built-in

const ROOT = process.env.SDLC_ROOT ?? process.cwd();
const INDEX_DIR = join(ROOT, ".sdlc/tools/ama/index");
const CONFIG_PATH = join(ROOT, ".sdlc/tools/ama/config.yaml");

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

export const meta = {
  name: "ama",
  display_name: "Ask Me Anything",
  description: "Answer questions about the codebase. Returns relevant file excerpts with line numbers.",
  version: "0.1.0",
  requires_setup: true,
  setup_description: "Walks and indexes the project source files into .sdlc/tools/ama/index/",
  input_schema: {
    type: "object",
    required: ["question"],
    properties: {
      question: { type: "string", description: "The question to answer" }
    }
  },
  output_schema: {
    type: "object",
    properties: {
      answer: { type: "string" },
      sources: {
        type: "array",
        items: {
          type: "object",
          properties: {
            path: { type: "string" },
            lines: { type: "array", items: { type: "number" } },
            excerpt: { type: "string" }
          }
        }
      }
    }
  }
};

// ---------------------------------------------------------------------------
// Setup — index the codebase
// ---------------------------------------------------------------------------

interface Chunk { path: string; lines: [number, number]; text: string; keywords: string[] }

function loadConfig() {
  // Reads .sdlc/tools/ama/config.yaml — include/exclude patterns
  // Returns sensible defaults if config missing
  return {
    include: ["**/*.ts", "**/*.tsx", "**/*.rs", "**/*.go", "**/*.py",
              "**/*.md", "**/*.json", "**/*.yaml", "**/*.toml"],
    exclude: ["node_modules/**", "target/**", ".git/**", ".sdlc/tools/ama/index/**",
              "dist/**", "build/**", "*.lock", "frontend/dist/**"]
  };
}

async function setup(): Promise<{ files_indexed: number }> {
  mkdirSync(INDEX_DIR, { recursive: true });
  const config = loadConfig();
  const chunks: Chunk[] = [];

  // Walk files respecting include/exclude
  for (const pattern of config.include) {
    const files = globSync(pattern, { cwd: ROOT, ignore: config.exclude, nodir: true });
    for (const file of files) {
      const abs = join(ROOT, file);
      try {
        const content = readFileSync(abs, "utf8");
        const lines = content.split("\n");
        // Chunk into 40-line windows with 5-line overlap
        const CHUNK_SIZE = 40, OVERLAP = 5;
        for (let i = 0; i < lines.length; i += CHUNK_SIZE - OVERLAP) {
          const slice = lines.slice(i, i + CHUNK_SIZE);
          const text = slice.join("\n");
          const keywords = extractKeywords(text);
          chunks.push({
            path: relative(ROOT, abs),
            lines: [i + 1, Math.min(i + CHUNK_SIZE, lines.length)],
            text,
            keywords
          });
        }
      } catch { /* skip unreadable files */ }
    }
  }

  writeFileSync(join(INDEX_DIR, "chunks.json"), JSON.stringify(chunks), "utf8");
  return { files_indexed: chunks.length };
}

function extractKeywords(text: string): string[] {
  // Extract identifier-like tokens for fast keyword matching
  const tokens = text.match(/\b[a-zA-Z_][a-zA-Z0-9_]{2,}\b/g) ?? [];
  return [...new Set(tokens.map(t => t.toLowerCase()))];
}

// ---------------------------------------------------------------------------
// Run — answer a question
// ---------------------------------------------------------------------------

interface Source { path: string; lines: [number, number]; excerpt: string }
interface AmaResult { answer: string; sources: Source[] }

async function run(input: { question: string }): Promise<AmaResult> {
  const indexPath = join(INDEX_DIR, "chunks.json");
  if (!existsSync(indexPath)) {
    return {
      answer: "Index not built. Run: sdlc tool run ama --setup",
      sources: []
    };
  }

  const chunks: Chunk[] = JSON.parse(readFileSync(indexPath, "utf8"));
  const queryKeywords = extractKeywords(input.question);

  // Score chunks by keyword overlap + path relevance
  const scored = chunks.map(chunk => {
    const keywordScore = queryKeywords.filter(k => chunk.keywords.includes(k)).length;
    const pathScore = queryKeywords.some(k => chunk.path.toLowerCase().includes(k)) ? 2 : 0;
    return { ...chunk, score: keywordScore + pathScore };
  });

  const top = scored
    .filter(c => c.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 5);

  if (top.length === 0) {
    return { answer: "No relevant code found for that question.", sources: [] };
  }

  const sources: Source[] = top.map(c => ({
    path: c.path,
    lines: c.lines,
    excerpt: c.text.slice(0, 400) + (c.text.length > 400 ? "..." : "")
  }));

  // Synthesize a brief textual answer (no LLM call — just structured summary)
  const pathList = [...new Set(top.map(c => c.path))].slice(0, 3).join(", ");
  const answer = `Found ${top.length} relevant locations. Most relevant: ${pathList}. ` +
    `See sources for full excerpts.`;

  return { answer, sources };
}

// ---------------------------------------------------------------------------
// CLI entrypoint
// ---------------------------------------------------------------------------

async function readStdin(): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) chunks.push(chunk as Buffer);
  return Buffer.concat(chunks).toString("utf8");
}

const mode = process.argv[2] ?? "--run";

if (mode === "--meta") {
  console.log(JSON.stringify(meta));
} else if (mode === "--setup") {
  try {
    const result = await setup();
    console.log(JSON.stringify({ ok: true, data: result }));
  } catch (e) {
    console.log(JSON.stringify({ ok: false, error: String(e) }));
    process.exit(1);
  }
} else if (mode === "--run") {
  try {
    const input = JSON.parse(await readStdin());
    const result = await run(input);
    console.log(JSON.stringify({ ok: true, data: result }));
  } catch (e) {
    console.log(JSON.stringify({ ok: false, error: String(e) }));
    process.exit(1);
  }
}
```

### 6b. Quality Check (`crates/sdlc-cli/assets/tools/quality-check.ts`)

```typescript
/**
 * Quality Check — Run SDLC quality gates on demand.
 *
 * Reads shell gates from .sdlc/config.yaml and runs them.
 * Returns structured pass/fail results.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

const ROOT = process.env.SDLC_ROOT ?? process.cwd();

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

export const meta = {
  name: "quality-check",
  display_name: "Dev Quality Check",
  description: "Run the project quality gates from .sdlc/config.yaml on demand.",
  version: "0.1.0",
  requires_setup: false,
  input_schema: {
    type: "object",
    properties: {
      scope: {
        type: "string",
        description: "Gate action name to filter by (e.g. 'implement_task'). Omit to run all gates."
      }
    }
  },
  output_schema: {
    type: "object",
    properties: {
      passed: { type: "number" },
      failed: { type: "number" },
      checks: {
        type: "array",
        items: {
          type: "object",
          properties: {
            name: { type: "string" },
            command: { type: "string" },
            status: { enum: ["passed", "failed"] },
            output: { type: "string" },
            duration_ms: { type: "number" }
          }
        }
      }
    }
  }
};

// ---------------------------------------------------------------------------
// Config parsing (minimal YAML — just enough for gates)
// ---------------------------------------------------------------------------

interface ShellGate {
  type: "shell";
  command: string;
  name: string;
}

function loadShellGates(scope?: string): Array<{ action: string; gate: ShellGate }> {
  const configPath = join(ROOT, ".sdlc/config.yaml");
  const raw = readFileSync(configPath, "utf8");

  // Very simple YAML parser for the gates section.
  // Format: gates:\n  <action>:\n    - type: shell\n      command: "..."\n      name: "..."
  // In practice, we shell out to `sdlc config show --json` to get parsed gates.
  // (sdlc binary is always available when this tool runs)
  const result = spawnSync("sdlc", ["config", "show", "--json"], { cwd: ROOT, encoding: "utf8" });
  if (result.status !== 0) throw new Error("Failed to load config: " + result.stderr);

  const config = JSON.parse(result.stdout);
  const gates = config.gates ?? {};
  const results: Array<{ action: string; gate: ShellGate }> = [];

  for (const [action, gateList] of Object.entries(gates)) {
    if (scope && action !== scope) continue;
    for (const gate of (gateList as ShellGate[])) {
      if (gate.type === "shell") results.push({ action, gate });
    }
  }

  return results;
}

// ---------------------------------------------------------------------------
// Run gates
// ---------------------------------------------------------------------------

interface CheckResult {
  name: string;
  command: string;
  action: string;
  status: "passed" | "failed";
  output: string;
  duration_ms: number;
}

async function run(input: { scope?: string }): Promise<{ passed: number; failed: number; checks: CheckResult[] }> {
  const gates = loadShellGates(input.scope);

  if (gates.length === 0) {
    return {
      passed: 0,
      failed: 0,
      checks: []
    };
  }

  const checks: CheckResult[] = [];

  for (const { action, gate } of gates) {
    const start = Date.now();
    const result = spawnSync(gate.command, {
      shell: true,
      cwd: ROOT,
      encoding: "utf8",
      timeout: 120_000
    });
    const duration_ms = Date.now() - start;
    const passed = result.status === 0;

    checks.push({
      name: gate.name,
      command: gate.command,
      action,
      status: passed ? "passed" : "failed",
      output: ((result.stdout ?? "") + (result.stderr ?? "")).trim().slice(0, 2000),
      duration_ms
    });
  }

  return {
    passed: checks.filter(c => c.status === "passed").length,
    failed: checks.filter(c => c.status === "failed").length,
    checks
  };
}

// ---------------------------------------------------------------------------
// CLI entrypoint
// ---------------------------------------------------------------------------

async function readStdin(): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) chunks.push(chunk as Buffer);
  return Buffer.concat(chunks).toString("utf8") || "{}";
}

const mode = process.argv[2] ?? "--run";

if (mode === "--meta") {
  console.log(JSON.stringify(meta));
} else if (mode === "--run") {
  try {
    const raw = await readStdin();
    const input = JSON.parse(raw.trim() || "{}");
    const result = await run(input);
    console.log(JSON.stringify({ ok: true, data: result }));
  } catch (e) {
    console.log(JSON.stringify({ ok: false, error: String(e) }));
    process.exit(1);
  }
}
```

---

## 7. Agent Discovery Flow

### 7a. What agents see after `sdlc init`

In **`AGENTS.md`** (SDLC managed section):
```
### Tool Suite

> **Available tools:** `.sdlc/tools/tools.md` — installed SDLC tools for this project.
> Run `sdlc tool list` to see them. <!-- sdlc:tools -->

- `sdlc tool list` — show all installed tools
- `sdlc tool run ama --question "..."` — ask a question about the codebase
- `sdlc tool run quality-check` — run project quality gates
- `sdlc tool sync` — regenerate the tools help menu
```

In **`.sdlc/guidance.md`** (§7 Tools, always overwritten):
```
## 7. SDLC Tool Suite

Available tools: `.sdlc/tools/tools.md`

| Tool           | Command                                | Purpose                     |
|----------------|----------------------------------------|-----------------------------|
| ama            | `sdlc tool run ama --question "..."`   | Answer codebase questions   |
| quality-check  | `sdlc tool run quality-check`          | Run project quality gates   |
```

### 7b. CLAUDE.md — opt-in pointer

`CLAUDE.md` is user-authored and never overwritten. However, agents installing a new project or running `/sdlc-specialize` should recommend adding:
```markdown
## SDLC Tools
See `.sdlc/tools/tools.md` for installed tools. Use `sdlc tool run <name>` to invoke.
```

### 7c. Discovery flow for an agent

```
1. Agent starts work on a project
2. Reads AGENTS.md → sees "Tool Suite" section with pointer to .sdlc/tools/tools.md
3. Reads .sdlc/tools/tools.md → full menu: names, descriptions, input/output schemas, commands
4. Agent decides which tool to call based on current task
5. Calls: sdlc tool run ama --question "..."
6. Gets JSON result back, proceeds with work
```

---

## 8. LLM Skill Templates

These are the exact templates installed by `sdlc init` / `sdlc update` as slash commands.

### 8a. `/sdlc-tool-build <name> <description>` — Build a New Tool

**File:** `~/.claude/commands/sdlc-tool-build.md`

```markdown
---
description: Scaffold and build a new SDLC tool for this project
argument-hint: <name> <description>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

Build a new SDLC tool named `$name` with description `$description`.

## Your job

You are a TypeScript tool engineer. You will scaffold a complete, working SDLC tool
that follows the contract in `docs/sdlc-tools-suite-technical.md`. Read that file first.

## Steps

### 1. Read the technical spec
Read `docs/sdlc-tools-suite-technical.md` sections 2 (Tool Script Contract) and 6 (Core Tool Scripts)
to understand exactly what shape the tool must have.

### 2. Understand the project
Run `sdlc tool list` to see what tools are already installed.
Grep for existing tool scripts: `ls .sdlc/tools/`

### 3. Scaffold the tool directory
```bash
mkdir -p .sdlc/tools/$ARGUMENTS
```

### 4. Write `tool.ts`
Write `.sdlc/tools/$ARGUMENTS/tool.ts` following the contract:
- Export `meta` with correct `name`, `display_name`, `description`, `input_schema`, `output_schema`
- Export `async function run(input, ctx)` that returns `{ ok: true, data: ... }`
- Handle `--meta`, `--run` CLI modes
- Write error path: `{ ok: false, error: "..." }` on any thrown error
- Use only Node.js built-ins + Bun built-ins — no npm dependencies unless necessary
- If external deps needed, document them in the tool's config.yaml under `dependencies:`

### 5. Write `config.yaml`
Write `.sdlc/tools/$ARGUMENTS/config.yaml`:
```yaml
name: $ARGUMENTS
version: 0.1.0
# Add tool-specific config here
```

### 6. Test the tool
```bash
# Test metadata
bun run .sdlc/tools/$ARGUMENTS/tool.ts --meta | jq .

# Test run (adjust input JSON for the tool)
echo '{}' | bun run .sdlc/tools/$ARGUMENTS/tool.ts --run | jq .

# Test via sdlc
sdlc tool info $ARGUMENTS
sdlc tool run $ARGUMENTS --json '{}'
```

### 7. Register in tools.md
```bash
sdlc tool sync
```

Verify `.sdlc/tools/tools.md` now includes the new tool.

### 8. Commit message
Stage and describe what the tool does: `feat: add <name> SDLC tool — <one-line description>`

## Quality bar
- `--meta` returns valid JSON matching ToolMeta shape
- `--run` with valid input returns `{ ok: true, data: ... }`
- `--run` with invalid input returns `{ ok: false, error: "..." }` and exits 1
- Tool handles missing config files gracefully (falls back to defaults)
- No unhandled promise rejections

**Next:** Verify with `sdlc tool run <name> --json '<test-input>'` and check the output.
```

---

### 8b. `/sdlc-tool-run <name>` — Run a Tool and Act on the Result

**File:** `~/.claude/commands/sdlc-tool-run.md`

```markdown
---
description: Run an SDLC tool and act on the result
argument-hint: <name> [--question "..." | --scope <scope>]
allowed-tools: Bash, Read, Write, Edit
---

Run the SDLC tool `$ARGUMENTS` and use the result to inform your current work.

## Steps

### 1. Check the tool exists
```bash
sdlc tool info $ARGUMENTS
```
If not found, run `sdlc tool list` to see available tools.

### 2. Run the tool
```bash
sdlc tool run $ARGUMENTS
```
Or with input:
```bash
sdlc tool run $ARGUMENTS --json '{"question": "..."}'
```

### 3. Interpret the result
- If `ok: false` → the tool failed. Read the error message and diagnose.
  - If "Index not built" → run `sdlc tool run ama --setup` first.
  - If "no supported runtime found" → install bun: https://bun.sh
  - Other errors → read the tool script at `.sdlc/tools/$ARGUMENTS/tool.ts` to debug.
- If `ok: true` → use `data` to inform your work.

### 4. For AMA results
Read the `sources` array. Each source has `path`, `lines`, `excerpt`.
Use the file paths and line numbers to read the actual code:
```bash
# Read a source file at the relevant lines
```
Then synthesize the answer from the excerpts.

### 5. For quality-check results
Read `checks`. For each `status: "failed"` check:
- Read its `output` for the error message
- Fix the underlying issue
- Re-run: `sdlc tool run quality-check`
Loop until all checks pass.

**Next:** After using tool results, continue with `sdlc next --for <slug>`.
```

---

### 8c. `/sdlc-tool-audit <name>` — Audit a Tool for Correctness

**File:** `~/.claude/commands/sdlc-tool-audit.md`

```markdown
---
description: Audit an SDLC tool script for correctness, completeness, and quality
argument-hint: <name>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

Audit the SDLC tool `$ARGUMENTS` against the contract in `docs/sdlc-tools-suite-technical.md`.

## Steps

### 1. Read the tool script
Read `.sdlc/tools/$ARGUMENTS/tool.ts`

### 2. Check the contract checklist

**Metadata (`--meta` mode)**
- [ ] `meta.name` matches the directory name
- [ ] `meta.description` is a single clear sentence
- [ ] `meta.input_schema` is a valid JSON Schema object
- [ ] `meta.output_schema` is a valid JSON Schema object
- [ ] `meta.requires_setup` is accurate (true if --setup must be run first)

**Run mode (`--run` mode)**
- [ ] Reads input from stdin as JSON
- [ ] Returns `{ ok: true, data: ... }` on success
- [ ] Returns `{ ok: false, error: "message" }` and exits 1 on failure
- [ ] `data` shape matches `meta.output_schema`
- [ ] No unhandled promise rejections (all awaits inside try/catch)
- [ ] Handles missing/malformed input gracefully

**CLI entrypoint**
- [ ] Handles `--meta`, `--run` args
- [ ] If `requires_setup: true`, also handles `--setup`
- [ ] Uses `process.argv[2]` (compatible with bun, deno, node)

**Config**
- [ ] `.sdlc/tools/$ARGUMENTS/config.yaml` exists
- [ ] Tool uses `SDLC_ROOT` env var to find project root (not hardcoded paths)

### 3. Run the tool
```bash
# Smoke test --meta
bun run .sdlc/tools/$ARGUMENTS/tool.ts --meta | jq .

# Smoke test --run with empty input
echo '{}' | bun run .sdlc/tools/$ARGUMENTS/tool.ts --run | jq .

# Test via sdlc cli
sdlc tool run $ARGUMENTS --json '{}'
```

### 4. Fix any issues found
For each checklist item that fails, fix the tool script.

### 5. Regenerate tools.md
```bash
sdlc tool sync
```

**Next:** After fixing, re-run the audit smoke tests to confirm all pass.
```

---

### 8d. `/sdlc-tool-uat <name>` — User Acceptance Test for a Tool

**File:** `~/.claude/commands/sdlc-tool-uat.md`

```markdown
---
description: Run UAT scenarios for an SDLC tool and record results
argument-hint: <name>
allowed-tools: Bash, Read, Write, Edit
---

Run acceptance tests for the SDLC tool `$ARGUMENTS`.

## UAT Protocol

Record results as you go. At the end, write a verdict: PASS or FAIL with notes.

### Scenario 1 — Metadata Contract
```bash
bun run .sdlc/tools/$ARGUMENTS/tool.ts --meta | jq .
```
**Expected:** Valid JSON with `name`, `description`, `input_schema`, `output_schema`, `version`
**Verdict:** [ ] PASS  [ ] FAIL

### Scenario 2 — Happy Path
```bash
# For AMA:
echo '{"question":"what is this project?"}' | \
  bun run .sdlc/tools/ama/tool.ts --run | jq .

# For quality-check:
echo '{}' | bun run .sdlc/tools/quality-check/tool.ts --run | jq .
```
**Expected:** `{ "ok": true, "data": { ... } }` with data matching output_schema
**Verdict:** [ ] PASS  [ ] FAIL

### Scenario 3 — Error Handling
```bash
# Send malformed JSON
echo 'NOT JSON' | bun run .sdlc/tools/$ARGUMENTS/tool.ts --run | jq .
```
**Expected:** `{ "ok": false, "error": "..." }` and process exits 1
**Verdict:** [ ] PASS  [ ] FAIL

### Scenario 4 — CLI wrapper
```bash
sdlc tool run $ARGUMENTS --json '{}'
sdlc tool info $ARGUMENTS
```
**Expected:** Same output as direct invocation; info shows correct metadata
**Verdict:** [ ] PASS  [ ] FAIL

### Scenario 5 — tools.md Discovery
```bash
cat .sdlc/tools/tools.md | grep -A 10 "## $ARGUMENTS"
```
**Expected:** Tool appears in help menu with correct description, command examples, input/output schemas
**Verdict:** [ ] PASS  [ ] FAIL

## Write UAT Result

Write results to `.sdlc/tools/$ARGUMENTS/uat_results.md`:
```markdown
# UAT Results: $ARGUMENTS

Date: <today>
Tester: <agent-name>
Verdict: PASS | FAIL

## Scenarios
| Scenario | Verdict | Notes |
|----------|---------|-------|
| Metadata Contract | PASS | ... |
| Happy Path | PASS | ... |
| Error Handling | PASS | ... |
| CLI wrapper | PASS | ... |
| Discovery | PASS | ... |

## Notes
<any observations>
```

**Next:** If FAIL — run `/sdlc-tool-audit <name>` to find and fix issues.
```

---

## 9. `sdlc tool sync` Implementation

`sync` is how the tools.md gets regenerated after custom tools are added.

For the core tools (ama, quality-check), the manifest is generated from static metadata in Rust (no subprocess). For custom tools, `sdlc tool sync` runs `bun run .sdlc/tools/<name>/tool.ts --meta` and formats the results.

```rust
// crates/sdlc-cli/src/cmd/tool.rs

fn sync_manifest(root: &Path) -> anyhow::Result<()> {
    use sdlc_core::tool_runner;

    let tools_dir = paths::tools_dir(root);
    let mut entries: Vec<ToolEntry> = Vec::new();

    for entry in std::fs::read_dir(&tools_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() { continue; }
        let script = entry.path().join("tool.ts");
        if !script.exists() { continue; }

        match tool_runner::run_tool(&script, "--meta", None, root) {
            Ok(json) => {
                if let Ok(meta) = serde_json::from_str::<ToolMeta>(&json) {
                    entries.push(meta.into());
                }
            }
            Err(e) => {
                eprintln!("warning: could not get metadata for {:?}: {}", entry.path(), e);
            }
        }
    }

    let manifest = format_tools_manifest(&entries);
    io::atomic_write(&paths::tools_manifest_path(root), manifest.as_bytes())?;
    println!("  updated: .sdlc/tools/tools.md ({} tools)", entries.len());
    Ok(())
}
```

---

## 10. `.gitignore` Management

`sdlc init` adds tool index directories to `.gitignore`:

```
# SDLC Tools (generated indexes — rebuild with `sdlc tool run ama --setup`)
.sdlc/tools/*/index/
.sdlc/tools/*/uat_results.md
```

This is appended via `io::append_text` with a check for duplicates (grep for the comment header first).

---

## 11. New `sdlc config show --json` Subcommand

The quality-check tool needs to read parsed gate config. This requires adding:

```
sdlc config show --json
```

Which emits the full parsed config as JSON. This command is also useful for agents in general. Implementation:

```rust
// crates/sdlc-cli/src/cmd/config.rs — add Show variant:
ConfigCommand::Show { json } => {
    let config = Config::load(root)?;
    if json { print_json(&config)?; }
    else { /* pretty-print */ }
}
```

---

## Implementation Summary

| Component | Location | Effort |
|-----------|----------|--------|
| Path constants | `crates/sdlc-core/src/paths.rs` | Small — add 6 constants + helpers |
| `SdlcError` variants | `crates/sdlc-core/src/error.rs` | Small — add 3 variants |
| `tool_runner.rs` | `crates/sdlc-core/src/tool_runner.rs` | Medium — runtime detection + subprocess |
| `tool.rs` CLI command | `crates/sdlc-cli/src/cmd/tool.rs` | Medium — list/run/sync/info |
| Tool assets | `crates/sdlc-cli/assets/tools/*.ts` | Largest — ama.ts + quality-check.ts |
| `init.rs` changes | `crates/sdlc-cli/src/cmd/init.rs` | Medium — write_core_tools(), AGENTS.md + guidance.md updates |
| Server routes | `crates/sdlc-server/src/routes/tools.rs` | Medium — list/run REST + MCP endpoint |
| Config command | `crates/sdlc-cli/src/cmd/config.rs` | Small — add show --json |
| Slash commands | `crates/sdlc-cli/src/cmd/init.rs` const strings | Medium — 4 new commands |
| Frontend tools page | `frontend/src/pages/ToolsPage.tsx` | Medium — list + run UI |
| Tests | `crates/sdlc-cli/tests/integration.rs` | Medium — tool install + sync tests |
