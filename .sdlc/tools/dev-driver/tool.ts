/**
 * Dev Driver
 * ==========
 * Reads the parallel work queue from the sdlc-server and dispatches all items
 * simultaneously via the agent-dispatch infrastructure.
 *
 * MODES
 * -----
 * multi (default): dispatch all parallel_work items (up to 4, max 1 UAT).
 * single:          dispatch only the first item from parallel_work.
 *
 * Both modes use the same selection logic as the dashboard — items are
 * computed by select_parallel_work() in sdlc-core (Rust) and returned as
 * part of GET /api/state. This guarantees the dev-driver and dashboard
 * always agree on what to work on next.
 *
 * PRIORITY WATERFALL
 * ------------------
 * 1. Quality gate  — if quality-check fails, exit quality_failing (no dispatches).
 * 2. Dispatch      — fetch parallel_work from server, dispatch each item.
 *    - 409 Conflict = already running for that slot, skip silently.
 * 3. Idle          — parallel_work is empty, nothing to do.
 *
 * DISPATCH
 * --------
 * Calls POST /api/tools/agent-dispatch (SDLC_SERVER_URL required).
 * Each slot uses a unique run_key so 409 Conflict provides dedup.
 * No global "active run" check — per-slot 409s handle flight detection.
 *
 * HOW TO SKIP A FEATURE
 * ---------------------
 * Add a task with "skip:autonomous" in the title:
 *   sdlc task add <slug> --title "skip:autonomous: needs human review"
 * select_parallel_work() (Rust) excludes these features automatically.
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import { runAgentDispatch } from '../_shared/agent.ts'
import { execSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { join } from 'node:path'

const log = makeLogger('dev-driver')

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'dev-driver',
  display_name: 'Dev Driver',
  description: 'Dispatches up to 4 parallel work items from the dashboard queue — advances all active milestones simultaneously',
  version: '2.0.0',
  requires_setup: false,
  input_schema: {
    type: 'object',
    properties: {
      mode: {
        type: 'string',
        enum: ['multi', 'single'],
        description: 'multi (default): dispatch all parallel_work items; single: dispatch first item only',
        default: 'multi',
      },
    },
    additionalProperties: false,
  },
  output_schema: {
    type: 'object',
    properties: {
      action: {
        type: 'string',
        enum: ['quality_failing', 'dispatched', 'idle'],
        description: 'What the dev-driver decided to do',
      },
      mode: {
        type: 'string',
        enum: ['multi', 'single'],
        description: 'The mode used (present when action=dispatched)',
      },
      failed_checks: {
        type: 'array',
        items: { type: 'string' },
        description: 'Names of failed quality checks (present when action=quality_failing)',
      },
      items: {
        type: 'array',
        description: 'Dispatched items (present when action=dispatched)',
        items: {
          type: 'object',
          properties: {
            milestone_slug: { type: 'string' },
            type: { type: 'string', enum: ['feature', 'uat'] },
            slug: { type: 'string' },
            next_action: { type: 'string' },
            command: { type: 'string' },
            run_id: { type: 'string' },
            status: { type: 'string', enum: ['dispatched', 'conflict'] },
          },
          required: ['milestone_slug', 'type', 'command', 'status'],
        },
      },
      reason: {
        type: 'string',
        description: 'Human-readable reason (present when action=idle)',
      },
    },
    required: ['action'],
  },
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

type Mode = 'multi' | 'single'

interface ParallelWorkItem {
  milestone_slug: string
  milestone_title: string
  type: 'feature' | 'uat'
  slug?: string
  next_action?: string
  command: string
}

interface ProjectState {
  parallel_work: ParallelWorkItem[]
}

interface DispatchedItem {
  milestone_slug: string
  type: 'feature' | 'uat'
  slug?: string
  next_action?: string
  command: string
  run_id?: string
  status: 'dispatched' | 'conflict'
}

type DevDriverOutput =
  | { action: 'quality_failing'; failed_checks: string[] }
  | { action: 'dispatched'; mode: Mode; items: DispatchedItem[] }
  | { action: 'idle'; reason: string }

// ---------------------------------------------------------------------------
// Quality check (unchanged from v1)
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

function nodeRuntime(): string {
  return process.env.SDLC_NODE_RUNTIME
    ?? (process.versions.bun ? 'bun' : 'node')
}

function runQualityCheck(root: string): { failed: number; failedNames: string[] } {
  const toolPath = join(root, '.sdlc', 'tools', 'quality-check', 'tool.ts')
  if (!existsSync(toolPath)) {
    log.warn('quality-check tool not found — skipping quality gate')
    return { failed: 0, failedNames: [] }
  }
  try {
    const raw = execSync(`${nodeRuntime()} ${toolPath} --run`, {
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
    log.error(`quality-check execution failed: ${e}`)
    return { failed: -1, failedNames: ['quality-check-tool-error'] }
  }
}

// ---------------------------------------------------------------------------
// Fetch parallel work from server (replaces all CLI subprocess calls)
// ---------------------------------------------------------------------------

async function fetchParallelWork(serverUrl: string, token: string): Promise<ParallelWorkItem[]> {
  const res = await fetch(`${serverUrl}/api/state`, {
    headers: { Authorization: `Bearer ${token}` },
    signal: AbortSignal.timeout(10_000),
  })
  if (!res.ok) {
    throw new Error(`GET /api/state failed: ${res.status} ${res.statusText}`)
  }
  let state: ProjectState
  try {
    state = (await res.json()) as ProjectState
  } catch {
    throw new Error('GET /api/state returned invalid JSON')
  }
  if (!state.parallel_work) {
    log.warn('parallel_work missing from /api/state response — server may be outdated')
  }
  return state.parallel_work ?? []
}

// ---------------------------------------------------------------------------
// Dispatch a single work item via agent-dispatch
// ---------------------------------------------------------------------------

async function dispatchItem(item: ParallelWorkItem): Promise<DispatchedItem> {
  const runKey = item.type === 'uat'
    ? `dev-driver:uat:${item.milestone_slug}`
    : `dev-driver:feature:${item.slug ?? item.milestone_slug}`

  const label = item.type === 'uat'
    ? `dev-driver: UAT ${item.milestone_slug}`
    : `dev-driver: advance ${item.slug}`

  const maxTurns = item.type === 'uat' ? 80 : 40

  const r = await runAgentDispatch(item.command, runKey, label, { maxTurns })

  return {
    milestone_slug: item.milestone_slug,
    type: item.type,
    slug: item.slug,
    next_action: item.next_action,
    command: item.command,
    run_id: r.run_id,
    status: r.status === 'conflict' ? 'conflict' : 'dispatched',
  }
}

// ---------------------------------------------------------------------------
// Main run function
// ---------------------------------------------------------------------------

export async function run(
  input: { mode?: Mode },
  root: string,
): Promise<ToolResult<DevDriverOutput>> {
  const start = Date.now()
  const mode: Mode = input.mode ?? 'multi'

  const serverUrl = process.env.SDLC_SERVER_URL
  const token = process.env.SDLC_AGENT_TOKEN

  if (!serverUrl || !token) {
    return {
      ok: false,
      error: 'SDLC_SERVER_URL and SDLC_AGENT_TOKEN are required (injected by sdlc-server)',
      duration_ms: Date.now() - start,
    }
  }

  // ── Level 1: Quality gate ─────────────────────────────────────────────────
  log.info('running quality check')
  const qc = runQualityCheck(root)
  if (qc.failed !== 0) {
    log.info(`quality failing: ${qc.failedNames.join(', ')}`)
    return {
      ok: true,
      data: { action: 'quality_failing', failed_checks: qc.failedNames },
      duration_ms: Date.now() - start,
    }
  }
  log.info('quality checks passed')

  // ── Level 2: Fetch and dispatch parallel work ─────────────────────────────
  log.info(`fetching parallel work (mode=${mode})`)
  let allWork = await fetchParallelWork(serverUrl, token)

  if (allWork.length === 0) {
    log.info('no parallel work available — idle')
    return {
      ok: true,
      data: { action: 'idle', reason: 'no actionable work found' },
      duration_ms: Date.now() - start,
    }
  }

  // single mode: take only first item
  const workSlots = mode === 'single' ? allWork.slice(0, 1) : allWork

  log.info(`dispatching ${workSlots.length} item(s)`)

  // Dispatch all slots concurrently — 409s are handled per-slot inside dispatchItem
  const dispatched = await Promise.all(workSlots.map(dispatchItem))

  dispatched.forEach(d => {
    if (d.status === 'conflict') {
      log.info(`slot already running: ${d.command} — skipped`)
    } else {
      log.info(`dispatched: ${d.command} (run_id: ${d.run_id})`)
    }
  })

  return {
    ok: true,
    data: { action: 'dispatched', mode, items: dispatched },
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
    .then(raw => run(JSON.parse(raw || '{}') as { mode?: Mode }, root))
    .then(result => {
      console.log(JSON.stringify(result))
      exit(result.ok ? 0 : 1)
    })
    .catch(e => {
      console.log(JSON.stringify({ ok: false, error: String(e) }))
      exit(1)
    })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta or --run.`)
  exit(1)
}
