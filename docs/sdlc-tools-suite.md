# SDLC Tools Suite

**Status:** Proposal
**Date:** 2026-02-27

---

## Problem

sdlc manages feature lifecycle state. But the _work_ that happens inside that lifecycle — understanding the codebase, verifying quality — has no first-class home. Agents currently rely on ad-hoc tool calls and hope the right context is available. Humans have no standard entry point to run quality checks. Both problems get worse as projects grow.

---

## Vision

Every project managed by sdlc ships with a small, opinionated toolkit of **SDLC Tools** — runnable units that agents and humans can call during any phase of the feature lifecycle. The toolkit starts with two tools (AMA, Dev Quality Check), grows as the community adds more, and lets individual projects define their own. Tools are installed when you run `sdlc init` or `sdlc update` — no extra setup step.

A tool is:
- **Callable by agents** as an MCP tool (standard interface)
- **Callable by humans** via CLI (`sdlc tool run <name>`)
- **Browseable in the web UI** via the sdlc-server dashboard
- **Configurable per-project** via `.sdlc/tools/<name>/config.yaml`

---

## Initial Tools

### Tool 1: AMA (Ask Me Anything)

**Purpose:** Answer questions about the codebase quickly — without leaving the AI assistant context or burning a full agent run just to ask "where is the auth middleware?"

**What it does:**
1. On first run (or when triggered), walks the project tree and builds a searchable index of code: file paths, function/class names, docstrings, comments, import relationships
2. Maintains that index in `.sdlc/tools/ama/index/` (gitignored by default, can opt in)
3. Exposes an MCP tool `sdlc_ama(question: string) → { answer: string, sources: FileRef[] }` that searches the index and synthesizes an answer
4. Re-indexes automatically when files change (file watcher, debounced)
5. Supports incremental updates — changed files are re-indexed, not the whole tree

**Example agent use:**
```
sdlc_ama("Where is JWT token validation implemented?")
→ { answer: "JWT validation lives in src/middleware/auth.rs lines 45-89...", sources: [...] }
```

**Example human use:**
```bash
sdlc tool run ama "how does the ponder session logging work?"
```

---

### Tool 2: Dev Quality Check

**Purpose:** Run the project's quality gates on demand — exactly the checks that sdlc would run at phase boundaries — without triggering a full sdlc phase transition.

**What it does:**
1. Reads shell gates defined in `.sdlc/config.yaml` (under `gates:` → each action's `shell` entries)
2. Optionally accepts a `scope` (specific gate action, e.g. `implement_task`) to run only relevant checks
3. Runs each check, streams output, collects pass/fail
4. Returns structured results agents can act on
5. In the web UI, shows a live output panel with expandable check results

**Example agent use:**
```
sdlc_quality_check({ scope: "implement_task" })
→ { passed: 2, failed: 1, checks: [{ name: "test", status: "failed", output: "..." }] }
```

**Example human use:**
```bash
sdlc tool run quality-check
sdlc tool run quality-check --scope implement_task
```

---

## Architecture

### Tool Contract

Every SDLC Tool is a directory under `.sdlc/tools/<name>/` with:

```
.sdlc/tools/ama/
├── config.yaml          # tool-specific config (include/exclude patterns, model, etc.)
├── tool.{ext}           # the runnable tool script/binary entrypoint
└── index/               # tool's persistent state (gitignored by default)
```

Tools expose two interfaces:
1. **MCP tool** — the `sdlc-server` embeds an MCP endpoint at `/mcp` that proxies calls to each installed tool's `run(input)` handler
2. **CLI** — `sdlc tool run <name> [args...]` invokes the tool directly

### Installation

`sdlc init` and `sdlc update` write the core tool scripts into `.sdlc/tools/`. This is the same pattern sdlc already uses for installing slash commands via `install_user_scaffolding()` — tools are just files written to a well-known path.

The tool runtime (whatever language executes the scripts) must already be present, or sdlc falls back gracefully with a "install X to enable tools" message.

### Extensibility

Projects add custom tools by:
1. Creating `.sdlc/tools/<name>/tool.{ext}` following the tool contract
2. Adding `<name>` to the `tools` list in `.sdlc/config.yaml`

The community can publish tools as npm packages, Deno modules, or crates — sdlc just writes the entrypoint script that loads them.

---

## User Stories

**Agent during `implement_task`:**
> "Before I start, I need to know where the existing auth logic lives."
> → calls `sdlc_ama("where is auth logic?")` — gets answer in 200ms without reading 50 files

**Agent after writing code:**
> "Check if this passes the quality gates."
> → calls `sdlc_quality_check()` — gets structured pass/fail without invoking a full sdlc phase transition

**Human developer:**
> "Just run the checks before I commit."
> → `sdlc tool run quality-check` — same checks, direct output

**Agent installing into a new project:**
> `sdlc init` — tools are written automatically, zero configuration needed for defaults

---

## Non-Goals

- **Not a full RAG / embedding pipeline for v1** — AMA v1 uses fast keyword + symbol search; semantic embeddings are an enhancement
- **Not a replacement for the pre-commit hook runner** — Dev Quality Check is an on-demand wrapper; it doesn't replace the hooks themselves
- **Not a plugin marketplace** — tools are files in the repo; distribution is a future concern
- **Not LLM-in-the-loop for quality checking** — Dev Quality Check runs shell commands, not AI analysis

---

## Success Metrics

- AMA answers a "where is X?" question in < 500ms after initial indexing
- Dev Quality Check returns results in the time it takes to run the underlying shell commands (no overhead)
- `sdlc init` in a new project produces working tools with zero additional commands
- A new tool can be written and registered in < 30 minutes following the contract
