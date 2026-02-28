# SDLC Tools Suite — Implementation Plan (8 Weeks)

**Spec references:** `docs/sdlc-tools-suite.md`, `docs/sdlc-tools-suite-technical.md`
**Date:** 2026-02-27
**Updated:** 2026-02-28 (Week 7 complete)

---

## Status Summary

| Week | Theme | Status |
|------|-------|--------|
| 1 | Rust Core Infrastructure | ✅ DONE |
| 2 | Common Interface + AMA Tool | ✅ DONE |
| 3 | Quality Check + Full Tool Suite Integration | ✅ DONE |
| 4 | Server Routes (API Layer) | ✅ DONE |
| 5 | Backend Polish + Cross-Platform Validation | ✅ DONE |
| 6 | Frontend UI — View and Run Tools | ✅ DONE |
| 7 | Template Skill — The Definitive Tool Builder | ✅ DONE |
| 8 | Tool Creation via MCP + sdlc UI | ⬜ NOT STARTED |

### What's done (Weeks 1–4 complete)

**`crates/sdlc-core/src/paths.rs`** — all tool path constants and helpers added:
- `TOOLS_DIR`, `TOOLS_MANIFEST`, `TOOLS_SHARED_DIR` constants
- `tools_dir()`, `tool_dir()`, `tool_script()`, `tool_config()`, `tool_readme()`, `tool_index_dir()`, `tools_manifest_path()`, `tools_shared_dir()`

**`crates/sdlc-core/src/error.rs`** — three tool error variants added:
- `NoToolRuntime`, `ToolSpawnFailed(String)`, `ToolFailed(String)` with actionable messages

**`crates/sdlc-core/src/tool_runner.rs`** (new) — full implementation (not a stub):
- `detect_runtime()`, `run_tool()`, `build_command()` for Bun/Deno/Node

**`crates/sdlc-cli/src/cmd/tool.rs`** (new) — full implementation (all subcommands):
- `List`, `Run` (with `--question`, `--scope`, `--setup` convenience flags), `Sync`, `Info`, `Scaffold`
- `sync_manifest()` — reads `--meta` from each tool, formats and writes `tools.md`
- `scaffold_tool()` — creates full skeleton: `tool.ts`, `config.yaml`, `README.md`

**`crates/sdlc-cli/src/main.rs`** — `sdlc tool <subcommand>` wired into clap

**`crates/sdlc-cli/src/cmd/config.rs`** — `sdlc config show --json` implemented

**`crates/sdlc-cli/src/cmd/init.rs`** — TypeScript tool suite embedded and installed by `write_core_tools()`:
- `_shared/types.ts`, `_shared/log.ts`, `_shared/config.ts`, `_shared/runtime.ts` (managed, always overwritten)
- `ama/tool.ts` (managed), `ama/config.yaml` + `ama/README.md` (write-if-missing)
- `quality-check/tool.ts` (managed), `quality-check/config.yaml` + `quality-check/README.md` (write-if-missing)
- `tools.md` static manifest (overwritten; `sdlc tool sync` can regenerate from live metadata)
- 4 slash commands: `SDLC_TOOL_RUN_COMMAND`, `SDLC_TOOL_BUILD_COMMAND`, `SDLC_TOOL_AUDIT_COMMAND`, `SDLC_TOOL_UAT_COMMAND`
- `### Tool Suite` subsection in `build_sdlc_section_inner()` with `<!-- sdlc:tools -->` marker
- `guidance.md §7` pointing agents to tool READMEs and `tools.md`

**`crates/sdlc-server/src/routes/tools.rs`** (new) — 4 route handlers:
- `GET /api/tools` — enumerate `.sdlc/tools/`, run `--meta` on each, return array; skips `_shared` and dirs without `tool.ts`; skips individual tools that fail `--meta` with WARN log
- `GET /api/tools/{name}` — single tool metadata via `--meta`
- `POST /api/tools/{name}/run` — accepts JSON body (or empty `{}`), feeds as stdin to `--run`, returns passthrough JSON
- `POST /api/tools/{name}/setup` — runs `--setup`, returns passthrough JSON
- `validate_tool_name()` helper — 400 for bad names, 404 for missing tools, 503 for no runtime, 422 for tool exit non-zero

**`crates/sdlc-server/src/error.rs`** — error handling improvements:
- `NotFoundError` sentinel added (parallel to `ConflictError`) → `AppError::not_found()`
- `SdlcError::NoToolRuntime` → 503 Service Unavailable
- `SdlcError::ToolFailed` → 422 Unprocessable Entity

**Integration tests** — all passing:
- `test_tool_list_after_init`, `test_config_show_json`
- `test_shared_types_written_by_init`, `test_ama_tool_and_readme_installed`, `test_guidance_md_has_tools_section`
- `test_quality_check_tool_installed`, `test_agents_md_has_tool_suite_section`, `test_tool_slash_commands_installed`
- `list_tools_returns_empty_array_when_no_tools_dir`, `list_tools_skips_dirs_without_tool_ts`
- `get_tool_returns_404_when_not_found`, `run_tool_returns_404_when_not_found`, `setup_tool_returns_404_when_not_found`
- `validate_tool_name_*` (2 tests), `no_tool_runtime_maps_to_503`, `tool_failed_maps_to_422`, `not_found_constructor_maps_to_404`

**`crates/sdlc-cli/src/cmd/init.rs`** — AMA tool upgraded to v0.2.0 (Week 5):
- Code-aware tokenization: `extractTokens()` splits camelCase + acronym boundaries (`featureTransition` → `feature`, `transition`; `SdlcError` → `sdlc`, `error`)
- TF-IDF scoring: IDF precomputed at index time (smoothed `log((N+1)/(df+1))+1`), stored in `chunks.json` as `idf: Record<string, number>`, used in `scoreChunks()` for IDF-weighted overlap; falls back to uniform weights for v1 indexes
- Incremental indexing: `last_indexed.json` mtime map; `--setup` skips unchanged files, re-indexes changed/new files, prunes deleted files from index
- Stale source detection: `--run` cross-checks file mtime against `last_indexed.json`; adds `stale: true` to sources from files changed since last setup
- Rich telemetry: `files_indexed`, `files_skipped`, `files_pruned`, `chunks_written`, `total_chunks`, `duration_ms`, `index_size_kb`
- Graceful backward compatibility: v1 indexes (no `idf` field) continue to work — `scoreChunks()` uses uniform weights
- `MtimeMap` interface added; `Index.version` + `Index.idf` added; `AmaSource.score` + `AmaSource.stale?` added
- 4 new integration tests: `test_ama_tool_has_incremental_indexing`, `test_ama_tool_has_tfidf_scoring`, `test_ama_tool_has_camelcase_tokenization`, `test_ama_tool_version_is_v2`
- Total tests: 390 passing, 0 failing

### What remains
- Week 6: Frontend Tools page, AmaResultPanel, QualityCheckPanel, Sidebar nav
- Week 7: Expanded slash command consts (full 11-step build, 18-item audit, 7-scenario UAT templates); Gemini/OpenCode variants
- Week 8: `POST /api/tools` → agent run + `CreateToolModal` + SSE streaming

---

## Week 1: Rust Core Infrastructure ✅ DONE

**Goals:**
- Every subsequent week builds on a clean, tested Rust foundation
- `sdlc tool list` runs without crashing on any project
- Runtime detection works on macOS, Linux, and Windows (bun → deno → node fallback chain)

**Status:** All tasks complete. `sdlc tool scaffold`, `sdlc tool sync`, and `sdlc tool info` were implemented here (ahead of their Week 7/3 planned schedule).

**Tasks:**

1. ✅ **`crates/sdlc-core/src/paths.rs`** — add constants and helpers:
   ```rust
   pub const TOOLS_DIR: &str = ".sdlc/tools";
   pub const TOOLS_MANIFEST: &str = ".sdlc/tools/tools.md";
   // + tools_dir(), tool_dir(), tool_script(), tool_config(),
   //   tool_index_dir(), tools_manifest_path(), tool_readme_path()
   ```

2. ✅ **`crates/sdlc-core/src/error.rs`** — add 3 variants with actionable Display messages:
   - `NoToolRuntime` → "Install bun to use SDLC tools: curl -fsSL https://bun.sh/install | bash"
   - `ToolSpawnFailed(String)` → "Failed to spawn tool subprocess: {0}"
   - `ToolFailed(String)` → first 500 chars of tool stderr

3. ✅ **`crates/sdlc-core/src/tool_runner.rs`** (new file):
   - `detect_runtime() -> Option<Runtime>` (Bun > Deno > Node via `which`)
   - `run_tool(script, mode, stdin_json, root) -> Result<String>` — spawns subprocess, returns stdout
   - Bun: `bun run script.ts --mode`
   - Deno: `deno run --allow-read --allow-run --allow-write --allow-env script.ts --mode`
   - Node: `npx --yes tsx script.ts --mode`

4. ✅ **`crates/sdlc-cli/src/cmd/tool.rs`** (new file) — full implementation (not stubs):
   - `ToolCommand` enum: `List`, `Run`, `Sync`, `Info`
   - `list_tools(root)` — reads `tools_dir`, prints name + description from config.yaml
   - Other subcommands: `todo!()` stubs

5. ✅ **`crates/sdlc-cli/src/main.rs`** — wire `sdlc tool <subcommand>` into clap

6. ✅ **`crates/sdlc-cli/src/cmd/config.rs`** — add `sdlc config show --json` (needed by quality-check):
   - `Config::load(root)` → `print_json(&config)`

7. **Integration tests:** (partial)
   - ✅ `test_tool_list_empty` — passing
   - ⬜ `test_no_runtime_error_message` — not yet written
   - ✅ `test_config_show_json` — passing

**Deliverables:**
- `SDLC_NO_NPM=1 cargo test --all` passes, `cargo clippy` clean
- `sdlc tool list` runs and prints gracefully on any project
- `sdlc config show --json` works

**Foundation this enables:**
- All tool invocation funnels through `tool_runner::run_tool` — one debug chokepoint
- Error variants give callers specific, fix-included messages

---

## Week 2: Common Interface + AMA Tool (First E2E Tool) ✅ DONE

**Goals:**
- A shared TypeScript foundation that ALL tools (core and custom) import from
- Standard logging that produces readable, consistent stderr output
- AMA tool fully working end-to-end with strong self-documenting code
- `sdlc init` produces a tool directory that agents can understand immediately without human explanation
- `guidance.md` §7 points agents directly to tool READMEs

**Tasks:**

### Common Interface

1. **`crates/sdlc-cli/assets/tools/_shared/types.ts`** (new file) — THE shared TypeScript contract:
   ```typescript
   /**
    * SDLC Tool Shared Interface
    *
    * Every SDLC tool imports from this file. It defines the full type contract
    * that tools must satisfy. Do not change the shape of these types without
    * updating all core tools and regenerating tools.md.
    */

   export interface ToolMeta {
     name: string                  // matches directory name exactly
     display_name: string          // human-readable title
     description: string           // one sentence, present tense
     version: string               // semver, mirrors sdlc binary version
     input_schema: JsonSchema      // JSON Schema for input
     output_schema: JsonSchema     // JSON Schema for output
     requires_setup: boolean       // true if --setup must run before --run
     setup_description?: string    // one sentence describing what setup does
   }

   export interface ToolResult<T = unknown> {
     ok: boolean
     data?: T
     error?: string       // present only when ok = false
     duration_ms?: number
   }

   export interface ToolContext {
     root: string         // absolute path to project root ($SDLC_ROOT)
     config: ToolConfig   // parsed config.yaml
   }

   export interface ToolConfig {
     name: string
     version: string
     [key: string]: unknown  // tool-specific config
   }

   export type JsonSchema = Record<string, unknown>
   ```

2. **`crates/sdlc-cli/assets/tools/_shared/log.ts`** (new file) — standard logging:
   ```typescript
   /**
    * Standard SDLC Tool Logger
    *
    * Writes structured log lines to STDERR (never stdout — stdout is reserved
    * for JSON output). Use this in every tool to produce consistent, parseable logs.
    *
    * Format: [sdlc-tool:<name>] LEVEL: message
    * Example: [sdlc-tool:ama] INFO: Indexed 312 files in 842ms
    *
    * Agents and humans see this in the run panel. Keep messages action-oriented:
    * what the tool is doing, not what it found (results go in the JSON output).
    */

   export function makeLogger(toolName: string) {
     const prefix = `[sdlc-tool:${toolName}]`
     return {
       info:  (msg: string) => console.error(`${prefix} INFO:  ${msg}`),
       warn:  (msg: string) => console.error(`${prefix} WARN:  ${msg}`),
       error: (msg: string) => console.error(`${prefix} ERROR: ${msg}`),
       debug: (msg: string) => {
         if (process.env.SDLC_TOOL_DEBUG) console.error(`${prefix} DEBUG: ${msg}`)
       },
     }
   }

   export type Logger = ReturnType<typeof makeLogger>
   ```

3. **`crates/sdlc-cli/assets/tools/_shared/config.ts`** (new file) — config loader:
   ```typescript
   /**
    * SDLC Tool Config Loader
    *
    * Reads .sdlc/tools/<name>/config.yaml. If the file is missing or unparseable,
    * returns the provided defaults — tools should never hard-fail on missing config.
    */
   export function loadToolConfig<T extends Record<string, unknown>>(
     root: string,
     name: string,
     defaults: T,
   ): T { ... }
   ```

4. **`crates/sdlc-cli/assets/tools/_shared/runtime.ts`** (new file) — cross-runtime compat:
   ```typescript
   /**
    * Cross-runtime helpers for Bun, Deno, and Node.
    * Normalizes: argv access, stdin reading, process exit.
    */
   export function getArgs(): string[]  // process.argv[2...] on all three
   export async function readStdin(): Promise<string>
   export function exit(code: number): never
   ```

   Shared files are written to `.sdlc/tools/_shared/` by `write_core_tools()` — the underscore prefix makes their purpose obvious and prevents them being treated as runnable tools.

### AMA Tool

5. **`crates/sdlc-cli/assets/tools/ama/tool.ts`** — full AMA implementation with strong instruction text:
   ```typescript
   /**
    * AMA — Ask Me Anything
    * =====================
    * Searches the project codebase and returns relevant file excerpts.
    *
    * WHAT IT DOES
    * ------------
    * --setup:  Walks all source files matching config.include patterns, chunks them into
    *           40-line windows, extracts keyword tokens, and writes the index to
    *           .sdlc/tools/ama/index/chunks.json. Must be run once before first --run.
    *           Re-running --setup is safe and overwrites the existing index.
    *
    * --run:    Reads JSON from stdin: { "question": "string" }
    *           Loads the index, scores chunks by keyword overlap, returns top 5 sources.
    *           Falls back gracefully if index is missing (returns setup instruction).
    *
    * --meta:   Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
    *
    * WHAT IT READS
    * -------------
    * - .sdlc/tools/ama/config.yaml  (include/exclude patterns, chunk settings)
    * - .sdlc/tools/ama/index/chunks.json  (built by --setup)
    * - All source files matching config.include patterns  (during --setup only)
    *
    * WHAT IT WRITES
    * --------------
    * - .sdlc/tools/ama/index/chunks.json  (during --setup)
    * - STDERR only during --run (structured log lines via log.ts)
    * - STDOUT: JSON only (ToolResult shape from _shared/types.ts)
    *
    * EXTENDING
    * ---------
    * To improve answer quality: replace the keyword scoring in scoreChunks() with
    * embedding-based cosine similarity (requires a local embedding model or API key).
    * The rest of the pipeline (chunking, index format, protocol) stays the same.
    *
    * For v2 LLM synthesis: call the Claude API in run() with the top excerpts as
    * context. Set ANTHROPIC_API_KEY in the environment. Add "synthesis_model" to
    * config.yaml to control which model is used.
    */

   import type { ToolMeta, ToolResult } from '../_shared/types.ts'
   import { makeLogger } from '../_shared/log.ts'
   import { loadToolConfig } from '../_shared/config.ts'
   import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
   // ... full implementation
   ```

6. **`crates/sdlc-cli/assets/tools/ama/README.md`** — written by `write_core_tools()` as write-if-missing:
   ```markdown
   # AMA — Ask Me Anything

   Answers questions about the codebase by searching a pre-built keyword index.

   ## Setup (run once)
   sdlc tool run ama --setup

   ## Usage
   sdlc tool run ama --question "where is JWT validation?"

   ## Configuration
   Edit .sdlc/tools/ama/config.yaml to change which files are indexed.

   ## How it works
   1. --setup walks source files, chunks into 40-line windows, extracts keywords
   2. --run scores chunks by keyword overlap, returns top 5 file excerpts
   3. Your AI assistant reads the excerpts and synthesizes the answer

   ## Index location
   .sdlc/tools/ama/index/chunks.json (gitignored — regenerate with --setup)
   ```

7. **`crates/sdlc-cli/assets/tools/ama/config.yaml`** — write-if-missing default config

8. **`guidance.md` §7 update** — 1 line description + pointer to READMEs:
   ```markdown
   ## 7. SDLC Tool Suite

   Project-scoped TypeScript tools in `.sdlc/tools/` — callable by agents and humans
   during any lifecycle phase. Read `.sdlc/tools/tools.md` for the help menu or each
   tool's `README.md` for full documentation.

   | Tool          | README                                  | Command                                        |
   |---------------|-----------------------------------------|------------------------------------------------|
   | ama           | `.sdlc/tools/ama/README.md`             | `sdlc tool run ama --question "..."`           |
   | quality-check | `.sdlc/tools/quality-check/README.md`   | `sdlc tool run quality-check`                  |
   ```

9. **`write_core_tools()` in `init.rs`**:
   - Creates `.sdlc/tools/` and `.sdlc/tools/_shared/`
   - Writes shared files (always overwrite — managed content)
   - Writes AMA `tool.ts` (always overwrite) and `config.yaml` + `README.md` (write-if-missing)
   - Calls `write_static_tools_manifest()` — generates `tools.md` from hardcoded metadata

10. **`.gitignore` management** — append `.sdlc/tools/*/index/`

11. **Tests:**
    - `test_shared_types_written_by_init`
    - `test_ama_tool_and_readme_installed`
    - `test_guidance_md_has_tools_section`
    - `test_ama_meta_mode` (skipped if no bun) — `--meta` output parses as valid ToolMeta

**Deliverables:**
- `sdlc init` produces `_shared/types.ts`, `_shared/log.ts`, `ama/tool.ts`, `ama/README.md`, `ama/config.yaml`
- `sdlc tool run ama --setup` indexes this repo, logs structured output to stderr
- `sdlc tool run ama --question "where is JWT validation?"` returns 3-5 relevant excerpts
- `guidance.md` §7 describes the system in 1 sentence and points to per-tool READMEs
- Any agent reading `tools.md` or a tool README can call the tool correctly with zero human guidance

**Foundation this enables:**
- Quality-check (Week 3) imports from the same `_shared/` — no new patterns introduced
- Custom tools written by agents have a clear, opinionated starting point
- The `makeLogger` pattern means every tool's logs look the same in the run panel

---

## Week 3: Quality Check + Full Tool Suite Integration ✅ DONE

**Goals:**
- Second tool working end-to-end
- All 4 slash commands installed by `sdlc update`
- AGENTS.md Tool Suite section live — agents find tools without being told

**Tasks:**

1. ✅ **`TOOL_QUALITY_CHECK_TS` const in `init.rs`** — full TypeScript implementation:
   - Reads `platform.commands` from `sdlc config show --json` (subprocess, no YAML parse dep)
   - Runs each script, records `passed`/`failed`/`output`/`duration_ms`
   - Graceful no-op when no platform commands configured
   - Supports `--scope` filter
2. ✅ **`TOOL_QUALITY_CHECK_CONFIG_YAML` and `TOOL_QUALITY_CHECK_README_MD`** consts — write-if-missing
3. ✅ **Extend `write_core_tools()`** — quality-check installed alongside AMA
4. ✅ **`TOOL_STATIC_TOOLS_MD` updated** — quality-check entry added
5. ✅ **4 slash command consts** embedded in `init.rs`:
   - `SDLC_TOOL_RUN_COMMAND` — check tools.md, run tool, parse result
   - `SDLC_TOOL_BUILD_COMMAND` — 11-step scaffold-to-commit workflow
   - `SDLC_TOOL_AUDIT_COMMAND` — 18-item checklist in 5 categories
   - `SDLC_TOOL_UAT_COMMAND` — 7 scenarios with exact commands
6. ✅ **`write_user_claude_commands()`** — 4 new entries registered
7. ✅ **`build_sdlc_section_inner()`** — `### Tool Suite` subsection with `<!-- sdlc:tools -->` marker
8. ✅ **`migrate_legacy_project_scaffolding()`** — 4 new filenames in claude, gemini, agents lists
9. ✅ **Integration tests** — `test_quality_check_tool_installed`, `test_agents_md_has_tool_suite_section`, `test_tool_slash_commands_installed`

**Deliverables:**
- ✅ `sdlc update` installs all 4 slash commands to `~/.claude/commands/`
- ✅ `sdlc tool run quality-check` reports pass/fail for every shell gate in config
- ✅ AGENTS.md includes Tool Suite section pointing to `tools.md`
- ✅ All tests pass, clippy clean

---

## Week 4: Server Routes (API Layer) ✅ DONE

**Goals:**
- Tools callable via HTTP — enables remote agents and the Week 6 UI
- All routes follow the existing Axum patterns in `crates/sdlc-server/src/routes/`
- Correct error codes with actionable messages

**Tasks:**

1. **`crates/sdlc-server/src/routes/tools.rs`** (new file):
   - `GET /api/tools` → list installed tools (run `--meta` on each via `spawn_blocking`)
   - `GET /api/tools/:name` → single tool metadata
   - `POST /api/tools/:name/run` → body = JSON input; returns `ToolResult` JSON
   - `POST /api/tools/:name/setup` → run setup mode
   - `NoToolRuntime` → 503 `{ "error": "...", "install": "https://bun.sh" }`
   - `ToolFailed` → 422 `{ "error": "...", "stderr": "..." }`
   - Long-running invocations wrapped in `tokio::task::spawn_blocking` with 120s timeout

2. **`crates/sdlc-server/src/routes/mod.rs`** — add `pub mod tools;`

3. **`crates/sdlc-server/src/lib.rs`** — add 4 routes to `build_router()`:
   ```rust
   .route("/api/tools", get(routes::tools::list_tools))
   .route("/api/tools/{name}", get(routes::tools::get_tool))
   .route("/api/tools/{name}/run", post(routes::tools::run_tool))
   .route("/api/tools/{name}/setup", post(routes::tools::setup_tool))
   ```

4. **Server tests:**
   - `test_tools_list_route_empty` — 200 with empty array when no tools installed
   - `test_tools_list_route_with_tools` — returns ToolMeta array when AMA installed
   - `test_tools_run_no_runtime_503` — 503 with install hint when no runtime
   - `test_tools_run_ama` — (integration, skipped if no bun) actual query returns sources

**Deliverables:**
- `curl localhost:3141/api/tools` returns list of installed tools
- `curl -X POST localhost:3141/api/tools/quality-check/run -d '{}'` runs checks, returns JSON
- All server tests pass

---

## Week 5: Backend Polish + Cross-Platform Validation ✅ DONE

**Goals:**
- Error messages are production-quality: every error includes what went wrong AND the fix command
- AMA performs well on large projects (500+ files, < 10s setup)
- Cross-platform validation: macOS ✓, Linux ✓, Windows (Node fallback) ✓

**Tasks:**

1. **AMA incremental re-index** — skip files unchanged since last index run:
   - Write `index/last_indexed.json` with `{ path → mtime }` map
   - On subsequent `--setup`, skip files whose mtime matches
   - Add `files_indexed`, `files_skipped`, `duration_ms` to setup output

2. **AMA file size guard** — skip files > 500KB with a WARN log

3. **Cross-runtime compat sweep** — test `tool.ts --meta` with bun, deno, npx tsx:
   - `import.meta.main` → Bun only; use `getArgs()` from `_shared/runtime.ts` instead
   - `Deno.args` → use cross-runtime shim

4. **Error message quality pass** — every error includes:
   - What went wrong (specific)
   - The exact command to fix it
   - `NoToolRuntime`: link to bun install page

5. **`docs/architecture.md`** — add Tool Suite section:
   - `crates/sdlc-cli/assets/tools/` — TypeScript tool scripts
   - `crates/sdlc-core/src/tool_runner.rs` — runtime detection + subprocess
   - `sdlc tool` — CLI subcommand

6. **CLAUDE.md update** — add Tool Suite row to Key Files table

7. **`sdlc update` smoke test on real project** — verify full install, tools.md generated, AGENTS.md updated

**Deliverables:**
- `sdlc tool run ama --setup` on this 150-file repo completes in < 5s
- Second run (incremental) completes in < 500ms
- All tests pass on macOS and Linux CI
- Node/npx fallback tested and documented

---

## Week 6: Frontend UI — View and Run Tools ✅ DONE

**Goals:**
- Tools are a first-class part of the sdlc-server dashboard
- Humans can run AMA and quality-check directly from the browser
- Tool output is rendered beautifully — quality-check as pass/fail rows, AMA as source cards
- Setup flow is clear and guided for tools that require it

**Tasks:**

1. **`frontend/src/lib/types.ts`** — add types mirroring Rust structs:
   ```typescript
   export interface ToolMeta { name, display_name, description, version,
     requires_setup, setup_description?, input_schema, output_schema }
   export interface ToolResult<T = unknown> { ok: boolean, data?: T, error?: string, duration_ms?: number }
   export interface CheckResult { name, command, action, status: 'passed'|'failed', output, duration_ms }
   export interface QualityCheckData { passed: number, failed: number, checks: CheckResult[] }
   export interface AmaSource { path: string, lines: [number, number], excerpt: string }
   export interface AmaData { answer: string, sources: AmaSource[] }
   ```

2. **`frontend/src/api/client.ts`** — add tool API methods:
   ```typescript
   listTools(): Promise<ToolMeta[]>
   getTool(name: string): Promise<ToolMeta>
   runTool(name: string, input: unknown): Promise<ToolResult>
   setupTool(name: string): Promise<ToolResult>
   ```

3. **`frontend/src/pages/ToolsPage.tsx`** (new file):
   - **Tool list sidebar** (left): cards for each installed tool with name, description, version badge
   - **Tool detail panel** (right): selected tool's full metadata + run interface
   - **"Setup required" banner** — yellow banner with "Run Setup" button if `requires_setup && !index_exists`
   - **Input area** — textarea for raw JSON input; convenience fields for known tools:
     - AMA: single text input "Ask a question..." that maps to `{ "question": "..." }`
     - Quality check: optional scope selector dropdown (populated from sdlc config gates)
   - **Run button** — calls `api.runTool(name, input)`, shows spinner
   - **Output renderer** — dispatches on tool name:
     - `ama` → renders `AmaResultPanel`
     - `quality-check` → renders `QualityCheckPanel`
     - Unknown → pretty-printed JSON with syntax highlighting

4. **`frontend/src/components/tools/AmaResultPanel.tsx`** (new file):
   - Answer text in a subtle card
   - Source list: each source shows `path:startLine-endLine` as a monospace filename chip
   - Expandable excerpt for each source (collapsible code block with syntax highlighting via the existing `MarkdownContent` wrapping)

5. **`frontend/src/components/tools/QualityCheckPanel.tsx`** (new file):
   - Summary row: "2 passed · 1 failed" with colored badges
   - Per-check rows: check name, action scope, duration, expandable output
   - Failed checks highlighted in red; passed in green
   - "Re-run" button per check or all checks

6. **`frontend/src/components/tools/ToolCard.tsx`** (new file):
   - Used in the sidebar list
   - Shows: tool name (mono), description, version badge, setup status indicator

7. **`frontend/src/components/layout/Sidebar.tsx`** — add Tools nav item (Wrench icon) between Dashboard and Features

8. **`frontend/src/App.tsx`** — add `/tools` route

9. **TypeScript typecheck** — `npx tsc --noEmit` clean after all changes

**Deliverables:**
- `/tools` page loads and shows AMA + quality-check cards
- AMA setup can be triggered from the browser (button → spinner → "312 files indexed")
- AMA question can be asked from the browser, sources render as expandable file excerpts
- Quality-check results render as pass/fail rows with expandable output
- `npx tsc --noEmit` clean, `npx vite build` succeeds

---

## Week 7: Template Skill — The Definitive Tool Builder ✅ DONE

**Status:** All 4 slash commands fully expanded and registered on all 4 platforms (Claude Code, Gemini CLI, OpenCode, Agents). `SDLC_TOOL_BUILD_COMMAND` has 12 steps including scaffold, README, audit, UAT, sync, and commit. `SDLC_TOOL_AUDIT_COMMAND` has the full 18-item checklist across 5 categories. `SDLC_TOOL_UAT_COMMAND` has all 7 scenarios with exact commands and expected outputs. Gemini/OpenCode/Agents variants added via `SDLC_TOOL_*_PLAYBOOK` and `SDLC_TOOL_*_SKILL` consts. Integration test `test_tool_commands_all_platforms` verifies all 12 platform files are installed.

**Goals:**
- Any agent invoking `/sdlc-tool-build` produces a tool indistinguishable in quality from the core tools
- The skill is so detailed it requires zero prior knowledge of the tool contract
- Includes: common interface usage, logging pattern, strong instruction text template, README template, config template, audit checklist, UAT scenarios
- A `sdlc tool scaffold <name>` CLI command generates the boilerplate so agents can fill in the logic rather than typing boilerplate

**Tasks:**

1. ✅ **`sdlc tool scaffold <name> <description>`** — fully implemented in `tool.rs`:
   - Creates `.sdlc/tools/<name>/` directory
   - Writes `tool.ts` from a scaffolding template (not the full implementation — skeleton with all the right structure, comments, and TODO markers)
   - Writes `config.yaml` and `README.md` with placeholders
   - Prints next steps: "Fill in the `run()` function, then test with: bun run .sdlc/tools/<name>/tool.ts --meta"
   - Template scaffold `tool.ts` (embedded as `TOOL_SCAFFOLD_TEMPLATE` const in `init.rs`):
   ```typescript
   /**
    * <DISPLAY_NAME>
    * ==============
    * <ONE SENTENCE DESCRIPTION>
    *
    * WHAT IT DOES
    * ------------
    * --setup:  [describe setup if required, or "No setup required."]
    * --run:    Reads JSON from stdin: <input shape>
    *           Returns JSON to stdout: <output shape>
    * --meta:   Writes ToolMeta JSON to stdout.
    *
    * WHAT IT READS
    * -------------
    * - .sdlc/tools/<name>/config.yaml
    * [list any other files it reads]
    *
    * WHAT IT WRITES
    * --------------
    * - STDERR only during --run (structured log lines via _shared/log.ts)
    * - STDOUT: JSON only (ToolResult shape)
    *
    * EXTENDING
    * ---------
    * [Describe the main extension point]
    */

   import type { ToolMeta, ToolResult } from '../_shared/types.ts'
   import { makeLogger } from '../_shared/log.ts'
   import { loadToolConfig } from '../_shared/config.ts'
   import { getArgs, readStdin, exit } from '../_shared/runtime.ts'

   const log = makeLogger('<name>')

   export const meta: ToolMeta = {
     name: '<name>',
     display_name: '<DISPLAY_NAME>',
     description: '<ONE SENTENCE DESCRIPTION>',
     version: '0.1.0',
     requires_setup: false,
     input_schema: {
       type: 'object',
       required: [],       // TODO: add required fields
       properties: {}      // TODO: define input fields
     },
     output_schema: {
       type: 'object',
       properties: {}      // TODO: define output fields
     }
   }

   interface Input {
     // TODO: define input fields
   }

   interface Output {
     // TODO: define output fields
   }

   export async function run(input: Input, ctx: { root: string }): Promise<ToolResult<Output>> {
     log.info('starting')
     try {
       // TODO: implement tool logic
       const result: Output = {}
       log.info('done')
       return { ok: true, data: result }
     } catch (e) {
       log.error(String(e))
       return { ok: false, error: String(e) }
     }
   }

   // CLI entrypoint
   const mode = getArgs()[0] ?? '--run'
   if (mode === '--meta') {
     console.log(JSON.stringify(meta)); exit(0)
   } else if (mode === '--run') {
     readStdin().then(raw => run(JSON.parse(raw || '{}'), { root: process.env.SDLC_ROOT! }))
       .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
       .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
   }
   ```

2. **`SDLC_TOOL_BUILD_COMMAND` const** — EXPANDED version of the slash command. Full template with every step:
   - Step 1: Read the technical spec + look at an existing tool for reference
   - Step 2: Run `sdlc tool scaffold <name> "<description>"` to generate the skeleton
   - Step 3: Fill in `meta.input_schema`, `meta.output_schema`
   - Step 4: Implement `run()` — error path first, then happy path
   - Step 5: Write the instruction header (use the template in `tool.ts`)
   - Step 6: Write `README.md` (use the AMA README as the template)
   - Step 7: Test all 3 modes: `--meta | jq .`, `echo '{}' | --run | jq .`, `--run with bad input exits 1`
   - Step 8: Run `/sdlc-tool-audit <name>` — fix every checklist item before proceeding
   - Step 9: Run `/sdlc-tool-uat <name>` — all 5 scenarios must pass
   - Step 10: `sdlc tool sync` — verify tool appears in `tools.md`
   - Step 11: Commit

3. **`SDLC_TOOL_AUDIT_COMMAND` const** — hardened checklist with 18 specific checks organized by category:
   - Metadata correctness (5 checks)
   - Protocol compliance (4 checks)
   - Error handling (4 checks)
   - Logging correctness (2 checks)
   - Documentation (3 checks)

4. **`SDLC_TOOL_UAT_COMMAND` const** — 7 scenarios with exact commands and expected outputs:
   - Scenario 1: Metadata shape
   - Scenario 2: Happy path with valid input
   - Scenario 3: Error on malformed JSON input
   - Scenario 4: Error on semantically invalid input (missing required fields)
   - Scenario 5: CLI wrapper (`sdlc tool run <name>`)
   - Scenario 6: Logging format (stderr lines match `[sdlc-tool:<name>] LEVEL: ` pattern)
   - Scenario 7: Discovery (appears in `tools.md`)

5. **Gemini + OpenCode variants** of all 4 commands (concise playbook format)

6. **Smoke test**: Run `/sdlc-tool-build` in Claude Code to create a `git-activity` tool. Verify it passes all 7 UAT scenarios before considering this week done.

**Deliverables:**
- `sdlc tool scaffold <name> "<description>"` generates a valid skeleton that passes `--meta` immediately
- `/sdlc-tool-build` skill produces a quality tool on first attempt — no iteration needed
- The audit checklist has zero ambiguous items — every check is pass/fail with a command to verify
- A tool built by an agent following this skill is structurally identical to the core tools

---

## Week 8: Tool Creation via MCP + sdlc UI ⬜ NOT STARTED

**Goals:**
- An agent (or human using the sdlc UI) can create a new SDLC tool by describing what it should do
- The creation process uses the existing `spawn_agent_run` pattern — agent writes the tool, streams progress via SSE
- The result is a production-quality tool installed in `.sdlc/tools/`, not a draft to be fixed

**Tasks:**

### Server Side

1. **`POST /api/tools`** (new route in `tools.rs`) — create a new tool via agent:
   ```rust
   // Request body:
   // { "name": "git-activity", "description": "Show recent git commits with author and message" }
   //
   // Response: same shape as spawn_agent_run (run_id, status, SSE key)
   ```
   - Validates name (slug rules from `paths::validate_slug`)
   - Checks tool doesn't already exist
   - Builds prompt: loads the tool-build skill template + appends "Build tool '{name}': {description}"
   - The prompt includes the FULL skill content (same as `/sdlc-tool-build`) so the agent has complete context
   - Calls `spawn_agent_run(key="tool:create:{name}", prompt, opts, app, "tool_create", name)`
   - Configures `QueryOptions` with Write tool allowed to `.sdlc/tools/<name>/**`

2. **SSE events for tool creation** — reuses existing SSE infrastructure:
   - `GET /api/tools/create/:name/events` — SSE stream for creation run
   - `POST /api/tools/create/:name/stop` — stop creation run

3. **`crates/sdlc-server/src/routes/mod.rs`** — already has `tools`; add new route registrations in `lib.rs`:
   ```rust
   .route("/api/tools", post(routes::tools::create_tool))
   .route("/api/tools/create/{name}/events", get(routes::tools::create_tool_events))
   .route("/api/tools/create/{name}/stop", post(routes::tools::create_tool_stop))
   ```

4. **Post-creation hook** — after agent run completes, server automatically runs `sdlc tool sync` to regenerate `tools.md`, then emits an SSE `tools_updated` event so the UI refreshes

### Frontend Side

5. **`frontend/src/pages/ToolsPage.tsx`** — add "Create Tool" button in the header:
   - Opens `CreateToolModal`

6. **`frontend/src/components/tools/CreateToolModal.tsx`** (new file):
   - Two inputs: Name (slug-validated inline), Description (textarea)
   - "Create Tool" button → `POST /api/tools` → transitions to progress view
   - Progress view: reuses `AgentLog` component for streaming SSE output from `/api/tools/create/:name/events`
   - On completion (`tool_create_completed` SSE event): shows "Tool created!" with "Run it now" link
   - Stop button using `/api/tools/create/:name/stop`

7. **`frontend/src/api/client.ts`** — add:
   ```typescript
   createTool(name: string, description: string): Promise<{ run_id: string, key: string }>
   getToolCreateEvents(name: string): EventSource
   stopToolCreate(name: string): Promise<void>
   ```

8. **SSE hook integration** — `useSSE` already handles generic SSE events; add `tools_updated` event to trigger `refetch()` on the tools list

9. **Prompt construction in `create_tool` route**:
   - Loads the embedded `SDLC_TOOL_BUILD_COMMAND` content
   - Prepends: "You are building a new SDLC tool. The tool must be production-quality and pass all 7 UAT scenarios."
   - Appends: "Tool to build: name='{name}', description='{description}'"
   - Appends: "After building and testing, run `sdlc tool sync` as the final step."
   - Sets `max_tokens_per_turn` high enough for a complete tool (tool.ts + README + config.yaml)

10. **Integration tests:**
    - `test_create_tool_validates_slug` — returns 400 for invalid names
    - `test_create_tool_rejects_duplicate` — returns 409 if tool already exists
    - `test_create_tool_starts_run` — returns run_id on success

**Deliverables:**
- From the sdlc UI, fill in name + description, click "Create Tool" — agent writes the tool
- SSE progress is visible in the creation modal (same UX as existing agent runs)
- After completion, new tool appears in the Tools list and is immediately runnable
- `tools.md` is auto-updated after every tool creation
- Agents using the API can create tools programmatically without the UI

---

## Sequence Summary

```
Week 1  ─── Rust types, tool_runner, SdlcError variants, sdlc tool list stub, config show --json
Week 2  ─── _shared/types.ts + log.ts + config.ts + runtime.ts, AMA tool + README, guidance §7
Week 3  ─── quality-check tool, 4 slash commands, AGENTS.md + guidance updates, sdlc tool sync
Week 4  ─── Server routes /api/tools/*, spawn_blocking wrappers, error codes
Week 5  ─── AMA incremental index, cross-platform validation, error polish, docs
Week 6  ─── ToolsPage, AmaResultPanel, QualityCheckPanel, Sidebar nav item
Week 7  ─── sdlc tool scaffold, expanded build/audit/uat skills, smoke-tested tool creation
Week 8  ─── POST /api/tools → agent run, CreateToolModal, SSE streaming, auto tools.md refresh
```

**Every week ends with:** `SDLC_NO_NPM=1 cargo test --all` passing, `cargo clippy` clean, `npx tsc --noEmit` clean (weeks 6–8).

---

## Risk Register

| Risk | Affects | Mitigation |
|------|---------|------------|
| `globSync` not available on all runtimes | Week 2 | Use recursive `readdir` fallback; test Bun + Deno + Node before AMA ships |
| `import.meta.main` is Bun-only | Week 2 | `_shared/runtime.ts` normalizes arg access; never use `import.meta.main` in tools |
| AMA index too slow on large monorepos | Week 5 | File size guard (skip > 500KB) + incremental re-index based on mtime |
| Windows subprocess quoting differences | Week 5 | `shell: true` in all spawnSync calls; test via Node fallback path |
| Deno module system differences | Week 3 | Use only Node-compatible built-ins; avoid `Deno.` APIs in shared code |
| Agent run for tool creation produces low-quality tool | Week 8 | Prompt includes full skill content + "all 7 UAT scenarios must pass" hard requirement |
| Long tool creation runs time out | Week 8 | Set `timeout_minutes: 30` on the agent run; UI shows elapsed time |
