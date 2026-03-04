/**
 * Beat — Agentic Project Pulse Check
 * ====================================
 * Evaluates project direction against VISION.md using a recruited leadership
 * agent. Streams NDJSON progress events and persists evaluations to beat.yaml.
 *
 * WHAT IT DOES
 * ------------
 * --run (evaluate mode):
 *   Reads project state (vision, features, milestones) via _shared/sdlc.ts.
 *   Recruits a CTO/CPO agent via _shared/agent.ts if not already present.
 *   Invokes the agent to produce a verdict: on-track | drifting | off-course.
 *   Appends the evaluation to .sdlc/beat.yaml.
 *   Streams NDJSON progress events throughout.
 *
 * --run (week mode):
 *   Reads recent evaluations from .sdlc/beat.yaml (last 14 days).
 *   Synthesizes a top-5 prioritized weekly check-in list.
 *   No agent invocation, no writes.
 *
 * --meta: Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 *
 * STREAMING PROTOCOL
 * ------------------
 * Each stdout line is a valid JSON object:
 *   {"event":"gathering","message":"..."}
 *   {"event":"recruiting","message":"..."}
 *   {"event":"evaluating","message":"..."}
 *   {"event":"writing","message":"..."}
 *   {"event":"done","result":{...}}
 *   {"event":"error","message":"..."}
 *
 * The "done" event always carries the full ToolResult. Callers that only want
 * the final result can read the last stdout line.
 *
 * WHAT IT READS
 * -------------
 * - VISION.md                              (via _shared/sdlc.ts readVision)
 * - sdlc feature list --json              (via _shared/sdlc.ts readFeatures)
 * - sdlc milestone list --json            (via _shared/sdlc.ts readMilestones)
 * - .sdlc/beat.yaml                        (evaluate: read for next ID; week: read for synthesis)
 * - .claude/agents/cto-cpo-lens.md        (or created by _shared/agent.ts ensureAgent)
 *
 * WHAT IT WRITES
 * --------------
 * - .sdlc/beat.yaml                        (evaluate mode only: appends one beat record)
 * - STDERR: structured log lines via _shared/log.ts
 * - STDOUT: NDJSON lines (one event per line)
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import type { FeatureSummary, MilestoneSummary } from '../_shared/sdlc.ts'
import { readVision, readFeatures, readMilestones } from '../_shared/sdlc.ts'
import { ensureAgent, runAgentCli as runAgent } from '../_shared/agent.ts'
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs'
import { join } from 'node:path'

const log = makeLogger('beat')

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'beat',
  display_name: 'Beat — Project Pulse Check',
  description: 'Evaluates project direction against VISION.md using a recruited leadership agent and produces a verdict with concerns',
  version: '1.0.0',
  requires_setup: false,
  input_schema: {
    type: 'object',
    required: ['scope', 'mode'],
    properties: {
      scope: {
        type: 'string',
        description: 'Evaluation scope: "project" for full project, a milestone slug, or "feature:<slug>" for a single feature',
      },
      mode: {
        type: 'string',
        enum: ['evaluate', 'week'],
        description: '"evaluate" runs a fresh evaluation; "week" produces top-5 weekly check-in items from recent beats',
      },
    },
    additionalProperties: false,
  },
  output_schema: {
    type: 'object',
    properties: {
      verdict: {
        type: 'string',
        enum: ['on-track', 'drifting', 'off-course'],
        description: 'Leadership verdict on project direction (evaluate mode only)',
      },
      score: {
        type: 'number',
        minimum: 0,
        maximum: 100,
        description: 'Alignment score 0-100 (evaluate mode only)',
      },
      concerns: {
        type: 'array',
        items: { type: 'string' },
        description: 'List of identified concerns (evaluate mode only)',
      },
      week_items: {
        type: 'array',
        items: {
          type: 'object',
          properties: {
            priority: { type: 'number', minimum: 1, maximum: 5 },
            item: { type: 'string' },
            feature: { type: 'string' },
          },
          required: ['priority', 'item'],
        },
        description: 'Top-5 prioritized check-in items (week mode only)',
      },
      beat_id: {
        type: 'string',
        description: 'ID of the written beat record (evaluate mode only)',
      },
    },
  },
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface BeatInput {
  scope: string
  mode: 'evaluate' | 'week'
}

interface WeekItem {
  priority: number
  item: string
  feature?: string
}

type BeatOutput =
  | { verdict: 'on-track' | 'drifting' | 'off-course'; score: number; concerns: string[]; beat_id: string }
  | { week_items: WeekItem[] }

interface BeatRecord {
  id: string
  scope: string
  mode: 'evaluate'
  timestamp: string
  verdict: 'on-track' | 'drifting' | 'off-course'
  score: number
  concerns: string[]
}

interface BeatFile {
  beats: BeatRecord[]
}

interface AgentVerdict {
  verdict: 'on-track' | 'drifting' | 'off-course'
  score: number
  concerns: string[]
}

// ---------------------------------------------------------------------------
// NDJSON event emitter
// ---------------------------------------------------------------------------

type BeatEvent = 'gathering' | 'recruiting' | 'evaluating' | 'writing' | 'done' | 'error'

function emit(event: BeatEvent, payload: Record<string, unknown>): void {
  console.log(JSON.stringify({ event, ...payload }))
}

// ---------------------------------------------------------------------------
// beat.yaml helpers
// ---------------------------------------------------------------------------

function loadBeatFile(root: string): BeatFile {
  const beatPath = join(root, '.sdlc', 'beat.yaml')
  if (!existsSync(beatPath)) return { beats: [] }
  try {
    const raw = readFileSync(beatPath, 'utf8')
    return parseBeatYaml(raw)
  } catch {
    return { beats: [] }
  }
}

/**
 * Minimal YAML parser for beat.yaml. Handles the specific structure we write.
 * We write it back as formatted YAML manually to avoid a yaml dependency.
 */
function parseBeatYaml(raw: string): BeatFile {
  const beats: BeatRecord[] = []
  // Split on beat record boundaries (lines starting with "  - id:")
  const recordPattern = /  - id: (.+)\n    scope: (.+)\n    mode: (.+)\n    timestamp: (.+)\n    verdict: (.+)\n    score: (\d+)\n    concerns:([\s\S]*?)(?=  - id:|$)/g
  let match
  while ((match = recordPattern.exec(raw)) !== null) {
    const [, id, scope, mode, timestamp, verdict, scoreStr, concernsBlock] = match
    const concerns: string[] = []
    for (const line of concernsBlock.split('\n')) {
      const trimmed = line.trim()
      if (trimmed.startsWith('- ')) {
        concerns.push(trimmed.slice(2).replace(/^["']|["']$/g, ''))
      }
    }
    beats.push({
      id: id.trim(),
      scope: scope.trim(),
      mode: (mode.trim() as 'evaluate'),
      timestamp: timestamp.trim(),
      verdict: verdict.trim() as BeatRecord['verdict'],
      score: parseInt(scoreStr.trim(), 10),
      concerns,
    })
  }
  return { beats }
}

function nextBeatId(beats: BeatRecord[]): string {
  if (beats.length === 0) return 'beat-001'
  // Find the highest numeric suffix
  let max = 0
  for (const beat of beats) {
    const match = beat.id.match(/^beat-(\d+)$/)
    if (match) max = Math.max(max, parseInt(match[1], 10))
  }
  return `beat-${String(max + 1).padStart(3, '0')}`
}

function serializeBeatFile(file: BeatFile): string {
  const lines: string[] = ['beats:']
  for (const beat of file.beats) {
    lines.push(`  - id: ${beat.id}`)
    lines.push(`    scope: ${beat.scope}`)
    lines.push(`    mode: ${beat.mode}`)
    lines.push(`    timestamp: ${beat.timestamp}`)
    lines.push(`    verdict: ${beat.verdict}`)
    lines.push(`    score: ${beat.score}`)
    lines.push('    concerns:')
    for (const concern of beat.concerns) {
      // Escape single quotes in concern strings
      const escaped = concern.includes("'") ? `"${concern}"` : concern
      lines.push(`      - ${escaped}`)
    }
  }
  return lines.join('\n') + '\n'
}

function writeBeat(root: string, record: Omit<BeatRecord, 'id'>): string {
  const beatPath = join(root, '.sdlc', 'beat.yaml')
  const sdlcDir = join(root, '.sdlc')
  mkdirSync(sdlcDir, { recursive: true })

  const file = loadBeatFile(root)
  const id = nextBeatId(file.beats)
  file.beats.push({ id, ...record })
  writeFileSync(beatPath, serializeBeatFile(file), 'utf8')
  return id
}

// ---------------------------------------------------------------------------
// Evaluate mode
// ---------------------------------------------------------------------------

async function runEvaluate(input: BeatInput, root: string): Promise<ToolResult<BeatOutput>> {
  const start = Date.now()

  // ── Step 1: Gather state ──────────────────────────────────────────────────
  let features: FeatureSummary[] = []
  let milestones: MilestoneSummary[] = []
  let vision = ''

  try {
    vision = readVision(root)
    features = readFeatures(root)
    milestones = readMilestones(root)
    const scopeNote = input.scope === 'project'
      ? `${features.length} features, ${milestones.length} milestones`
      : input.scope.startsWith('feature:')
        ? `feature ${input.scope.slice(8)}`
        : `domain ${input.scope}`
    emit('gathering', { message: `Reading project state (${scopeNote})...` })
    log.info(`gathered state: ${features.length} features, ${milestones.length} milestones, vision ${vision.length} chars`)
  } catch (e) {
    emit('error', { message: `Failed to gather state: ${e}` })
    return { ok: false, error: `State gathering failed: ${e}`, duration_ms: Date.now() - start }
  }

  // ── Step 2: Filter by scope ───────────────────────────────────────────────
  let scopedFeatures = features
  let scopeDescription = 'the entire project'
  if (input.scope.startsWith('feature:')) {
    const slug = input.scope.slice(8)
    scopedFeatures = features.filter(f => f.slug === slug)
    scopeDescription = `feature ${slug}`
    if (scopedFeatures.length === 0) {
      emit('error', { message: `Feature '${slug}' not found` })
      return { ok: false, error: `Feature '${slug}' not found`, duration_ms: Date.now() - start }
    }
  } else if (input.scope !== 'project') {
    // Domain/milestone scope: filter features by milestone slug match
    const domain = input.scope
    const matchingMilestone = milestones.find(m => m.slug === domain)
    if (matchingMilestone) {
      // Features associated with this milestone (heuristic: slug contains domain or phase matches)
      scopeDescription = `milestone ${domain}`
    }
    // Fall back to all features if domain not found as milestone
  }

  // ── Step 3: Recruit/load agent ────────────────────────────────────────────
  const agentSlug = input.scope.startsWith('feature:') ? 'tech-lead-lens' : 'cto-cpo-lens'
  const agentRole = agentSlug === 'cto-cpo-lens'
    ? 'Strategic CTO/CPO who evaluates product direction against vision. Expert at identifying drift from strategic objectives and surfacing the most important concerns about project health.'
    : 'Tech lead who evaluates feature health, implementation progress, task completeness, and timeline risk. Expert at identifying blockers and assessing whether a feature is heading for a clean ship.'

  let agentPath: string
  try {
    emit('recruiting', { message: `Loading ${agentSlug} agent...` })
    agentPath = ensureAgent(root, agentSlug, agentRole)
    log.info(`agent ready at: ${agentPath}`)
  } catch (e) {
    emit('error', { message: `Failed to recruit agent: ${e}` })
    return { ok: false, error: `Agent recruitment failed: ${e}`, duration_ms: Date.now() - start }
  }

  // ── Step 4: Build prompt and invoke agent ─────────────────────────────────
  const featureSummary = scopedFeatures
    .slice(0, 20)
    .map(f => `  - ${f.slug} (${f.phase})`)
    .join('\n')

  const milestoneSummary = milestones
    .slice(0, 10)
    .map(m => `  - ${m.slug} (${m.status ?? 'unknown'})`)
    .join('\n')

  const prompt = `You are reviewing ${scopeDescription} against its vision. Produce a leadership-level verdict.

VISION:
${vision || '(No VISION.md found — evaluate based on feature health only)'}

CURRENT FEATURES (${scopedFeatures.length} total):
${featureSummary || '  (none)'}

ACTIVE MILESTONES (${milestones.length} total):
${milestoneSummary || '  (none)'}

Respond with JSON only — no markdown, no explanation, just the JSON object:
{
  "verdict": "on-track",
  "score": 75,
  "concerns": ["concern 1", "concern 2"]
}

verdict must be one of: on-track, drifting, off-course
score is 0-100 (100 = perfectly on track)
concerns is an array of strings, each a specific actionable concern (max 5)`

  emit('evaluating', { message: `Agent evaluating ${scopedFeatures.length} features against VISION.md...` })
  log.info('invoking agent...')

  let rawResponse: string
  try {
    rawResponse = runAgent(agentPath, prompt, { timeout_ms: 90_000 })
  } catch (e) {
    emit('error', { message: `Agent invocation failed: ${e}` })
    return { ok: false, error: `Agent invocation failed: ${e}`, duration_ms: Date.now() - start }
  }

  // ── Step 5: Parse verdict ─────────────────────────────────────────────────
  let verdict: AgentVerdict | null = parseVerdict(rawResponse)
  if (!verdict) {
    // Retry: agent may have wrapped in markdown — try extracting JSON block
    const jsonMatch = rawResponse.match(/\{[\s\S]*"verdict"[\s\S]*\}/)
    if (jsonMatch) verdict = parseVerdict(jsonMatch[0])
  }

  if (!verdict) {
    log.warn(`failed to parse verdict from: ${rawResponse.slice(0, 200)}`)
    emit('error', { message: 'Agent response could not be parsed as verdict JSON' })
    return {
      ok: false,
      error: `Could not parse agent verdict. Raw response: ${rawResponse.slice(0, 500)}`,
      duration_ms: Date.now() - start,
    }
  }

  // ── Step 6: Persist beat record ───────────────────────────────────────────
  let beatId: string
  try {
    emit('writing', { message: `Persisting beat record to .sdlc/beat.yaml...` })
    beatId = writeBeat(root, {
      scope: input.scope,
      mode: 'evaluate',
      timestamp: new Date().toISOString(),
      verdict: verdict.verdict,
      score: verdict.score,
      concerns: verdict.concerns,
    })
    emit('writing', { message: `Wrote ${beatId}` })
    log.info(`wrote ${beatId}`)
  } catch (e) {
    emit('error', { message: `Failed to write beat.yaml: ${e}` })
    return { ok: false, error: `beat.yaml write failed: ${e}`, duration_ms: Date.now() - start }
  }

  const result: ToolResult<BeatOutput> = {
    ok: true,
    data: {
      verdict: verdict.verdict,
      score: verdict.score,
      concerns: verdict.concerns,
      beat_id: beatId,
    },
    duration_ms: Date.now() - start,
  }
  emit('done', { result })
  return result
}

function parseVerdict(raw: string): AgentVerdict | null {
  try {
    const parsed = JSON.parse(raw.trim()) as Record<string, unknown>
    if (
      typeof parsed.verdict === 'string' &&
      ['on-track', 'drifting', 'off-course'].includes(parsed.verdict) &&
      typeof parsed.score === 'number' &&
      Array.isArray(parsed.concerns)
    ) {
      return {
        verdict: parsed.verdict as AgentVerdict['verdict'],
        score: Math.max(0, Math.min(100, parsed.score)),
        concerns: (parsed.concerns as unknown[]).filter(c => typeof c === 'string').slice(0, 5) as string[],
      }
    }
    return null
  } catch {
    return null
  }
}

// ---------------------------------------------------------------------------
// Week mode
// ---------------------------------------------------------------------------

async function runWeek(input: BeatInput, root: string): Promise<ToolResult<BeatOutput>> {
  const start = Date.now()
  emit('gathering', { message: 'Reading recent beat history...' })

  const file = loadBeatFile(root)
  if (file.beats.length === 0) {
    const result: ToolResult<BeatOutput> = {
      ok: true,
      data: { week_items: [] },
      duration_ms: Date.now() - start,
    }
    emit('done', { result })
    return result
  }

  // Filter to last 14 days
  const windowMs = 14 * 24 * 60 * 60 * 1000
  const cutoff = Date.now() - windowMs
  const recent = file.beats.filter(b => Date.parse(b.timestamp) >= cutoff)
  const source = recent.length > 0 ? recent : file.beats.slice(-5)

  log.info(`week mode: ${source.length} recent beats from ${file.beats.length} total`)

  // Collect all concerns across beats and score by recurrence
  const concernCounts = new Map<string, { count: number; feature?: string; verdict: string }>()
  for (const beat of source) {
    for (const concern of beat.concerns) {
      const key = normalizeConcern(concern)
      const existing = concernCounts.get(key)
      if (existing) {
        existing.count++
      } else {
        const featureMatch = concern.match(/feature:(\S+)/) ?? beat.scope.match(/^feature:(\S+)/)
        concernCounts.set(key, {
          count: 1,
          feature: featureMatch?.[1],
          verdict: beat.verdict,
        })
      }
    }
  }

  // Also weight by recency and severity (off-course > drifting > on-track)
  const verdictWeight = { 'off-course': 3, 'drifting': 2, 'on-track': 1 }

  const scored = [...concernCounts.entries()]
    .map(([concern, data]) => ({
      concern,
      score: data.count * (verdictWeight[data.verdict as keyof typeof verdictWeight] ?? 1),
      feature: data.feature,
    }))
    .sort((a, b) => b.score - a.score)
    .slice(0, 5)

  const week_items: WeekItem[] = scored.map((s, i) => ({
    priority: i + 1,
    item: s.concern,
    ...(s.feature ? { feature: s.feature } : {}),
  }))

  const result: ToolResult<BeatOutput> = {
    ok: true,
    data: { week_items },
    duration_ms: Date.now() - start,
  }
  emit('done', { result })
  return result
}

function normalizeConcern(concern: string): string {
  // Lowercase, collapse whitespace, strip leading numbers/bullets
  return concern.toLowerCase().replace(/^\d+\.\s*/, '').replace(/\s+/g, ' ').trim()
}

// ---------------------------------------------------------------------------
// Main run function
// ---------------------------------------------------------------------------

export async function run(
  input: BeatInput,
  root: string,
): Promise<ToolResult<BeatOutput>> {
  // Validate input
  if (!input.scope || typeof input.scope !== 'string') {
    return { ok: false, error: 'input.scope is required (string)' }
  }
  if (!input.mode || !['evaluate', 'week'].includes(input.mode)) {
    return { ok: false, error: 'input.mode must be "evaluate" or "week"' }
  }

  if (input.mode === 'week') {
    return runWeek(input, root)
  }
  return runEvaluate(input, root)
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
    .then(raw => run(JSON.parse(raw || '{}') as BeatInput, root))
    .then(result => {
      // Only emit done event if not already emitted (evaluate mode emits it internally)
      // We check if the result indicates it was already streamed
      if (!result.ok) {
        // Error results that weren't streamed yet
        emit('done', { result })
      }
      exit(result.ok ? 0 : 1)
    })
    .catch(e => {
      const result = { ok: false, error: String(e) }
      emit('done', { result })
      exit(1)
    })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta or --run.`)
  exit(1)
}
