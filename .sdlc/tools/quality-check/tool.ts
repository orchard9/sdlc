/**
 * Quality Check
 * =============
 * Runs platform shell commands from .sdlc/config.yaml and reports pass/fail.
 *
 * WHAT IT DOES
 * ------------
 * --run:   Reads JSON from stdin: { "scope"?: "string" }
 *          Loads platform.commands from .sdlc/config.yaml via `sdlc config show --json`.
 *          Runs each command's script as a shell command, records pass/fail + output.
 *          If scope is provided, only runs checks whose name matches the filter string.
 *          Returns ToolResult<{ passed, failed, checks[] }>.
 *
 * --meta:  Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 *
 * WHAT IT READS
 * -------------
 * - .sdlc/config.yaml (via `sdlc config show --json`)
 *   → platform.commands[]: { name, description, script }
 * - .sdlc/tools/quality-check/config.yaml (tool-level config, currently unused)
 *
 * WHAT IT WRITES
 * --------------
 * - STDERR: structured log lines via _shared/log.ts
 * - STDOUT: JSON only (ToolResult shape from _shared/types.ts)
 *
 * EXTENDING
 * ---------
 * Add checks under platform.commands in .sdlc/config.yaml:
 *   platform:
 *     commands:
 *       - name: test
 *         description: Run unit tests
 *         script: cargo test --all
 * The quality-check tool picks them up automatically — no code changes needed.
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { loadToolConfig } from '../_shared/config.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import { execSync } from 'node:child_process'
import { join } from 'node:path'

const log = makeLogger('quality-check')

// ---------------------------------------------------------------------------
// Config (tool-level — currently no per-tool settings)
// ---------------------------------------------------------------------------

interface QualityCheckConfig {
  name: string
  version: string
}

const DEFAULT_CONFIG: QualityCheckConfig = {
  name: 'quality-check',
  version: '0.1.0',
}

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'quality-check',
  display_name: 'Quality Check',
  description: 'Runs platform shell commands from .sdlc/config.yaml and reports pass/fail',
  version: '0.1.0',
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

interface SdlcConfig {
  platform?: {
    commands?: PlatformCommand[]
  }
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
// Run — execute platform checks
// ---------------------------------------------------------------------------

export async function run(
  input: { scope?: string },
  root: string,
): Promise<ToolResult<QualityCheckOutput>> {
  const start = Date.now()
  loadToolConfig(root, 'quality-check', DEFAULT_CONFIG)

  // Load sdlc config via CLI to avoid YAML parsing dependency
  let sdlcConfig: SdlcConfig = {}
  try {
    const raw = execSync('sdlc config show --json', {
      cwd: root,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
    })
    sdlcConfig = JSON.parse(raw) as SdlcConfig
  } catch (e) {
    log.warn(`Could not load sdlc config: ${e}`)
  }

  const commands = sdlcConfig?.platform?.commands ?? []

  if (commands.length === 0) {
    log.warn('No platform commands configured in .sdlc/config.yaml — nothing to check')
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
