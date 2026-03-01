/**
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
