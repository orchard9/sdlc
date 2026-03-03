/**
 * restart-sdlc
 * ============
 * Kills any process on port 7777, rebuilds the project with `make install`,
 * then relaunches `sdlc ui --port 7777` in the background.
 *
 * WHAT IT DOES
 * ------------
 * --run:   No required input. Runs three steps in sequence:
 *            1. Kill any process bound to :7777 (SIGKILL via lsof)
 *            2. `make install` in the project root (blocking, stdout+stderr captured)
 *            3. `sdlc ui --port 7777` (detached, unref'd — tool exits immediately after)
 *          Returns JSON summary with killed PIDs, make output, and the new UI pid.
 *
 * --meta:  Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 *
 * WHAT IT READS
 * -------------
 * - $SDLC_ROOT / process.cwd() — project root (must contain a Makefile)
 *
 * WHAT IT WRITES
 * --------------
 * - STDERR: structured log lines via _shared/log.ts
 * - STDOUT: JSON only (ToolResult shape)
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import { spawnSync, spawn } from 'node:child_process'

const log = makeLogger('restart-sdlc')

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'restart-sdlc',
  display_name: 'Restart SDLC',
  description: 'Kills port 7777, rebuilds with make install, and relaunches sdlc ui on port 7777',
  version: '0.1.0',
  requires_setup: false,
  input_schema: {
    type: 'object',
    required: [],
    properties: {
      port: {
        type: 'number',
        description: 'Port to kill and relaunch on (default: 7777)',
      },
    },
  },
  output_schema: {
    type: 'object',
    properties: {
      killed_pids: {
        type: 'array',
        items: { type: 'number' },
        description: 'PIDs that were killed on the target port',
      },
      make_output: {
        type: 'string',
        description: 'Combined stdout+stderr from make install',
      },
      make_exit_code: {
        type: 'number',
        description: 'Exit code from make install (0 = success)',
      },
      ui_pid: {
        type: 'number',
        description: 'PID of the newly launched sdlc ui process',
      },
    },
  },
}

// ---------------------------------------------------------------------------
// Input / output types
// ---------------------------------------------------------------------------

interface Input {
  port?: number
}

interface Output {
  killed_pids: number[]
  make_output: string
  make_exit_code: number
  ui_pid: number
}

// ---------------------------------------------------------------------------
// run()
// ---------------------------------------------------------------------------

export async function run(input: Input): Promise<ToolResult<Output>> {
  const start = Date.now()
  const port = input.port ?? 7777
  const root = process.env.SDLC_ROOT ?? process.cwd()

  // ── Step 1: Kill anything on the port ─────────────────────────────────────
  log.info(`step 1: kill processes on port ${port}`)

  const killed_pids: number[] = []

  const lsof = spawnSync('lsof', ['-ti', `:${port}`], { encoding: 'utf8' })
  if (lsof.status === 0 && lsof.stdout.trim()) {
    const pids = lsof.stdout
      .trim()
      .split('\n')
      .map(s => parseInt(s.trim(), 10))
      .filter(n => !isNaN(n))

    for (const pid of pids) {
      log.info(`killing pid ${pid}`)
      const k = spawnSync('kill', ['-9', String(pid)])
      if (k.status === 0) {
        killed_pids.push(pid)
      } else {
        log.warn(`kill ${pid} exited ${k.status}: ${k.stderr ?? ''}`)
      }
    }
    log.info(`killed ${killed_pids.length} process(es)`)
  } else {
    log.info(`no processes found on port ${port}`)
  }

  // Brief pause so the port is fully released before we bind again
  await new Promise(r => setTimeout(r, 500))

  // ── Step 2: make install ───────────────────────────────────────────────────
  log.info('step 2: make install')

  const make = spawnSync('make', ['install'], {
    cwd: root,
    encoding: 'utf8',
    env: { ...process.env },
  })

  const make_output = [make.stdout ?? '', make.stderr ?? ''].join('').trim()
  const make_exit_code = make.status ?? 1

  if (make_exit_code !== 0) {
    log.error(`make install failed (exit ${make_exit_code})`)
    return {
      ok: false,
      error: `make install failed with exit code ${make_exit_code}:\n${make_output}`,
      data: { killed_pids, make_output, make_exit_code, ui_pid: -1 },
      duration_ms: Date.now() - start,
    }
  }

  log.info('make install succeeded')

  // ── Step 3: sdlc ui --port <port> (detached) ──────────────────────────────
  log.info(`step 3: launching sdlc ui --port ${port}`)

  const ui = spawn('sdlc', ['ui', '--port', String(port)], {
    cwd: root,
    detached: true,
    stdio: 'ignore',
    env: { ...process.env },
  })
  ui.unref()

  const ui_pid = ui.pid ?? -1
  log.info(`sdlc ui launched with pid ${ui_pid}`)

  return {
    ok: true,
    data: { killed_pids, make_output, make_exit_code, ui_pid },
    duration_ms: Date.now() - start,
  }
}

// ---------------------------------------------------------------------------
// CLI entrypoint
// ---------------------------------------------------------------------------

const mode = getArgs()[0] ?? '--run'

if (mode === '--meta') {
  console.log(JSON.stringify(meta))
  exit(0)
} else if (mode === '--run') {
  readStdin()
    .then(raw => run(JSON.parse(raw || '{}') as Input))
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
