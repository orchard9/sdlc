/**
 * SDLC State Primitives — typed access to .sdlc/ state files
 *
 * Provides typed readers and writers for .sdlc/ state so tools do not
 * need to invent path conventions or parse YAML ad-hoc.
 *
 * ALL functions accept `root: string` — the project root directory.
 * Resolve once at startup via getProjectRoot() and pass down.
 *
 * Read functions are fully defensive: missing files return empty defaults,
 * corrupted manifests are skipped with a stderr warning.
 *
 * Write functions use atomic temp-file + rename so partial writes never
 * leave the state directory in a corrupt state.
 *
 * No external npm dependencies. Only Node.js built-ins and _shared/runtime.ts.
 */

import {
  readFileSync,
  writeFileSync,
  readdirSync,
  renameSync,
  unlinkSync,
  statSync,
} from 'node:fs'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'
import { getEnv } from './runtime.ts'

// ---------------------------------------------------------------------------
// Private path helpers — tools use public functions, not paths directly
// ---------------------------------------------------------------------------

const FEATURES_DIR = '.sdlc/features'
const MILESTONES_DIR = '.sdlc/milestones'
const BEAT_FILE = '.sdlc/beat.yaml'
const VISION_FILE = 'VISION.md'

function featuresDir(root: string): string {
  return join(root, FEATURES_DIR)
}

function milestonesDir(root: string): string {
  return join(root, MILESTONES_DIR)
}

function beatPath(root: string): string {
  return join(root, BEAT_FILE)
}

function visionPath(root: string): string {
  return join(root, VISION_FILE)
}

// ---------------------------------------------------------------------------
// YAML mini-parser
//
// Handles the manifest.yaml schema used by features and milestones.
// Supports:
//   - Flat scalars:  key: value
//   - Nested array:  key:\n  - subkey: value\n    ...
//
// Does NOT handle arbitrary nesting, multi-line scalars, anchors, or tags.
// This is sufficient for the stable manifest schemas we own.
// ---------------------------------------------------------------------------

/** Parse flat key:value pairs from a YAML string. */
function parseYamlScalars(content: string): Record<string, string> {
  const result: Record<string, string> = {}
  for (const line of content.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('-')) continue
    const colonIdx = trimmed.indexOf(':')
    if (colonIdx === -1) continue
    const key = trimmed.slice(0, colonIdx).trim()
    const raw = trimmed.slice(colonIdx + 1).trim()
    if (!key || !raw) continue
    result[key] = raw.replace(/^["'](.*)["']$/, '$1').replace(/^"(.*)"$/, '$1')
  }
  return result
}

/** Parse a named array block from YAML into an array of flat objects.
 *
 * Given YAML like:
 *   tasks:
 *     - id: T1
 *       title: Do the thing
 *       status: pending
 *
 * parseYamlArray(content, 'tasks') returns:
 *   [{ id: 'T1', title: 'Do the thing', status: 'pending' }]
 */
function parseYamlArray(content: string, key: string): Record<string, string>[] {
  const lines = content.split('\n')
  const items: Record<string, string>[] = []
  let inArray = false
  let current: Record<string, string> | null = null

  for (const line of lines) {
    // Top-level key matching our target array
    if (line.match(new RegExp(`^${key}:`))) {
      inArray = true
      continue
    }
    // Any other top-level key (no indentation) ends the array
    if (inArray && line.length > 0 && !/^\s/.test(line)) {
      inArray = false
      if (current) {
        items.push(current)
        current = null
      }
      continue
    }
    if (!inArray) continue

    // New array item
    const itemMatch = line.match(/^\s{2}-\s+(\w[\w-]*):\s*(.*)$/)
    if (itemMatch) {
      if (current) items.push(current)
      current = {}
      const val = itemMatch[2].trim().replace(/^["'](.*)["']$/, '$1')
      current[itemMatch[1]] = val
      continue
    }

    // Continuation key:value within current item
    if (current) {
      const kvMatch = line.match(/^\s{4,}(\w[\w-]*):\s*(.*)$/)
      if (kvMatch) {
        const val = kvMatch[2].trim().replace(/^["'](.*)["']$/, '$1')
        current[kvMatch[1]] = val
      }
    }
  }

  if (current) items.push(current)
  return items
}

// ---------------------------------------------------------------------------
// Exported types
// ---------------------------------------------------------------------------

export interface TaskSummary {
  id: string
  title: string
  status: 'pending' | 'in_progress' | 'completed'
}

export interface FeatureSummary {
  slug: string
  title: string
  phase: string
  description?: string
  tasks: TaskSummary[]
}

export interface MilestoneSummary {
  slug: string
  title: string
  status: string
  /** Feature slugs listed in the milestone manifest */
  features: string[]
}

export interface BeatConcern {
  slug?: string
  title: string
  severity: 'high' | 'medium' | 'low'
  last_checked: string
  trend: 'improving' | 'stalling' | 'worsening' | null
}

export interface BeatEvaluation {
  date: string
  scope: string
  lens: string
  verdict: 'on-track' | 'drifting' | 'off-course'
  summary: string
  concerns: BeatConcern[]
}

export interface BeatWeeklyItem {
  id: string
  title: string
  domain: string
  severity: 'high' | 'medium' | 'low'
  last_checked: string | null
  verdict: string | null
  trend: string | null
}

export interface BeatState {
  last_updated?: string
  evaluations: BeatEvaluation[]
  weekly?: {
    generated: string
    items: BeatWeeklyItem[]
  }
}

// ---------------------------------------------------------------------------
// getProjectRoot
// ---------------------------------------------------------------------------

/**
 * Resolve the project root directory.
 *
 * Priority:
 *   1. SDLC_ROOT env var — injected by `sdlc tool run`
 *   2. process.cwd() fallback
 *
 * Call once at tool startup and pass `root` to all other functions.
 */
export function getProjectRoot(): string {
  return getEnv('SDLC_ROOT') ?? (typeof process !== 'undefined' ? process.cwd() : '.')
}

// ---------------------------------------------------------------------------
// readVision
// ---------------------------------------------------------------------------

/**
 * Read VISION.md from the project root.
 * Returns empty string if the file does not exist — never throws.
 */
export function readVision(root: string): string {
  try {
    return readFileSync(visionPath(root), 'utf8')
  } catch {
    return ''
  }
}

// ---------------------------------------------------------------------------
// readFeatures
// ---------------------------------------------------------------------------

/**
 * Read all features from .sdlc/features/.
 * Returns [] if the directory does not exist or is empty.
 * Skips corrupted manifests with a stderr warning.
 */
export function readFeatures(root: string): FeatureSummary[] {
  const dir = featuresDir(root)
  let slugs: string[]
  try {
    slugs = readdirSync(dir, { withFileTypes: true })
      .filter(e => e.isDirectory())
      .map(e => e.name)
  } catch {
    return []
  }

  const features: FeatureSummary[] = []
  for (const slug of slugs) {
    const manifestPath = join(dir, slug, 'manifest.yaml')
    try {
      const content = readFileSync(manifestPath, 'utf8')
      const scalars = parseYamlScalars(content)
      const taskRows = parseYamlArray(content, 'tasks')

      const tasks: TaskSummary[] = taskRows
        .filter(t => t['id'] && t['title'])
        .map(t => ({
          id: t['id'],
          title: t['title'],
          status: (t['status'] as TaskSummary['status']) ?? 'pending',
        }))

      features.push({
        slug,
        title: scalars['title'] ?? slug,
        phase: scalars['phase'] ?? 'unknown',
        description: scalars['description'] || undefined,
        tasks,
      })
    } catch (e) {
      console.error(`[sdlc._shared] WARN: skipping corrupted manifest at ${manifestPath}: ${e}`)
    }
  }
  return features
}

// ---------------------------------------------------------------------------
// readMilestones
// ---------------------------------------------------------------------------

/**
 * Read all milestones from .sdlc/milestones/.
 * Returns [] if the directory does not exist or is empty.
 * Skips corrupted manifests with a stderr warning.
 */
export function readMilestones(root: string): MilestoneSummary[] {
  const dir = milestonesDir(root)
  let slugs: string[]
  try {
    slugs = readdirSync(dir, { withFileTypes: true })
      .filter(e => e.isDirectory())
      .map(e => e.name)
  } catch {
    return []
  }

  const milestones: MilestoneSummary[] = []
  for (const slug of slugs) {
    const manifestPath = join(dir, slug, 'manifest.yaml')
    try {
      const content = readFileSync(manifestPath, 'utf8')
      const scalars = parseYamlScalars(content)

      // features: may be a simple list of strings or missing
      const featureSlugs: string[] = []
      const lines = content.split('\n')
      let inFeatures = false
      for (const line of lines) {
        if (/^features:/.test(line)) { inFeatures = true; continue }
        if (inFeatures && /^\S/.test(line) && !/^features:/.test(line)) { inFeatures = false }
        if (inFeatures) {
          const m = line.match(/^\s*-\s+(.+)$/)
          if (m) featureSlugs.push(m[1].trim().replace(/^["'](.*)["']$/, '$1'))
        }
      }

      milestones.push({
        slug,
        title: scalars['title'] ?? slug,
        status: scalars['status'] ?? 'unknown',
        features: featureSlugs,
      })
    } catch (e) {
      console.error(`[sdlc._shared] WARN: skipping corrupted manifest at ${manifestPath}: ${e}`)
    }
  }
  return milestones
}

// ---------------------------------------------------------------------------
// readBeat / writeBeat
// ---------------------------------------------------------------------------

/**
 * Read .sdlc/beat.yaml.
 * Returns { evaluations: [] } if the file does not exist — never throws.
 */
export function readBeat(root: string): BeatState {
  const p = beatPath(root)
  let content: string
  try {
    content = readFileSync(p, 'utf8')
  } catch {
    return { evaluations: [] }
  }

  const scalars = parseYamlScalars(content)
  const evaluationBlocks = parseBeatEvaluations(content)
  const weekly = parseBeatWeekly(content)

  const state: BeatState = {
    evaluations: evaluationBlocks,
  }
  if (scalars['last_updated']) state.last_updated = scalars['last_updated']
  if (weekly) state.weekly = weekly

  return state
}

/** Parse the evaluations[] block from beat.yaml content. */
function parseBeatEvaluations(content: string): BeatEvaluation[] {
  const lines = content.split('\n')
  const evaluations: BeatEvaluation[] = []
  let inEvals = false
  let inConcerns = false
  let current: Partial<BeatEvaluation> | null = null
  let currentConcern: Partial<BeatConcern> | null = null

  const pushConcern = () => {
    if (currentConcern && currentConcern.title) {
      if (!current) return
      if (!current.concerns) current.concerns = []
      current.concerns.push({
        title: currentConcern.title,
        severity: (currentConcern.severity as BeatConcern['severity']) ?? 'medium',
        last_checked: currentConcern.last_checked ?? '',
        trend: (currentConcern.trend as BeatConcern['trend']) ?? null,
        slug: currentConcern.slug,
      })
      currentConcern = null
    }
  }

  const pushEval = () => {
    pushConcern()
    if (current && current.date) {
      evaluations.push({
        date: current.date,
        scope: current.scope ?? 'project',
        lens: current.lens ?? '',
        verdict: (current.verdict as BeatEvaluation['verdict']) ?? 'on-track',
        summary: current.summary ?? '',
        concerns: current.concerns ?? [],
      })
      current = null
    }
  }

  for (const line of lines) {
    if (/^evaluations:/.test(line)) { inEvals = true; continue }
    if (/^weekly:/.test(line) || (inEvals && /^\S/.test(line) && !/^evaluations:/.test(line))) {
      pushEval()
      inEvals = false
      inConcerns = false
      continue
    }
    if (!inEvals) continue

    // New evaluation item (2-space indent + dash)
    const evalItem = line.match(/^\s{2}-\s+(\w[\w-]*):\s*(.*)$/)
    if (evalItem) {
      if (current) pushEval()
      inConcerns = false
      current = { concerns: [] }
      const val = evalItem[2].trim().replace(/^["'](.*)["']$/, '$1')
      ;(current as Record<string, unknown>)[evalItem[1]] = val
      continue
    }

    // concerns: block start
    if (/^\s{4}concerns:/.test(line)) { inConcerns = true; continue }

    if (inConcerns) {
      // new concern item (6-space indent + dash)
      const concernItem = line.match(/^\s{6}-\s+(\w[\w-]*):\s*(.*)$/)
      if (concernItem) {
        pushConcern()
        currentConcern = {}
        const val = concernItem[2].trim().replace(/^["'](.*)["']$/, '$1')
        ;(currentConcern as Record<string, unknown>)[concernItem[1]] = val
        continue
      }
      // continuation of current concern (8-space indent)
      const concernKv = line.match(/^\s{8}(\w[\w-]*):\s*(.*)$/)
      if (concernKv && currentConcern) {
        const val = concernKv[2].trim().replace(/^["'](.*)["']$/, '$1')
        ;(currentConcern as Record<string, unknown>)[concernKv[1]] = val
        continue
      }
      // end concerns on lower indentation
      if (/^\s{4}\S/.test(line)) { pushConcern(); inConcerns = false }
    }

    // Continuation key:value for current evaluation (4-space indent)
    if (current && !inConcerns) {
      const kv = line.match(/^\s{4}(\w[\w-]*):\s*(.*)$/)
      if (kv) {
        const val = kv[2].trim().replace(/^["'](.*)["']$/, '$1')
        ;(current as Record<string, unknown>)[kv[1]] = val
      }
    }
  }

  pushEval()
  return evaluations
}

/** Parse the weekly: block from beat.yaml content. */
function parseBeatWeekly(content: string): BeatState['weekly'] | undefined {
  const lines = content.split('\n')
  let inWeekly = false
  let generated: string | undefined
  const items: BeatWeeklyItem[] = []
  let inItems = false
  let current: Partial<BeatWeeklyItem> | null = null

  const pushItem = () => {
    if (current && current.id) {
      items.push({
        id: current.id,
        title: current.title ?? '',
        domain: current.domain ?? '',
        severity: (current.severity as BeatWeeklyItem['severity']) ?? 'medium',
        last_checked: current.last_checked ?? null,
        verdict: current.verdict ?? null,
        trend: current.trend ?? null,
      })
      current = null
    }
  }

  for (const line of lines) {
    if (/^weekly:/.test(line)) { inWeekly = true; continue }
    if (inWeekly && /^\S/.test(line) && !/^weekly:/.test(line)) {
      pushItem()
      break
    }
    if (!inWeekly) continue

    const genMatch = line.match(/^\s{2}generated:\s*(.+)$/)
    if (genMatch) { generated = genMatch[1].trim().replace(/^["'](.*)["']$/, '$1'); continue }

    if (/^\s{2}items:/.test(line)) { inItems = true; continue }
    if (!inItems) continue

    const itemFirst = line.match(/^\s{4}-\s+(\w[\w-]*):\s*(.*)$/)
    if (itemFirst) {
      pushItem()
      current = {}
      const val = itemFirst[2].trim().replace(/^["'](.*)["']$/, '$1')
      ;(current as Record<string, unknown>)[itemFirst[1]] = val
      continue
    }

    const kv = line.match(/^\s{6}(\w[\w-]*):\s*(.*)$/)
    if (kv && current) {
      const val = kv[2].trim().replace(/^["'](.*)["']$/, '$1')
      ;(current as Record<string, unknown>)[kv[1]] = val
    }
  }

  pushItem()

  if (!generated && items.length === 0) return undefined
  return { generated: generated ?? '', items }
}

/**
 * Write .sdlc/beat.yaml atomically (temp file + rename).
 * Serializes BeatState to YAML using a template builder.
 * Strings are double-quote escaped to handle special characters.
 */
export function writeBeat(root: string, state: BeatState): void {
  const content = serializeBeat(state)
  const p = beatPath(root)
  const tmp = p + '.tmp'
  writeFileSync(tmp, content, 'utf8')
  renameSync(tmp, p)
}

function yamlStr(s: string | null | undefined): string {
  if (s === null || s === undefined) return 'null'
  // Replace double-quotes and newlines, wrap in double quotes
  const escaped = String(s).replace(/"/g, "'").replace(/\n/g, ' ')
  return `"${escaped}"`
}

function yamlScalar(s: string | null | undefined): string {
  if (s === null || s === undefined) return 'null'
  return String(s).replace(/\n/g, ' ')
}

function serializeBeat(state: BeatState): string {
  const lines: string[] = []
  lines.push(`last_updated: ${yamlStr(state.last_updated ?? new Date().toISOString().slice(0, 10))}`)
  lines.push('evaluations:')

  for (const ev of state.evaluations) {
    lines.push(`  - date: ${yamlStr(ev.date)}`)
    lines.push(`    scope: ${yamlScalar(ev.scope)}`)
    lines.push(`    lens: ${yamlStr(ev.lens)}`)
    lines.push(`    verdict: ${yamlScalar(ev.verdict)}`)
    lines.push(`    summary: ${yamlStr(ev.summary)}`)
    lines.push('    concerns:')
    for (const c of ev.concerns ?? []) {
      lines.push(`      - title: ${yamlStr(c.title)}`)
      if (c.slug) lines.push(`        slug: ${yamlScalar(c.slug)}`)
      lines.push(`        severity: ${yamlScalar(c.severity)}`)
      lines.push(`        last_checked: ${yamlStr(c.last_checked)}`)
      lines.push(`        trend: ${yamlScalar(c.trend)}`)
    }
    if (!ev.concerns || ev.concerns.length === 0) {
      lines.push('      []')
    }
  }

  if (state.evaluations.length === 0) {
    lines.push('  []')
  }

  if (state.weekly) {
    lines.push('weekly:')
    lines.push(`  generated: ${yamlStr(state.weekly.generated)}`)
    lines.push('  items:')
    for (const item of state.weekly.items) {
      lines.push(`    - id: ${yamlScalar(item.id)}`)
      lines.push(`      title: ${yamlStr(item.title)}`)
      lines.push(`      domain: ${yamlScalar(item.domain)}`)
      lines.push(`      severity: ${yamlScalar(item.severity)}`)
      lines.push(`      last_checked: ${item.last_checked ? yamlStr(item.last_checked) : 'null'}`)
      lines.push(`      verdict: ${item.verdict ? yamlStr(item.verdict) : 'null'}`)
      lines.push(`      trend: ${item.trend ? yamlStr(item.trend) : 'null'}`)
    }
    if (state.weekly.items.length === 0) {
      lines.push('    []')
    }
  }

  return lines.join('\n') + '\n'
}

// ---------------------------------------------------------------------------
// createPonder / appendPonderSession
// ---------------------------------------------------------------------------

/**
 * Create a new ponder entry by spawning `sdlc ponder create "<title>"`.
 * Returns the generated slug.
 * Throws if sdlc is not on PATH or the command fails.
 */
export function createPonder(root: string, title: string): string {
  // Generate a slug from the title: lowercase, replace non-alphanumeric with hyphens,
  // collapse consecutive hyphens, trim leading/trailing hyphens.
  const slug = title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')

  const result = spawnSync('sdlc', ['ponder', 'create', '--title', title, slug], {
    cwd: root,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  })

  if (result.status !== 0) {
    throw new Error(
      `sdlc ponder create failed (exit ${result.status ?? 'unknown'}): ${result.stderr ?? result.stdout ?? 'no output'}`,
    )
  }

  return slug
}

/**
 * Append a session to an existing ponder entry.
 *
 * Strictly follows the two-step session log protocol:
 *   1. Write content to a temp file at /tmp/ponder-session-<slug>-<ts>.md
 *   2. Spawn `sdlc ponder session log <slug> --file <tmp>`
 *   3. Best-effort cleanup of the temp file
 *
 * Throws if the sdlc command fails.
 */
export function appendPonderSession(root: string, slug: string, content: string): void {
  const tmp = `/tmp/ponder-session-${slug}-${Date.now()}.md`
  writeFileSync(tmp, content, 'utf8')

  let exitCode: number | null = null
  let stderr = ''
  try {
    const result = spawnSync('sdlc', ['ponder', 'session', 'log', slug, '--file', tmp], {
      cwd: root,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    })
    exitCode = result.status
    stderr = result.stderr ?? ''
  } finally {
    // Best-effort cleanup — never let a cleanup failure mask the real error
    try { unlinkSync(tmp) } catch { /* ignore */ }
  }

  if (exitCode !== 0) {
    throw new Error(
      `sdlc ponder session log failed (exit ${exitCode ?? 'unknown'}): ${stderr || 'no output'}`,
    )
  }
}
