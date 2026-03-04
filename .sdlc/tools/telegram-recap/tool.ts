/**
 * telegram-recap
 * ==============
 * Fetches Telegram chat messages from the configured window and emails a digest via Resend.
 * Delegates all logic to `sdlc telegram digest --json`.
 *
 * WHAT IT DOES
 * ------------
 * --setup:  Runs `sdlc telegram status` to verify bot token and DB connectivity.
 *           Returns success/failure with bot identity.
 *
 * --run:    Reads JSON from stdin: { window_hours?, dry_run?, chat_ids? }
 *           Builds args and spawns `sdlc telegram digest --json [args]`.
 *           Maps the digest JSON summary → ToolResult.
 *
 * --meta:   Writes ToolMeta JSON to stdout. Declares required secrets.
 *
 * SECRETS (all sourced from: sdlc secrets env export telegram)
 * ------------------------------------------------------------
 * TELEGRAM_BOT_TOKEN   Required — Telegram bot API token (from @BotFather)
 * RESEND_API_KEY       Required — Resend API key (starts with re_*)
 * RESEND_FROM          Required — Verified sender address (e.g. digest@yourdomain.com)
 * RESEND_TO            Required — Recipient address(es), comma-separated
 * TELEGRAM_CHAT_IDS    Optional — Comma-separated chat IDs (overrides config.yaml list)
 *
 * CHAT IDS
 * --------
 * Chat IDs identify which Telegram chats to include in the digest. The format is:
 *   - Private chats: positive integer (e.g. "123456789")
 *   - Groups / supergroups: negative integer (e.g. "-100123456789")
 *   - Channels: negative integer prefixed with -100 (e.g. "-1001234567890")
 *
 * How to find your chat ID:
 *   1. Add your bot to the chat (it must be a member to receive messages)
 *   2. Send a message in the chat
 *   3. Run: sdlc telegram poll  (leave it running a few seconds, then Ctrl-C)
 *   4. Query the DB for stored chat IDs:
 *        sqlite3 .sdlc/telegram/messages.db "SELECT DISTINCT chat_id, chat_title FROM messages LIMIT 20;"
 *   5. Store discovered IDs:
 *        sdlc secrets env set telegram TELEGRAM_CHAT_IDS="-100123456789,-100987654321"
 *
 * WHAT IT READS
 * -------------
 * - $SDLC_ROOT/.sdlc/telegram/messages.db   (populated by `sdlc telegram poll`)
 *
 * WHAT IT WRITES
 * --------------
 * - STDERR: structured log lines via _shared/log.ts
 * - STDOUT: JSON only (ToolResult shape)
 */

import type { ToolMeta as BaseToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import { spawnSync } from 'node:child_process'

const log = makeLogger('telegram-recap')

// ---------------------------------------------------------------------------
// Extended ToolMeta type with secrets, tags, result_actions, secrets_env
// ---------------------------------------------------------------------------

interface ToolMeta extends BaseToolMeta {
  secrets?: Array<{ env_var: string; description: string; required?: boolean }>
  secrets_env?: string
  tags?: string[]
  result_actions?: Array<{
    label: string
    icon?: string
    condition?: string
    prompt_template: string
    confirm?: string
  }>
}

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'telegram-recap',
  display_name: 'Telegram Recap',
  description:
    'Fetch and email a Telegram chat digest — pulls messages from the configured window and sends via Resend',
  version: '1.1.0',
  requires_setup: true,
  setup_description:
    'Verifies TELEGRAM_BOT_TOKEN by calling Telegram getMe and checking database connectivity',
  // All secrets sourced from: sdlc secrets env export telegram
  secrets_env: 'telegram',
  input_schema: {
    type: 'object',
    required: [],
    properties: {
      window_hours: {
        type: 'number',
        description: 'Time window in hours to include in the digest (default: 24)',
      },
      dry_run: {
        type: 'boolean',
        description: 'Print digest to stdout without sending email',
      },
      chat_ids: {
        type: 'array',
        items: { type: 'string' },
        description:
          'Override configured chat IDs for this run. ' +
          'Format: negative integers for groups/channels (e.g. ["-100123456789"]), ' +
          'positive integers for private chats. ' +
          'If omitted, uses TELEGRAM_CHAT_IDS env var or the chat_ids list in .sdlc/config.yaml. ' +
          'To discover IDs: run `sdlc telegram poll`, then: ' +
          'sqlite3 .sdlc/telegram/messages.db "SELECT DISTINCT chat_id, chat_title FROM messages LIMIT 20;"',
      },
    },
  },
  output_schema: {
    type: 'object',
    properties: {
      dry_run: { type: 'boolean', description: 'Whether this was a dry run' },
      total_messages: { type: 'number', description: 'Messages included in digest' },
      chat_count: { type: 'number', description: 'Number of chats included' },
      period_start: { type: 'string', description: 'ISO 8601 period start timestamp' },
      period_end: { type: 'string', description: 'ISO 8601 period end timestamp' },
      sent_to: {
        type: 'array',
        items: { type: 'string' },
        description: 'Recipient addresses (empty on dry run)',
      },
    },
  },
  secrets: [
    {
      env_var: 'TELEGRAM_BOT_TOKEN',
      description: 'Telegram bot API token (from @BotFather). Store with: sdlc secrets env set telegram TELEGRAM_BOT_TOKEN=<value>',
      required: true,
    },
    {
      env_var: 'RESEND_API_KEY',
      description: 'Resend API key (starts with re_*). Store with: sdlc secrets env set telegram RESEND_API_KEY=<value>',
      required: true,
    },
    {
      env_var: 'RESEND_FROM',
      description:
        'Verified sender address for digest emails (e.g. digest@yourdomain.com). ' +
        'Must be a domain verified in your Resend account. Store with: sdlc secrets env set telegram RESEND_FROM=<value>',
      required: true,
    },
    {
      env_var: 'RESEND_TO',
      description: 'Recipient address(es), comma-separated. Store with: sdlc secrets env set telegram RESEND_TO=<value>',
      required: true,
    },
    {
      env_var: 'TELEGRAM_CHAT_IDS',
      description:
        'Comma-separated list of Telegram chat IDs to digest (e.g. "-100123456789,-100987654321"). ' +
        'Groups/channels use negative IDs. Find IDs by running `sdlc telegram poll` then querying messages.db. ' +
        'Store with: sdlc secrets env set telegram TELEGRAM_CHAT_IDS=<value>',
      required: false,
    },
  ],
  tags: ['telegram', 'email', 'digest'],
  result_actions: [
    {
      label: 'Send test digest',
      icon: 'send',
      condition: '$.ok == true',
      prompt_template:
        'Run the telegram-recap tool with dry_run: true to preview the digest without sending email.',
      confirm: 'This will fetch messages and display the digest without sending email.',
    },
  ],
}

// ---------------------------------------------------------------------------
// Input / output types
// ---------------------------------------------------------------------------

interface Input {
  window_hours?: number
  dry_run?: boolean
  chat_ids?: string[]
}

interface DigestOutput {
  dry_run: boolean
  total_messages: number
  chat_count: number
  period_start: string
  period_end: string
  sent_to: string[]
}

// ---------------------------------------------------------------------------
// setup() — verify bot token and database connectivity
// ---------------------------------------------------------------------------

export function setup(): ToolResult<{ status_output: string }> {
  const start = Date.now()
  log.info('running: sdlc telegram status')

  const proc = spawnSync('sdlc', ['telegram', 'status'], {
    env: process.env,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  })

  const stdout = (proc.stdout ?? '').trim()
  const stderr = (proc.stderr ?? '').trim()
  const exitCode = proc.status ?? 1

  if (exitCode !== 0) {
    log.error(`sdlc telegram status failed (exit ${exitCode}): ${stderr}`)
    return {
      ok: false,
      error: `Bot token check failed (exit ${exitCode}): ${stderr || stdout || 'no output'}`,
      duration_ms: Date.now() - start,
    }
  }

  log.info(`setup ok:\n${stdout}`)
  return {
    ok: true,
    data: { status_output: stdout },
    duration_ms: Date.now() - start,
  }
}

// ---------------------------------------------------------------------------
// run() — fetch messages and send (or preview) the digest
// ---------------------------------------------------------------------------

export function run(input: Input): ToolResult<DigestOutput> {
  const start = Date.now()

  const args: string[] = ['telegram', 'digest', '--json']

  if (input.dry_run) {
    args.push('--dry-run')
  }
  if (input.window_hours !== undefined) {
    args.push('--window', String(Math.round(input.window_hours)))
  }
  if (input.chat_ids && input.chat_ids.length > 0) {
    for (const id of input.chat_ids) {
      args.push('--chat', id)
    }
  }

  log.info(`running: sdlc ${args.join(' ')}`)

  const proc = spawnSync('sdlc', args, {
    env: process.env,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  })

  const stdout = (proc.stdout ?? '').trim()
  const stderr = (proc.stderr ?? '').trim()
  const exitCode = proc.status ?? 1

  if (exitCode !== 0) {
    log.error(`sdlc telegram digest failed (exit ${exitCode})`)
    return {
      ok: false,
      error: `sdlc telegram digest failed (exit ${exitCode}): ${stderr || stdout || 'no output'}`,
      duration_ms: Date.now() - start,
    }
  }

  let parsed: DigestOutput
  try {
    parsed = JSON.parse(stdout) as DigestOutput
  } catch (e) {
    return {
      ok: false,
      error: `Failed to parse digest JSON output: ${e}. Raw stdout: ${stdout}`,
      duration_ms: Date.now() - start,
    }
  }

  log.info(
    `digest ok: ${parsed.total_messages} messages across ${parsed.chat_count} chats, sent_to=${JSON.stringify(parsed.sent_to)}`,
  )
  return {
    ok: true,
    data: parsed,
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
} else if (mode === '--setup') {
  const result = setup()
  console.log(JSON.stringify(result))
  exit(result.ok ? 0 : 1)
} else if (mode === '--run') {
  readStdin()
    .then(raw => {
      const result = run(JSON.parse(raw || '{}') as Input)
      console.log(JSON.stringify(result))
      exit(result.ok ? 0 : 1)
    })
    .catch(e => {
      console.log(JSON.stringify({ ok: false, error: String(e) }))
      exit(1)
    })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta, --setup, or --run.`)
  exit(1)
}
