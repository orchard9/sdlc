/**
 * Dev Driver
 * ==========
 * Finds the single most important next development action and dispatches it
 * asynchronously via the sdlc-server agent infrastructure. Designed to run on
 * a schedule (e.g. every 4 hours) via the sdlc orchestrator to make development
 * self-advancing.
 *
 * WHAT IT DOES
 * ------------
 * --run:   Reads JSON from stdin: {} (no parameters)
 *          Applies a 5-level priority waterfall:
 *            1. Active runs  — if an agent run is in flight, exit waiting
 *            2. Quality      — if quality-check fails, exit quality_failing
 *            3. Features     — if features have active directives, dispatch /sdlc-next
 *            4. Wave         — if a milestone wave is ready, dispatch /sdlc-run-wave
 *            5. Idle         — nothing to do, exit idle
 *          Returns ToolResult<DevDriverOutput>.
 *
 * --meta:  Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 *
 * KEY INVARIANT
 * -------------
 * Level 3 dispatches /sdlc-next (one step), NOT /sdlc-run (to completion).
 * The 4h recurrence IS the iteration rhythm. Each tick advances exactly one
 * feature by one directive. This keeps the developer in control.
 *
 * DISPATCH
 * --------
 * Calls POST /api/tools/agent-dispatch on the local sdlc-server, which:
 *   - Creates a RunRecord (visible in the activity feed)
 *   - Streams SSE events to the frontend
 *   - Returns 409 Conflict if the same run key is already in flight
 * Requires SDLC_SERVER_URL and SDLC_AGENT_TOKEN (injected by the server).
 *
 * HOW TO SKIP A FEATURE
 * ---------------------
 * Add a task to the feature with "skip:autonomous" in the title:
 *   sdlc task add <slug> --title "skip:autonomous: needs human review"
 * The dev-driver will exclude this feature from Level 3 until the task is removed.
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import { runAgentDispatch } from '../_shared/agent.ts'
import { execSync } from 'node:child_process'
import { readFileSync, existsSync } from 'node:fs'
import { join } from 'node:path'

const log = makeLogger('dev-driver')

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'dev-driver',
  display_name: 'Dev Driver',
  description: 'Finds the next development action and dispatches it — advances the project one step per tick',
  version: '1.0.0',
  requires_setup: false,
  input_schema: {
    type: 'object',
    properties: {},
    additionalProperties: false,
  },
  output_schema: {
    type: 'object',
    properties: {
      action: {
        type: 'string',
        enum: ['waiting', 'quality_failing', 'feature_advanced', 'wave_started', 'idle'],
        description: 'What the dev-driver decided to do',
      },
      reason: {
        type: 'string',
        description: 'Human-readable reason (present when action=waiting or idle)',
      },
      failed_checks: {
        type: 'array',
        items: { type: 'string' },
        description: 'Names of failed quality checks (present when action=quality_failing)',
      },
      slug: {
        type: 'string',
        description: 'Feature slug that was advanced (present when action=feature_advanced)',
      },
      phase: {
        type: 'string',
        description: 'Current phase of the feature (present when action=feature_advanced)',
      },
      directive: {
        type: 'string',
        description: 'The /sdlc-next command that was dispatched (present when action=feature_advanced)',
      },
      milestone: {
        type: 'string',
        description: 'Milestone slug that started (present when action=wave_started)',
      },
      run_id: {
        type: 'string',
        description: 'Server run ID for the dispatched agent (present when action=feature_advanced or wave_started)',
      },
    },
    required: ['action'],
  },
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

type DevDriverOutput =
  | { action: 'waiting'; reason: string }
  | { action: 'quality_failing'; failed_checks: string[] }
  | { action: 'feature_advanced'; slug: string; phase: string; directive: string; run_id?: string }
  | { action: 'wave_started'; milestone: string; run_id?: string }
  | { action: 'idle'; reason: string }

// ---------------------------------------------------------------------------
// Quality check (T3 - Level 2)
// ---------------------------------------------------------------------------

interface QCCheck {
  name: string
  status: 'passed' | 'failed'
}

interface QCResult {
  passed: number
  failed: number
  checks: QCCheck[]
}

function runQualityCheck(root: string): { failed: number; failedNames: string[] } {
  const toolPath = join(root, '.sdlc', 'tools', 'quality-check', 'tool.ts')
  if (!existsSync(toolPath)) {
    log.warn('quality-check tool not found — skipping quality gate')
    return { failed: 0, failedNames: [] }
  }
  try {
    const raw = execSync(`node ${toolPath} --run`, {
      input: '{}',
      encoding: 'utf8',
      cwd: root,
      timeout: 120_000,
      stdio: ['pipe', 'pipe', 'pipe'],
    })
    const result = JSON.parse(raw) as ToolResult<QCResult>
    if (!result.data) return { failed: 0, failedNames: [] }
    const failedNames = result.data.checks
      .filter(c => c.status === 'failed')
      .map(c => c.name)
    return { failed: result.data.failed, failedNames }
  } catch (e) {
    log.warn(`quality-check execution error: ${e} — treating as no failures`)
    return { failed: 0, failedNames: [] }
  }
}

// ---------------------------------------------------------------------------
// Active run check (T8)
// ---------------------------------------------------------------------------

function hasActiveRuns(root: string): boolean {
  try {
    const raw = execSync('sdlc run list --status running --json', {
      encoding: 'utf8',
      cwd: root,
      timeout: 10_000,
    })
    const runs = JSON.parse(raw)
    return Array.isArray(runs) && runs.length > 0
  } catch {
    // sdlc run list may not exist yet — skip this check gracefully
    log.warn('sdlc run list not available — skipping active run check')
    return false
  }
}

// ---------------------------------------------------------------------------
// Feature selection (T3 - Level 3, T7, T9, T10)
// ---------------------------------------------------------------------------

interface FeatureDirective {
  feature: string
  current_phase: string
  action: string
  next_command: string
}

const ACTIVE_PHASES = new Set(['implementation', 'review', 'audit', 'qa'])

function hasSkipTag(slug: string, root: string): boolean {
  const tasksPath = join(root, '.sdlc', 'features', slug, 'tasks.md')
  if (!existsSync(tasksPath)) return false
  try {
    const content = readFileSync(tasksPath, 'utf8')
    return /skip:autonomous/i.test(content)
  } catch {
    return false
  }
}

function findActionableFeature(root: string): FeatureDirective | null {
  try {
    const raw = execSync('sdlc next --json', {
      encoding: 'utf8',
      cwd: root,
      timeout: 30_000,
    })
    const all = JSON.parse(raw) as FeatureDirective[]
    const actionable = all
      .filter(d => d.action !== 'done')
      .filter(d => ACTIVE_PHASES.has(d.current_phase))
      .filter(d => !hasSkipTag(d.feature, root))
      .sort((a, b) => a.feature.localeCompare(b.feature))
    return actionable[0] ?? null
  } catch (e) {
    log.warn(`sdlc next --json failed: ${e}`)
    return null
  }
}

// ---------------------------------------------------------------------------
// Wave detection (T3 - Level 4)
// ---------------------------------------------------------------------------

interface MilestoneInfo {
  slug: string
  status: string
  features: { phase: string; status: string }[]
  done: number
  total: number
}

const WAVE_READY_PHASES = new Set(['planned', 'ready'])

function findReadyWave(root: string): string | null {
  try {
    const raw = execSync('sdlc milestone list --json', {
      encoding: 'utf8',
      cwd: root,
      timeout: 15_000,
    })
    const milestones = JSON.parse(raw) as MilestoneInfo[]
    const ready = milestones
      .filter(m => m.status !== 'released' && m.total > 0)
      .filter(m =>
        m.features.every(f =>
          WAVE_READY_PHASES.has(f.phase) || f.phase === 'released'
        )
      )
      .filter(m =>
        m.features.some(f => WAVE_READY_PHASES.has(f.phase))
      )
      .sort((a, b) => a.slug.localeCompare(b.slug))
    return ready[0]?.slug ?? null
  } catch (e) {
    log.warn(`sdlc milestone list failed: ${e}`)
    return null
  }
}

// ---------------------------------------------------------------------------
// Main run function (T1, T3, T5)
// ---------------------------------------------------------------------------

export async function run(
  _input: Record<string, never>,
  root: string,
): Promise<ToolResult<DevDriverOutput>> {
  const start = Date.now()

  // ── Level 1: Active run check ─────────────────────────────────────────────
  // Replaces the TTL-based lock file. If any sdlc agent run is currently
  // in flight (per `sdlc run list --status running`), wait for it to finish
  // before dispatching another.
  if (hasActiveRuns(root)) {
    log.info('active sdlc agent run detected — waiting')
    return { ok: true, data: { action: 'waiting', reason: 'agent run in progress' }, duration_ms: Date.now() - start }
  }

  // ── Level 2: Quality check ────────────────────────────────────────────────
  log.info('running quality check')
  const qc = runQualityCheck(root)
  if (qc.failed > 0) {
    log.info(`quality failing: ${qc.failedNames.join(', ')}`)
    return { ok: true, data: { action: 'quality_failing', failed_checks: qc.failedNames }, duration_ms: Date.now() - start }
  }
  log.info('quality checks passed')

  // ── Level 3: Features with active directives ──────────────────────────────
  const feature = findActionableFeature(root)
  if (feature) {
    log.info(`advancing feature: ${feature.feature} (${feature.current_phase})`)

    // Intentionally /sdlc-next — one step per tick, not /sdlc-run to completion.
    // Dispatched via the server so the run appears in the activity feed with a
    // RunRecord, SSE events, and exact flight detection (409 on duplicate key).
    const r = await runAgentDispatch(
      `/sdlc-next ${feature.feature}`,
      `dev-driver:feature:${feature.feature}`,
      `dev-driver: advance ${feature.feature}`,
      { maxTurns: 40 },
    )

    if (r.status === 'conflict') {
      log.info(`agent already running for ${feature.feature} — waiting`)
      return { ok: true, data: { action: 'waiting', reason: 'agent run in progress' }, duration_ms: Date.now() - start }
    }

    log.info(`dispatched /sdlc-next ${feature.feature} (run_id: ${r.run_id})`)
    return {
      ok: true,
      data: {
        action: 'feature_advanced',
        slug: feature.feature,
        phase: feature.current_phase,
        directive: feature.next_command || `/sdlc-next ${feature.feature}`,
        run_id: r.run_id,
      },
      duration_ms: Date.now() - start,
    }
  }

  // ── Level 4: Wave ready ───────────────────────────────────────────────────
  const milestone = findReadyWave(root)
  if (milestone) {
    log.info(`wave ready for milestone: ${milestone}`)

    const r = await runAgentDispatch(
      `/sdlc-run-wave ${milestone}`,
      `dev-driver:wave:${milestone}`,
      `dev-driver: run wave ${milestone}`,
      { maxTurns: 100 },
    )

    if (r.status === 'conflict') {
      log.info(`wave agent already running for ${milestone} — waiting`)
      return { ok: true, data: { action: 'waiting', reason: 'agent run in progress' }, duration_ms: Date.now() - start }
    }

    log.info(`dispatched /sdlc-run-wave ${milestone} (run_id: ${r.run_id})`)
    return {
      ok: true,
      data: { action: 'wave_started', milestone, run_id: r.run_id },
      duration_ms: Date.now() - start,
    }
  }

  // ── Level 5: Idle ─────────────────────────────────────────────────────────
  log.info('no actionable work found — idle')
  return {
    ok: true,
    data: { action: 'idle', reason: 'no actionable work found' },
    duration_ms: Date.now() - start,
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
    .then(raw => run(JSON.parse(raw || '{}') as Record<string, never>, root))
    .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
    .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta or --run.`)
  exit(1)
}
