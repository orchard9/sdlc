// Non-command static string content used by `sdlc init` and `sdlc update`.
// These are platform script templates, the AI lookup index, engineering
// guidance, and the core tool suite TypeScript/YAML files.

// ---------------------------------------------------------------------------
// Platform script templates (masquerade)
// ---------------------------------------------------------------------------

pub const MASQ_DEPLOY_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform deploy <service> <environment>
set -e

SERVICE="$1"
ENVIRONMENT="$2"

echo "Deploying $SERVICE to $ENVIRONMENT..."
# TODO: wire up real deploy command
# Example: kubectl set image deployment/$SERVICE $SERVICE=$REGISTRY/$SERVICE:latest -n $ENVIRONMENT
echo "Deploy complete: $SERVICE -> $ENVIRONMENT"
"#;

pub const MASQ_LOGS_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform logs [service]
set -e

SERVICE="${1:-}"

if [ -n "$SERVICE" ]; then
    echo "Fetching logs for $SERVICE..."
    # TODO: kubectl logs -n production deployment/$SERVICE --tail=100 -f
else
    echo "Fetching logs for all services..."
    # TODO: kubectl logs -n production --all-containers=true
fi
"#;

pub const MASQ_DEV_START_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform dev start
set -e

echo "Starting development environment..."
# TODO: docker compose up -d
echo "Dev environment started."
"#;

pub const MASQ_DEV_STOP_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform dev stop
set -e

echo "Stopping development environment..."
# TODO: docker compose down
echo "Dev environment stopped."
"#;

pub const MASQ_DEV_QUALITY_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform dev quality
set -e

echo "Running quality checks..."
# TODO: run linters, type checks, and unit tests
echo "Quality checks complete."
"#;

pub const MASQ_DEV_MIGRATE_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform dev migrate
set -e

echo "Running database migrations..."
# TODO: run migration tool against local dev database
echo "Migrations complete."
"#;

// ---------------------------------------------------------------------------
// Static file content
// ---------------------------------------------------------------------------

pub const AI_LOOKUP_INDEX_CONTENT: &str = r#"# .ai Index

Project knowledge base. Entries are organized by category.

## Categories

- **patterns/** — How we do things (coding patterns, architectural conventions)
- **decisions/** — Why we chose X over Y (ADRs, trade-off notes)
- **gotchas/** — Non-obvious pitfalls and workarounds
- **architecture/** — How the system works (data flow, component relationships)
- **conventions/** — Naming, style, standards

## Usage

Entries are harvested automatically after each SDLC artifact is approved.
Each entry follows the format:

```
---
category: patterns
title: How we handle X
learned: YYYY-MM-DD
source: spec|design|review|human
confidence: high|medium|low
---

## Summary
...

## Key Facts
- ...

## File Pointer
`path/to/file.go:line-range`
```
"#;

pub const GUIDANCE_MD_CONTENT: &str = r#"# Engineering Guidance

Read this before any implementation, bug fix, or test action.

## North Star: Vision & Architecture

Before writing a single line of code, read:

- **`VISION.md`** — *what* we are building and *why*. Every feature, every tradeoff, every design decision must serve this vision. If a proposed change works against it, surface it before proceeding.
- **`ARCHITECTURE.md`** — *how* the system works. Components, interfaces, data flows, and sequence diagrams showing how everything fits together. Code must conform to the architecture — never silently deviate.

These are the guiding light. When in doubt about any decision, return to them first.

## 1. Build It Right

Do it the proper way — not the quick way. The correct solution is one that
will still be correct in six months. Favor proven patterns, clear
abstractions, and designs that are easy to understand and extend. Never
trade long-term correctness for short-term convenience.

## 2. Understand Bugs Before Fixing Them

Before touching a bug, trace its root cause holistically — read surrounding
code, follow the data flow, understand why it broke. Fix the cause, not the
symptom. A patch that introduces a new bug in three months is worse than
no fix.

## 3. Enterprise Quality Bar

We build enterprise-grade software. The bar is Steve Jobs: relentless
attention to detail, nothing ships that embarrasses us, correctness and
reliability are non-negotiable. If something isn't right, make it right.

## 4. Philosophy of Software Design

Follow John Ousterhout's principles: deep modules, minimal exposed
complexity, interfaces that hide implementation detail, and code readable
in isolation. Complexity is the enemy — fight it at every level.

## 5. Meaningful, Reliable, Fast Tests

Tests must earn their place. When a test breaks, choose deliberately:
- **Remove** — if it adds little value or tests implementation detail
- **Rewrite** — if it was poorly structured for the scenario
- **Refactor** — if the interface it tests changed legitimately
- **Quick-fix** — only if the fix is obvious and the test is clearly valuable

Never keep a flaky or low-value test just to preserve coverage numbers.

## 6. Using sdlc

All state lives in `.sdlc/` YAML files. **Never edit them directly** — use the CLI.
Direct edits cause deserialization failures and corrupt state.

| Action | Command |
|---|---|
| Create feature | `sdlc feature create <slug> --title "…"` |
| Get next action | `sdlc next --for <slug> --json` |
| Write artifact | Write Markdown to `output_path` from the directive |
| Submit draft | `sdlc artifact draft <slug> <type>` |
| Approve artifact | `sdlc artifact approve <slug> <type>` |
| Reject artifact | `sdlc artifact reject <slug> <type>` |
| Merge (release feature) | `sdlc merge <slug>` |
| Add task | `sdlc task add <slug> "title"` |
| Start task | `sdlc task start <slug> <task-id>` |
| Complete task | `sdlc task complete <slug> <task-id>` |
| Block task | `sdlc task block <slug> <task-id> "reason"` |
| Add comment | `sdlc comment create <slug> "body"` |
| Show feature | `sdlc feature show <slug> --json` |
| List tasks | `sdlc task list <slug>` |
| Project state | `sdlc state` |
| Survey milestone waves | `sdlc project prepare [--milestone <slug>]` |
| Mark milestone prepared | `sdlc milestone mark-prepared <slug>` |
| Project phase | `sdlc project status` |
| Escalate to human | `sdlc escalate create --kind <kind> --title "…" --context "…" [--feature <slug>]` |
| List escalations | `sdlc escalate list` |
| Resolve escalation | `sdlc escalate resolve <id> "resolution note"` |
| Knowledge base status | `sdlc knowledge status` |
| List knowledge entries | `sdlc knowledge list [--code-prefix <code>]` |
| Search knowledge base | `sdlc knowledge search <query>` |
| Show knowledge entry | `sdlc knowledge show <slug>` |
| Add knowledge entry | `sdlc knowledge add --title "..." --code <code> --content "..."` |
| Show catalog taxonomy | `sdlc knowledge catalog show` |
| Seed from workspaces | `sdlc knowledge librarian init` |

Phases advance automatically from artifact approvals — never call `sdlc feature transition`.
The only files you write directly are Markdown artifacts to `output_path`.

## 7. SDLC Tool Suite

Project-scoped TypeScript tools in `.sdlc/tools/` — callable by agents and humans during any lifecycle phase.
Read `.sdlc/tools/tools.md` for the full list, or each tool's `README.md` for detailed docs.

| Tool | Command | Purpose |
|---|---|---|
| ama | `sdlc tool run ama --setup` then `sdlc tool run ama --question "..."` | Search codebase for relevant file excerpts |

Build a custom tool: `sdlc tool scaffold <name> "<description>"`
Update the manifest after adding/changing tools: `sdlc tool sync`

## 8. Project Secrets

Encrypted secrets live in `.sdlc/secrets/`. The encrypted files (`.age`) and key
name sidecars (`.meta.yaml`) are **safe to commit**. Plain `.env.*` files must never
be committed — they are gitignored automatically.

| Action | Command |
|---|---|
| List environments | `sdlc secrets env list` |
| List key names (no decrypt) | `sdlc secrets env names <env>` |
| Load secrets into shell | `eval $(sdlc secrets env export <env>)` |
| Set a secret | `sdlc secrets env set <env> KEY=value` |
| List authorized keys | `sdlc secrets keys list` |
| Add a key | `sdlc secrets keys add --name <n> --key "$(cat ~/.ssh/id_ed25519.pub)"` |
| Rekey after key change | `sdlc secrets keys rekey` |

**For agents:** Check `sdlc secrets env names <env>` to see which variables are
available. Load the matching env before any task or build step that needs credentials:
- Feature/local work → `eval $(sdlc secrets env export development)`
- Deploy tasks → `eval $(sdlc secrets env export production)`

Never log or hardcode secret values. Reference by env var name only (e.g. `$ANTHROPIC_API_KEY`).

**In builds:** The vault is for local and agent use only. CI/CD platforms (GitHub Actions,
etc.) manage their own secrets separately — agents cannot inject into platform CI secrets.
If a build needs a credential that must live in CI, use `secret_request` escalation (§9).

## 9. Escalating to the Human

Escalations are for **actions only a human can take**. They are rare and deliberate — not a
general-purpose communication channel. Before escalating, ask: "Can I resolve this myself?"
If yes, do it. If not, escalate.

| Kind | When to escalate | Example |
|---|---|---|
| `secret_request` | Need a credential or env var that doesn't exist | "Add STRIPE_API_KEY to production env in Secrets page" |
| `question` | Strategic decision with no clear right answer | "Should checkout support crypto payments?" |
| `vision` | Product direction is undefined or contradictory | "No vision defined — what is the milestone goal?" |
| `manual_test` | Testing requires physical interaction | "Verify Google OAuth login in production browser" |

**Do NOT escalate:** code review findings, spec ambiguity you can resolve, implementation
decisions, anything an agent can handle autonomously.

**How to escalate:**

```bash
sdlc escalate create \
  --kind secret_request \
  --title "Need OPENAI_API_KEY in .env.production" \
  --context "AI summary feature calls OpenAI in prod. Dev works with a mock. Need the real key to test end-to-end." \
  --feature my-ai-feature   # omit if not feature-specific
```

**After creating:** stop the current run immediately. If `--feature` was specified, the feature
is now gated by an auto-added Blocker comment. The escalation appears in the Dashboard under
**"Needs Your Attention"**. The human must act before the feature can proceed.

**The difference from `comment --flag blocker`:**

- `comment --flag blocker` — an implementation concern the next agent cycle might fix
- `sdlc escalate create` — an action only a human can perform; stop until resolved

## 10. Frontend API Calls

Never hardcode `http://localhost:PORT` in frontend code — CORS blocks cross-origin
requests in development and the address is wrong in production.

**Pattern:**
- Use a relative base URL (`/api`) in all fetch/client code
- Configure the dev server proxy (Vite `server.proxy`, Next.js `rewrites`,
  webpack `devServer.proxy`) to forward `/api` → `http://localhost:<API_PORT>`
- In production, frontend and API share the same origin — relative paths resolve correctly

When fixing a CORS error or adding a new API client, apply this pattern instead of
adding CORS headers or introducing environment-specific URLs.

## 11. Production Safety

This is a live system with real users. Every change must leave the codebase healthier — not just correct, but cleaner.

**Migrations:** Add defensive deserialization before removing old formats. Never the reverse. Test that both old and new formats load cleanly before shipping.

**Stability hazards to avoid:**
- Infinite loops: any polling, retry, or SSE reconnect loop must have a termination condition and backoff
- Connection exhaustion: SSE subscriptions, DB connections, and broadcast channels must be bounded and cleaned up on drop
- Complex failure modes: prefer simple, flat control flow over deeply nested async chains — when it breaks at 3am, you must be able to read the trace

**Quality bar:** if a change makes the code harder to reason about, makes logs less useful, or adds a failure mode with no clear recovery path — stop and reconsider. Simpler is always better.

## 12. Project Guidelines

Before writing implementation code, check if `.sdlc/guidelines/index.yaml` exists.
If it does, read it and load any guidelines whose `scope` overlaps with the work at hand.

```bash
# Check
ls .sdlc/guidelines/index.yaml 2>/dev/null && cat .sdlc/guidelines/index.yaml
```

Guidelines contain `⚑ Rule:` statements with `✓ Good:` and `✗ Bad:` code examples derived
from this codebase. They are authoritative — if your implementation would violate a rule,
fix the approach before proceeding, not after review catches it.

If no index exists, no guidelines have been published yet. Proceed normally.
"#;

// ---------------------------------------------------------------------------
// Tool Suite TypeScript content
// ---------------------------------------------------------------------------

pub const TOOL_SHARED_TYPES_TS: &str = r#"/**
 * SDLC Tool Shared Interface
 *
 * Every SDLC tool imports from this file. It defines the full type contract
 * that tools must satisfy. Do not change the shape of these types without
 * updating all core tools and regenerating tools.md.
 *
 * Tool protocol (stdin/stdout):
 * - --meta   No stdin. Writes ToolMeta JSON to stdout.
 * - --run    Reads JSON from stdin. Writes ToolResult JSON to stdout. Exit 0 ok, 1 error.
 * - --setup  No stdin. Writes ToolResult JSON to stdout. Exit 0 ok, 1 error.
 *
 * All log output goes to STDERR. STDOUT is reserved for JSON only.
 */

/** Metadata describing a tool — returned by --meta mode. */
export interface ToolMeta {
  /** Matches the directory name exactly (e.g. "ama", "quality-check") */
  name: string
  /** Human-readable title shown in the tools list */
  display_name: string
  /** One sentence, present tense, no trailing period */
  description: string
  /** Semver, mirrors sdlc binary version at install time */
  version: string
  /** JSON Schema describing valid input for --run */
  input_schema: JsonSchema
  /** JSON Schema describing the data field in ToolResult */
  output_schema: JsonSchema
  /** True if --setup must run before first --run */
  requires_setup: boolean
  /** One sentence describing what setup does (required if requires_setup = true) */
  setup_description?: string
}

/** The result envelope returned by --run and --setup modes. */
export interface ToolResult<T = unknown> {
  ok: boolean
  data?: T
  /** Present only when ok = false */
  error?: string
  /** Wall-clock milliseconds for the operation */
  duration_ms?: number
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type JsonSchema = Record<string, any>
"#;

pub const TOOL_SHARED_LOG_TS: &str = r#"/**
 * Standard SDLC Tool Logger
 *
 * Writes structured log lines to STDERR (never stdout — stdout is reserved
 * for JSON output). Use this in every tool to produce consistent, parseable logs.
 *
 * Format: [sdlc-tool:<name>] LEVEL: message
 * Example: [sdlc-tool:ama] INFO:  Indexed 312 files in 842ms
 *
 * Set SDLC_TOOL_DEBUG=1 to enable debug-level output.
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
"#;

pub const TOOL_SHARED_CONFIG_TS: &str = r#"/**
 * SDLC Tool Config Loader
 *
 * Reads .sdlc/tools/<name>/config.yaml. If the file is missing or unparseable,
 * returns the provided defaults — tools should never hard-fail on missing config.
 *
 * Supports flat key: value YAML only. Arrays and nested objects are intentionally
 * not supported — keep tool configs simple scalars.
 */
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

export function loadToolConfig<T extends Record<string, unknown>>(
  root: string,
  toolName: string,
  defaults: T,
): T {
  const configPath = join(root, '.sdlc', 'tools', toolName, 'config.yaml')
  try {
    const raw = readFileSync(configPath, 'utf8')
    const parsed = parseSimpleYaml(raw)
    return { ...defaults, ...parsed } as T
  } catch {
    return defaults
  }
}

/** Parse a flat key: value YAML file. Skips blank lines, comments, and array items. */
function parseSimpleYaml(content: string): Record<string, unknown> {
  const result: Record<string, unknown> = {}
  for (const line of content.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('-')) continue
    const colonIdx = trimmed.indexOf(':')
    if (colonIdx === -1) continue
    const key = trimmed.slice(0, colonIdx).trim()
    const rawValue = trimmed.slice(colonIdx + 1).trim()
    if (!key || !rawValue) continue
    const value = rawValue.replace(/^["'](.*)["']$/, '$1')
    const num = Number(value)
    result[key] = Number.isNaN(num) ? value : num
  }
  return result
}
"#;

pub const TOOL_SHARED_RUNTIME_TS: &str = r#"/**
 * Cross-runtime helpers for Bun, Deno, and Node.
 *
 * Normalizes: argv access, stdin reading, env access, and process exit
 * across the three supported runtimes.
 *
 * Detection: checks for globalThis.Deno to identify Deno; falls back
 * to process (Node.js / Bun).
 */

/* eslint-disable @typescript-eslint/no-explicit-any */

/** Returns command-line arguments after the script name (process.argv[2+]). */
export function getArgs(): string[] {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    return [...(globalThis as any).Deno.args]
  }
  return process.argv.slice(2)
}

/** Read all of stdin as a UTF-8 string. Returns empty string if stdin is a TTY or closed. */
export async function readStdin(): Promise<string> {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    const chunks: Uint8Array[] = []
    const reader = (globalThis as any).Deno.stdin.readable.getReader()
    try {
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        chunks.push(value)
      }
    } finally {
      reader.releaseLock()
    }
    const total = chunks.reduce((sum: number, c: Uint8Array) => sum + c.length, 0)
    const merged = new Uint8Array(total)
    let offset = 0
    for (const chunk of chunks) {
      merged.set(chunk, offset)
      offset += chunk.length
    }
    return new TextDecoder().decode(merged)
  }
  // Node.js / Bun
  if ((process.stdin as any).isTTY) return ''
  const chunks: Buffer[] = []
  for await (const chunk of process.stdin) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk))
  }
  return Buffer.concat(chunks).toString('utf8')
}

/** Get a process environment variable. Works across Bun, Deno, and Node. */
export function getEnv(key: string): string | undefined {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    return (globalThis as any).Deno.env.get(key)
  }
  return process.env[key]
}

/** Exit the process with the given code. */
export function exit(code: number): never {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    ;(globalThis as any).Deno.exit(code)
  }
  process.exit(code)
  throw new Error('unreachable')
}
"#;

pub const TOOL_SHARED_AGENT_TS: &str = include_str!("../../../../../.sdlc/tools/_shared/agent.ts");

pub const TOOL_AMA_TS: &str = r#"/**
 * AMA — Ask Me Anything
 * =====================
 * Answers questions about the codebase by searching a pre-built keyword index.
 *
 * WHAT IT DOES
 * ------------
 * --setup:  Walks all source files matching configured extensions. On first run,
 *           indexes every file. On subsequent runs, skips unchanged files (mtime
 *           check), re-indexes changed/new files, and prunes deleted files.
 *           Writes chunks.json (TF-IDF index) and last_indexed.json (mtime map).
 *           Re-running --setup is always safe (incremental or full).
 *
 * --run:    Reads JSON from stdin: { "question": "string" }
 *           Loads the TF-IDF index, scores chunks by IDF-weighted keyword overlap,
 *           returns top results as source excerpts with relevance scores.
 *           Sources from files changed since last indexing are flagged stale.
 *
 * --meta:   Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 *
 * WHAT IT READS
 * -------------
 * - .sdlc/tools/ama/config.yaml                (extensions, chunk settings)
 * - .sdlc/tools/ama/index/chunks.json          (built by --setup)
 * - .sdlc/tools/ama/index/last_indexed.json    (mtime map; built by --setup)
 * - Source files matching config.extensions    (during --setup only)
 *
 * WHAT IT WRITES
 * --------------
 * - .sdlc/tools/ama/index/chunks.json          (during --setup; TF-IDF index)
 * - .sdlc/tools/ama/index/last_indexed.json    (during --setup; mtime map for incremental re-runs)
 * - STDERR: structured log lines via _shared/log.ts
 * - STDOUT: JSON only (ToolResult shape from _shared/types.ts)
 *
 * EXTENDING
 * ---------
 * Replace scoreChunks() with embedding-based cosine similarity to improve answer
 * quality. The rest of the pipeline (chunking, index format, protocol) stays the same.
 *
 * For LLM synthesis: call the Claude API in run() with the top excerpts as context.
 * Add "synthesis_model" to config.yaml to control which model is used.
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { loadToolConfig } from '../_shared/config.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import {
  readdirSync, readFileSync, writeFileSync, mkdirSync, statSync, existsSync,
} from 'node:fs'
import { join, extname, relative } from 'node:path'

const log = makeLogger('ama')

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

interface AmaConfig {
  chunk_lines: number
  chunk_overlap: number
  max_results: number
  max_file_kb: number
  extensions: string
}

const DEFAULT_CONFIG: AmaConfig = {
  chunk_lines: 40,
  chunk_overlap: 5,
  max_results: 5,
  max_file_kb: 500,
  extensions: '.ts,.js,.tsx,.jsx,.rs,.go,.py,.rb,.java,.md,.txt,.yaml,.yml,.toml',
}

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'ama',
  display_name: 'AMA — Ask Me Anything',
  description: 'Answers questions about the codebase using a pre-built TF-IDF keyword index',
  version: '0.2.1',
  requires_setup: true,
  setup_description: 'Indexes source files for keyword search (first run is full index; subsequent runs are incremental)',
  input_schema: {
    type: 'object',
    required: ['question'],
    properties: {
      question: { type: 'string', description: 'The question to answer about the codebase' },
    },
  },
  output_schema: {
    type: 'object',
    properties: {
      sources: {
        type: 'array',
        items: {
          type: 'object',
          properties: {
            path: { type: 'string' },
            lines: { type: 'array', items: { type: 'number' }, minItems: 2, maxItems: 2 },
            excerpt: { type: 'string' },
            score: { type: 'number', description: 'TF-IDF relevance score (0.0–1.0)' },
            stale: { type: 'boolean', description: 'True if the source file changed since last index run' },
          },
        },
      },
    },
  },
}

// ---------------------------------------------------------------------------
// Index types
// ---------------------------------------------------------------------------

interface Chunk {
  path: string
  start: number
  end: number
  tokens: string[]
}

interface Index {
  version: number
  generated: string
  chunks: Chunk[]
  idf: Record<string, number>
}

interface MtimeMap {
  version: number
  indexed_at: string
  files: Record<string, number>
}

interface AmaSource {
  path: string
  lines: [number, number]
  excerpt: string
  score: number
  stale?: boolean
}

interface AmaOutput {
  sources: AmaSource[]
}

// ---------------------------------------------------------------------------
// Setup — build the keyword index
// ---------------------------------------------------------------------------

export async function setup(root: string): Promise<ToolResult<{
  files_indexed: number
  files_skipped: number
  files_pruned: number
  chunks_written: number
  total_chunks: number
  duration_ms: number
  index_size_kb: number
}>> {
  const start = Date.now()
  const config = loadToolConfig(root, 'ama', DEFAULT_CONFIG)
  const extensions = new Set(
    String(config.extensions).split(',').map(e => e.trim()).filter(Boolean),
  )

  const indexDir = join(root, '.sdlc', 'tools', 'ama', 'index')
  mkdirSync(indexDir, { recursive: true })

  const chunksPath = join(indexDir, 'chunks.json')
  const mtimePath = join(indexDir, 'last_indexed.json')

  // Load previous index and mtime map for incremental re-indexing
  let prevChunks: Chunk[] = []
  let prevMtimes: Record<string, number> = {}
  const isIncremental = existsSync(chunksPath) && existsSync(mtimePath)
  if (isIncremental) {
    try {
      const prevIndex = JSON.parse(readFileSync(chunksPath, 'utf8')) as Index
      prevChunks = prevIndex.chunks ?? []
      const mtimeData = JSON.parse(readFileSync(mtimePath, 'utf8')) as MtimeMap
      prevMtimes = mtimeData.files ?? {}
      log.info(`incremental mode: ${prevChunks.length} existing chunks, ${Object.keys(prevMtimes).length} tracked files`)
    } catch {
      log.warn('could not load previous index — falling back to full re-index')
      prevChunks = []
      prevMtimes = {}
    }
  } else {
    log.info('full index mode (no previous index found)')
  }

  log.info(`indexing with extensions: ${[...extensions].join(', ')}`)

  const allFiles = walkFiles(root, extensions, Number(config.max_file_kb))
  log.info(`found ${allFiles.length} files to consider`)

  // Group previous chunks by file for efficient lookup
  const prevChunksByFile = new Map<string, Chunk[]>()
  for (const chunk of prevChunks) {
    const arr = prevChunksByFile.get(chunk.path) ?? []
    arr.push(chunk)
    prevChunksByFile.set(chunk.path, arr)
  }

  const newMtimes: Record<string, number> = {}
  const unchangedChunks: Chunk[] = []
  const freshChunks: Chunk[] = []
  let filesSkipped = 0
  let filesIndexed = 0

  for (const filePath of allFiles) {
    const relPath = relative(root, filePath)
    const mtime = statSync(filePath).mtimeMs
    if (isIncremental && prevMtimes[relPath] === mtime) {
      unchangedChunks.push(...(prevChunksByFile.get(relPath) ?? []))
      newMtimes[relPath] = mtime
      filesSkipped++
    } else {
      try {
        const content = readFileSync(filePath, 'utf8')
        const fileChunks = chunkFile(relPath, content, Number(config.chunk_lines), Number(config.chunk_overlap))
        freshChunks.push(...fileChunks)
        newMtimes[relPath] = mtime
        filesIndexed++
      } catch (e) {
        log.warn(`skipping ${relPath}: ${e}`)
      }
    }
  }

  // Count pruned files (tracked before but no longer on disk)
  const currentPaths = new Set(allFiles.map(f => relative(root, f)))
  const filesPruned = Object.keys(prevMtimes).filter(p => !currentPaths.has(p)).length
  if (filesPruned > 0) log.info(`pruned ${filesPruned} deleted/moved file(s) from index`)

  const allChunks = [...unchangedChunks, ...freshChunks]
  log.info(`${filesIndexed} indexed, ${filesSkipped} skipped, ${filesPruned} pruned — ${allChunks.length} total chunks`)

  // Compute smoothed IDF: log((N+1)/(df+1)) + 1 for each term
  const N = allChunks.length
  const df: Record<string, number> = {}
  for (const chunk of allChunks) {
    for (const token of chunk.tokens) {
      df[token] = (df[token] ?? 0) + 1
    }
  }
  const idf: Record<string, number> = {}
  for (const [term, freq] of Object.entries(df)) {
    idf[term] = Math.log((N + 1) / (freq + 1)) + 1
  }

  // Write index and mtime map
  const index: Index = { version: 2, generated: new Date().toISOString(), chunks: allChunks, idf }
  const indexJson = JSON.stringify(index)
  writeFileSync(chunksPath, indexJson)

  const mtimeMap: MtimeMap = { version: 1, indexed_at: new Date().toISOString(), files: newMtimes }
  writeFileSync(mtimePath, JSON.stringify(mtimeMap))

  const duration_ms = Date.now() - start
  const index_size_kb = Math.round(indexJson.length / 1024)
  log.info(`done in ${duration_ms}ms — index size: ${index_size_kb}KB`)

  return {
    ok: true,
    data: {
      files_indexed: filesIndexed,
      files_skipped: filesSkipped,
      files_pruned: filesPruned,
      chunks_written: freshChunks.length,
      total_chunks: allChunks.length,
      duration_ms,
      index_size_kb,
    },
    duration_ms,
  }
}

// ---------------------------------------------------------------------------
// Run — answer a question using the index
// ---------------------------------------------------------------------------

export async function run(
  input: { question?: string },
  root: string,
): Promise<ToolResult<AmaOutput>> {
  const start = Date.now()
  const config = loadToolConfig(root, 'ama', DEFAULT_CONFIG)

  const question = input.question?.trim()
  if (!question) {
    return { ok: false, error: 'input.question is required' }
  }

  const indexPath = join(root, '.sdlc', 'tools', 'ama', 'index', 'chunks.json')
  if (!existsSync(indexPath)) {
    return {
      ok: false,
      error: 'Index not built. Run setup first: sdlc tool run ama --setup',
    }
  }

  let index: Index
  try {
    index = JSON.parse(readFileSync(indexPath, 'utf8')) as Index
  } catch (e) {
    return { ok: false, error: `Failed to load index: ${e}. Re-run: sdlc tool run ama --setup` }
  }

  // Load mtime map for stale source detection (non-fatal if absent)
  let mtimes: Record<string, number> = {}
  try {
    const mtimePath = join(root, '.sdlc', 'tools', 'ama', 'index', 'last_indexed.json')
    if (existsSync(mtimePath)) {
      mtimes = (JSON.parse(readFileSync(mtimePath, 'utf8')) as MtimeMap).files ?? {}
    }
  } catch { /* stale detection skipped */ }

  log.info(`scoring ${index.chunks.length} chunks for: "${question}"`)

  // idf falls back gracefully to 1.0 weights for v1 indexes without IDF
  const idf = index.idf ?? {}
  const topChunks = scoreChunks(question, index.chunks, idf).slice(0, Number(config.max_results))

  const sources: AmaSource[] = []
  for (const { chunk, score } of topChunks) {
    const fullPath = join(root, chunk.path)
    try {
      const lines = readFileSync(fullPath, 'utf8').split('\n')
      const excerpt = lines.slice(chunk.start - 1, chunk.end).join('\n')

      // Stale detection: flag if file changed since last index run
      let stale = false
      try {
        if (mtimes[chunk.path] !== undefined && statSync(fullPath).mtimeMs !== mtimes[chunk.path]) {
          stale = true
          log.warn(`stale source: ${chunk.path} changed since last index run`)
        }
      } catch { /* file may not exist — handled above */ }

      const source: AmaSource = { path: chunk.path, lines: [chunk.start, chunk.end], excerpt, score }
      if (stale) source.stale = true
      sources.push(source)
    } catch {
      log.warn(`skipping deleted/moved file: ${chunk.path}`)
    }
  }

  const duration_ms = Date.now() - start
  log.info(`returned ${sources.length} sources in ${duration_ms}ms`)

  return { ok: true, data: { sources }, duration_ms }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

const SKIP_DIRS = new Set([
  'node_modules', '.git', 'target', 'dist', 'build', '.sdlc',
  '.next', '__pycache__', '.cache', 'coverage',
])

function walkFiles(root: string, extensions: Set<string>, maxFileKb: number): string[] {
  const results: string[] = []

  function walk(dir: string) {
    let entries: ReturnType<typeof readdirSync>
    try {
      entries = readdirSync(dir, { withFileTypes: true })
    } catch {
      return
    }
    for (const entry of entries) {
      if (entry.name.startsWith('.')) continue
      const full = join(dir, entry.name)
      if (entry.isDirectory()) {
        if (!SKIP_DIRS.has(entry.name)) walk(full)
      } else if (entry.isFile()) {
        if (!extensions.has(extname(entry.name))) continue
        try {
          if (statSync(full).size > maxFileKb * 1024) {
            log.warn(`skipping large file (${Math.round(statSync(full).size / 1024)}KB): ${relative(root, full)}`)
            continue
          }
        } catch {
          continue
        }
        results.push(full)
      }
    }
  }

  walk(root)
  return results
}

function chunkFile(
  relPath: string,
  content: string,
  chunkLines: number,
  overlap: number,
): Chunk[] {
  const lines = content.split('\n')
  const chunks: Chunk[] = []
  const step = Math.max(1, chunkLines - overlap)

  for (let i = 0; i < lines.length; i += step) {
    const start = i + 1 // 1-based line numbers
    const end = Math.min(i + chunkLines, lines.length)
    const tokens = extractTokens(lines.slice(i, end).join(' '))
    if (tokens.length > 0) {
      chunks.push({ path: relPath, start, end, tokens })
    }
    if (end >= lines.length) break
  }

  return chunks
}

/**
 * Extract lowercase tokens from text, splitting on camelCase and snake_case
 * boundaries to enable code-aware search. Words < 4 chars are omitted as noise.
 *
 * Examples:
 *   featureTransition → ['feature', 'transition']
 *   SdlcError         → ['sdlc', 'error']
 *   auth_token        → ['auth', 'token']
 *   authenticate      → ['authenticate']
 */
function extractTokens(text: string): string[] {
  // Split on camelCase and acronym boundaries before lowercasing
  const expanded = text
    .replace(/([a-z])([A-Z])/g, '$1 $2')        // camelCase → camel Case
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')  // XMLParser → XML Parser
  const seen = new Set<string>()
  const tokens: string[] = []
  for (const word of expanded.toLowerCase().split(/[^a-z0-9]+/)) {
    if (word.length >= 3 && !seen.has(word)) {
      seen.add(word)
      tokens.push(word)
    }
  }
  return tokens
}

/**
 * Score chunks using TF-IDF weighted overlap.
 * IDF is precomputed at index time (stored in chunks.json v2+).
 * Falls back to uniform weights (raw overlap) for v1 indexes without IDF.
 */
function scoreChunks(
  question: string,
  chunks: Chunk[],
  idf: Record<string, number>,
): { chunk: Chunk; score: number }[] {
  const queryTokens = extractTokens(question)
  if (queryTokens.length === 0) return []

  const hasIdf = Object.keys(idf).length > 0
  const results: { chunk: Chunk; score: number }[] = []

  for (const chunk of chunks) {
    const chunkSet = new Set(chunk.tokens)
    let score = 0
    let totalWeight = 0

    for (const token of queryTokens) {
      const weight = hasIdf ? (idf[token] ?? 1.0) : 1.0
      totalWeight += weight
      if (chunkSet.has(token)) score += weight
    }

    if (score > 0) {
      results.push({ chunk, score: totalWeight > 0 ? score / totalWeight : 0 })
    }
  }

  return results.sort((a, b) => b.score - a.score)
}

// ---------------------------------------------------------------------------
// CLI entrypoint
// ---------------------------------------------------------------------------

const mode = getArgs()[0] ?? '--run'
const root = process.env.SDLC_ROOT ?? process.cwd()

if (mode === '--meta') {
  console.log(JSON.stringify(meta))
  exit(0)
} else if (mode === '--setup') {
  setup(root)
    .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
    .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
} else if (mode === '--run') {
  readStdin()
    .then(raw => run(JSON.parse(raw || '{}') as { question?: string }, root))
    .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
    .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta, --setup, or --run.`)
  exit(1)
}
"#;

pub const TOOL_AMA_CONFIG_YAML: &str = r#"name: ama
version: 0.1.0
description: Answers questions about the codebase using a pre-built keyword index

# File extensions to include in the index (comma-separated)
extensions: .ts,.js,.tsx,.jsx,.rs,.go,.py,.rb,.java,.md,.txt,.yaml,.yml,.toml

# Number of lines per chunk
chunk_lines: 40

# Lines of overlap between consecutive chunks (reduces missed context at boundaries)
chunk_overlap: 5

# Maximum results to return per query
max_results: 5

# Skip files larger than this size (kilobytes)
max_file_kb: 500
"#;

pub const TOOL_AMA_README_MD: &str = r#"# AMA — Ask Me Anything

Answers questions about the codebase by searching a pre-built keyword index.

## Setup (run once)

```bash
sdlc tool run ama --setup
```

## Usage

```bash
sdlc tool run ama --question "where is JWT validation?"
sdlc tool run ama --question "how does feature transition work?"
```

## How it works

1. `--setup` walks source files, chunks them into 40-line windows, extracts keyword tokens,
   and writes `.sdlc/tools/ama/index/chunks.json`
2. `--run` scores chunks by keyword overlap with your question, returns top file excerpts
3. Your AI assistant reads the excerpts and synthesizes an answer

## Configuration

Edit `.sdlc/tools/ama/config.yaml` to change which file extensions are indexed
or to adjust chunk size, overlap, and result count.

## Index location

`.sdlc/tools/ama/index/chunks.json` — gitignored, regenerate with `--setup`

## Re-index when needed

Re-run `--setup` after significant file changes. It's fast and safe to run any time.
"#;

pub const TOOL_QUALITY_CHECK_TS: &str = r#"/**
 * Quality Check
 * =============
 * Runs checks defined in .sdlc/tools/quality-check/config.yaml and reports pass/fail.
 *
 * WHAT IT DOES
 * ------------
 * --run:   Reads JSON from stdin: { "scope"?: "string" }
 *          Loads checks from .sdlc/tools/quality-check/config.yaml.
 *          Runs each check's script as a shell command, records pass/fail + output.
 *          If scope is provided, only runs checks whose name matches the filter string.
 *          Returns ToolResult<{ passed, failed, checks[] }>.
 *
 * --meta:  Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 *
 * WHAT IT READS
 * -------------
 * - .sdlc/tools/quality-check/config.yaml
 *   → checks[]: { name, description, script }
 *
 * WHAT IT WRITES
 * --------------
 * - STDERR: structured log lines via _shared/log.ts
 * - STDOUT: JSON only (ToolResult shape from _shared/types.ts)
 *
 * EXTENDING
 * ---------
 * Add or edit checks in .sdlc/tools/quality-check/config.yaml:
 *   checks:
 *     - name: test
 *       description: Run unit tests
 *       script: cargo test --all
 * The quality-check tool picks them up automatically — no code changes needed.
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

const log = makeLogger('quality-check')

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'quality-check',
  display_name: 'Quality Check',
  description: 'Runs checks from .sdlc/tools/quality-check/config.yaml and reports pass/fail',
  version: '0.3.0',
  requires_setup: false,
  input_schema: {
    type: 'object',
    properties: {
      scope: {
        type: 'string',
        description: 'Optional filter — only run checks whose name matches this string',
      },
    },
  },
  output_schema: {
    type: 'object',
    properties: {
      passed: { type: 'number' },
      failed: { type: 'number' },
      checks: {
        type: 'array',
        items: {
          type: 'object',
          properties: {
            name: { type: 'string' },
            description: { type: 'string' },
            command: { type: 'string' },
            status: { type: 'string', enum: ['passed', 'failed'] },
            output: { type: 'string' },
            duration_ms: { type: 'number' },
          },
        },
      },
    },
  },
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface PlatformCommand {
  name: string
  description?: string
  script: string
}

interface CheckResult {
  name: string
  description: string
  command: string
  status: 'passed' | 'failed'
  output: string
  duration_ms: number
}

interface QualityCheckOutput {
  passed: number
  failed: number
  checks: CheckResult[]
}

// ---------------------------------------------------------------------------
// Config YAML parser — reads checks[] from tool-local config.yaml
// ---------------------------------------------------------------------------

/**
 * Parse the `checks:` array from the tool's config.yaml.
 * Handles the specific YAML shape used by quality-check:
 *   checks:
 *     - name: <string>
 *       description: <string>
 *       script: <single-quoted or bare string>
 */
function parseChecksFromYaml(content: string): PlatformCommand[] {
  const checks: PlatformCommand[] = []
  const lines = content.split('\n')

  let inChecks = false
  let current: Partial<PlatformCommand> | null = null

  for (const line of lines) {
    // Top-level `checks:` section header
    if (/^checks:/.test(line)) {
      inChecks = true
      continue
    }
    // Any other top-level key ends the checks section
    if (/^\S/.test(line) && !/^checks:/.test(line)) {
      inChecks = false
    }

    if (!inChecks) continue

    // New item: `  - name: <value>`
    const itemMatch = line.match(/^\s{2}-\s+name:\s*(.*)$/)
    if (itemMatch) {
      if (current?.name && current?.script) {
        checks.push(current as PlatformCommand)
      }
      current = { name: unquoteYaml(itemMatch[1].trim()), description: '', script: '' }
      continue
    }

    if (!current) continue

    const descMatch = line.match(/^\s+description:\s*(.*)$/)
    if (descMatch) {
      current.description = unquoteYaml(descMatch[1].trim())
      continue
    }

    const scriptMatch = line.match(/^\s+script:\s*(.*)$/)
    if (scriptMatch) {
      current.script = unquoteYaml(scriptMatch[1].trim())
      continue
    }
  }

  if (current?.name && current?.script) {
    checks.push(current as PlatformCommand)
  }

  return checks
}

/** Strip surrounding single or double quotes from a YAML scalar value. */
function unquoteYaml(s: string): string {
  return s.replace(/^'([\s\S]*)'$/, '$1').replace(/^"([\s\S]*)"$/, '$1')
}

/** Load checks from the tool's own config.yaml. Returns [] on any error. */
function loadChecks(root: string): PlatformCommand[] {
  const configPath = join(root, '.sdlc', 'tools', 'quality-check', 'config.yaml')
  try {
    const raw = readFileSync(configPath, 'utf8')
    return parseChecksFromYaml(raw)
  } catch (e) {
    log.warn(`Could not read tool config at ${configPath}: ${e}`)
    return []
  }
}

// ---------------------------------------------------------------------------
// Run — execute platform checks
// ---------------------------------------------------------------------------

export async function run(
  input: { scope?: string },
  root: string,
): Promise<ToolResult<QualityCheckOutput>> {
  const start = Date.now()

  const commands = loadChecks(root)

  if (commands.length === 0) {
    log.warn('No checks configured in .sdlc/tools/quality-check/config.yaml — nothing to run')
    const duration_ms = Date.now() - start
    return {
      ok: true,
      data: { passed: 0, failed: 0, checks: [] },
      duration_ms,
    }
  }

  // Apply scope filter
  const scope = input.scope?.trim()
  const filtered = scope
    ? commands.filter(c => c.name.includes(scope))
    : commands

  log.info(`running ${filtered.length} check(s)${scope ? ` (scope: "${scope}")` : ''}`)

  const checks: CheckResult[] = []

  for (const cmd of filtered) {
    const checkStart = Date.now()
    log.info(`running check: ${cmd.name}`)

    let status: 'passed' | 'failed' = 'passed'
    let output = ''

    try {
      const result = execSync(cmd.script, {
        cwd: root,
        encoding: 'utf8',
        stdio: ['pipe', 'pipe', 'pipe'],
      })
      output = result.slice(-500) // last 500 chars
    } catch (e: unknown) {
      status = 'failed'
      if (e && typeof e === 'object' && 'stdout' in e && 'stderr' in e) {
        const err = e as { stdout?: string; stderr?: string }
        const combined = `${err.stdout ?? ''}${err.stderr ?? ''}`
        output = combined.slice(-500)
      } else {
        output = String(e).slice(-500)
      }
    }

    const duration_ms = Date.now() - checkStart
    log.info(`  ${cmd.name}: ${status} (${duration_ms}ms)`)

    checks.push({
      name: cmd.name,
      description: cmd.description ?? '',
      command: cmd.script,
      status,
      output,
      duration_ms,
    })
  }

  const passed = checks.filter(c => c.status === 'passed').length
  const failed = checks.filter(c => c.status === 'failed').length
  const duration_ms = Date.now() - start

  log.info(`done: ${passed} passed, ${failed} failed in ${duration_ms}ms`)

  return {
    ok: failed === 0,
    data: { passed, failed, checks },
    duration_ms,
  }
}

// ---------------------------------------------------------------------------
// CLI entrypoint
// ---------------------------------------------------------------------------

const mode = getArgs()[0] ?? '--run'
const root = process.env.SDLC_ROOT ?? process.cwd()

if (mode === '--meta') {
  console.log(JSON.stringify(meta))
  exit(0)
} else if (mode === '--run') {
  readStdin()
    .then(raw => run(JSON.parse(raw || '{}') as { scope?: string }, root))
    .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
    .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta or --run.`)
  exit(1)
}
"#;

pub const TOOL_QUALITY_CHECK_CONFIG_YAML: &str = r#"# quality-check tool configuration
# Add your project's quality checks below.
# Each check runs its `script` as a shell command in the project root.
#
# Example:
#   checks:
#     - name: test
#       description: Run unit tests
#       script: cargo test --all
name: quality-check
version: "0.3.0"
checks:
"#;

pub const TOOL_QUALITY_CHECK_README_MD: &str = r#"# Quality Check

Runs checks defined in `.sdlc/tools/quality-check/config.yaml` and reports pass/fail.

## Usage

```bash
# Run all configured checks
sdlc tool run quality-check

# Filter to checks whose name matches a string
sdlc tool run quality-check --scope test
```

## How it works

Reads `checks` from `.sdlc/tools/quality-check/config.yaml`, runs each script as a shell
command in the project root, and reports pass/fail with the last 500 characters of output.

## Adding checks

Edit `.sdlc/tools/quality-check/config.yaml`:

```yaml
checks:
  - name: test
    description: Run unit tests
    script: cargo test --all
  - name: lint
    description: Run linter
    script: cargo clippy --all -- -D warnings
```

The quality-check tool picks them up automatically — no code changes needed.
"#;

/// dev-driver tool.ts — always overwrite (managed content).
pub const TOOL_DEV_DRIVER_TS: &str = include_str!("../../../../../.sdlc/tools/dev-driver/tool.ts");

/// dev-driver README.md — write-if-missing (user-annotatable).
pub const TOOL_DEV_DRIVER_README_MD: &str =
    include_str!("../../../../../.sdlc/tools/dev-driver/README.md");

/// telegram-recap tool.ts — always overwrite (managed content).
pub const TOOL_TELEGRAM_RECAP_TS: &str =
    include_str!("../../../../../.sdlc/tools/telegram-recap/tool.ts");

/// telegram-recap config.yaml — write-if-missing (user may customize).
pub const TOOL_TELEGRAM_RECAP_CONFIG_YAML: &str =
    include_str!("../../../../../.sdlc/tools/telegram-recap/config.yaml");

/// telegram-recap README.md — write-if-missing (user-annotatable).
pub const TOOL_TELEGRAM_RECAP_README_MD: &str =
    include_str!("../../../../../.sdlc/tools/telegram-recap/README.md");

pub const TOOL_STATIC_TOOLS_MD: &str = r#"# SDLC Tools

Project-specific tools installed by sdlc. Use `sdlc tool run <name>` to invoke.

Run `sdlc tool sync` to regenerate this file from live tool metadata.

---

## ama — AMA — Ask Me Anything

Answers questions about the codebase by searching a pre-built keyword index.

**Run:** `sdlc tool run ama --question "..."`
**Setup required:** Yes — `sdlc tool run ama --setup`
_Indexes source files for keyword search (run once, then re-run when files change significantly)_

---

## quality-check — Quality Check

Runs checks from .sdlc/tools/quality-check/config.yaml and reports pass/fail.

**Run:** `sdlc tool run quality-check`
**Setup required:** No
_Edit `.sdlc/tools/quality-check/config.yaml` to add your project's checks_

---

## dev-driver — Dev Driver

Finds the next development action and dispatches it — advances the project one step per tick.

**Run:** `sdlc tool run dev-driver`
**Setup required:** No
_Configure via orchestrator: Label=dev-driver, Tool=dev-driver, Input={}, Recurrence=14400. See `.sdlc/tools/dev-driver/README.md` for full docs._

---

## telegram-recap — Telegram Recap

Fetch and email a Telegram chat digest — pulls messages from the configured window and sends via Resend.

**Run:** `sdlc tool run telegram-recap --input '{}'`
**Setup required:** Yes — `sdlc tool run telegram-recap --setup`
_Requires 5 secrets: TELEGRAM_BOT_TOKEN, RESEND_API_KEY, RESEND_FROM, RESEND_TO, TELEGRAM_CHAT_IDS (optional). Schedule with orchestrator (--every 86400) for a daily digest._

---

## Adding a Custom Tool

Run `sdlc tool scaffold <name> "<description>"` to create a new tool skeleton.
Then implement the `run()` function in `.sdlc/tools/<name>/tool.ts` and run `sdlc tool sync`.
"#;
