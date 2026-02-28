/**
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
