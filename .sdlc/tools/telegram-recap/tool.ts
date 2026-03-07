/**
 * telegram-recap
 * ==============
 * Fetches Telegram messages from the sdlc webhook store and emails a daily digest via Resend.
 * Fully self-contained — no sdlc CLI calls.
 *
 * WHAT IT DOES
 * ------------
 * --setup:  Verifies TELEGRAM_BOT_TOKEN via getMe and checks SDLC_SERVER_URL reachability.
 *           Returns { ok: true, data: { bot_username, webhook_store: 'reachable'|'unavailable' } }
 *
 * --run:    Reads JSON from stdin: { window_hours?, dry_run? }
 *           Queries stored webhook payloads from the sdlc server,
 *           builds digest, sends via Resend.
 *           Returns { ok: true, data: { total_messages, chat_count, ... } }
 *
 * --meta:   Writes ToolMeta JSON to stdout.
 *
 * SECRETS (all from: sdlc secrets env export telegram)
 * -----------------------------------------------------
 * TELEGRAM_BOT_TOKEN       Required for setup only (getMe verification)
 * RESEND_API_KEY           Required
 * RESEND_FROM              Required
 * RESEND_TO                Required (comma-separated)
 * SDLC_SERVER_URL          Optional (default: http://localhost:7777)
 * TELEGRAM_WEBHOOK_ROUTE   Optional (default: telegram)
 * WINDOW_HOURS             Optional (default 24)
 */

import type { ToolMeta as BaseToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'

const log = makeLogger('telegram-recap')

// ---------------------------------------------------------------------------
// Extended ToolMeta with secrets, tags, result_actions
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
    'Fetch Telegram messages from the sdlc webhook store and email a daily digest via Resend',
  version: '3.0.0',
  requires_setup: true,
  setup_description:
    'Verifies TELEGRAM_BOT_TOKEN via getMe and checks SDLC_SERVER_URL reachability',
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
    },
  },
  output_schema: {
    type: 'object',
    properties: {
      dry_run: { type: 'boolean' },
      total_messages: { type: 'number' },
      chat_count: { type: 'number' },
      sent_to: { type: 'array', items: { type: 'string' } },
      period_start: { type: 'string' },
      period_end: { type: 'string' },
    },
  },
  secrets: [
    { env_var: 'TELEGRAM_BOT_TOKEN', description: 'Telegram bot API token (from @BotFather) — used for setup verification only', required: true },
    { env_var: 'RESEND_API_KEY', description: 'Resend API key (starts with re_*)', required: true },
    { env_var: 'RESEND_FROM', description: 'Verified sender address (e.g. digest@yourdomain.com)', required: true },
    { env_var: 'RESEND_TO', description: 'Recipient address(es), comma-separated', required: true },
    { env_var: 'SDLC_SERVER_URL', description: 'sdlc server URL (default: http://localhost:7777)', required: false },
    { env_var: 'TELEGRAM_WEBHOOK_ROUTE', description: 'Webhook route name on the sdlc server (default: telegram)', required: false },
    { env_var: 'WINDOW_HOURS', description: 'Default digest window in hours (optional, default 24)', required: false },
  ],
  tags: ['telegram', 'email', 'digest'],
  result_actions: [
    {
      label: 'Send test digest',
      icon: 'send',
      condition: '$.ok == true',
      prompt_template: 'Run the telegram-recap tool with dry_run: true to preview the digest.',
      confirm: 'This will fetch messages from the webhook store and display the digest without sending email.',
    },
  ],
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface Input {
  window_hours?: number
  dry_run?: boolean
}

interface DigestOutput {
  dry_run: boolean
  total_messages: number
  chat_count: number
  period_start: string
  period_end: string
  sent_to: string[]
}

// Raw payload item returned by GET /api/webhooks/{route}/data
interface WebhookPayloadItem {
  id: string
  route_path: string
  received_at: string
  content_type: string
  body: string // base64-encoded
}

// Telegram Update shape — only the fields we need
interface TelegramUpdate {
  update_id: number
  message?: {
    message_id: number
    date: number
    text?: string
    caption?: string
    from?: { id: number; first_name: string; last_name?: string; username?: string }
    chat: { id: number; title?: string; first_name?: string; type: string }
  }
}

interface TelegramMessage {
  update_id: number
  date: number
  text: string
  from_name: string
  chat_id: number
  chat_title: string
}

// ---------------------------------------------------------------------------
// Webhook store helpers
// ---------------------------------------------------------------------------

/**
 * Fetch stored Telegram webhook payloads from the sdlc server.
 * GET /api/webhooks/{route}/data?since={iso}&limit=200
 * Each item's `body` is base64-encoded Telegram Update JSON.
 */
async function fetchStoredPayloads(
  serverUrl: string,
  route: string,
  since: string,
): Promise<TelegramUpdate[]> {
  const url = `${serverUrl}/api/webhooks/${route}/data?since=${encodeURIComponent(since)}&limit=200`
  log.info(`Fetching webhook payloads: GET ${url}`)

  let res: Response
  try {
    res = await fetch(url, { signal: AbortSignal.timeout(15000) })
  } catch (e) {
    throw new Error(`Failed to reach sdlc server at ${serverUrl}: ${e}`)
  }

  if (!res.ok) {
    const txt = await res.text()
    throw new Error(`Webhook store query failed (${res.status}): ${txt}`)
  }

  const items = await res.json() as WebhookPayloadItem[]
  log.info(`Received ${items.length} webhook payload(s)`)

  const updates: TelegramUpdate[] = []
  for (const item of items) {
    try {
      const decoded = atob(item.body)
      const update = JSON.parse(decoded) as TelegramUpdate
      updates.push(update)
    } catch (e) {
      log.warn(`Skipping payload ${item.id}: failed to decode/parse body — ${e}`)
    }
  }

  return updates
}

/**
 * Check that the webhook store endpoint is reachable (no since param — just a probe).
 */
async function probeWebhookStore(serverUrl: string, route: string): Promise<boolean> {
  try {
    const url = `${serverUrl}/api/webhooks/${route}/data?limit=1`
    const res = await fetch(url, { signal: AbortSignal.timeout(5000) })
    return res.ok
  } catch {
    return false
  }
}

// ---------------------------------------------------------------------------
// Telegram API helpers
// ---------------------------------------------------------------------------

async function telegramGetMe(token: string): Promise<{ username: string }> {
  const res = await fetch(`https://api.telegram.org/bot${token}/getMe`, {
    signal: AbortSignal.timeout(10000),
  })
  if (!res.ok) {
    const txt = await res.text()
    throw new Error(`getMe failed (${res.status}): ${txt}`)
  }
  const json = await res.json() as { ok: boolean; result?: { username?: string } }
  if (!json.ok) throw new Error('getMe returned ok=false')
  return { username: json.result?.username ?? 'unknown' }
}

// ---------------------------------------------------------------------------
// Resend email helper
// ---------------------------------------------------------------------------

async function sendEmail(opts: {
  apiKey: string
  from: string
  to: string[]
  subject: string
  text: string
  html: string
}): Promise<void> {
  const res = await fetch('https://api.resend.com/emails', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${opts.apiKey}`,
    },
    body: JSON.stringify({
      from: opts.from,
      to: opts.to,
      subject: opts.subject,
      text: opts.text,
      html: opts.html,
    }),
    signal: AbortSignal.timeout(15000),
  })
  if (!res.ok) {
    const txt = await res.text()
    throw new Error(`Resend delivery failed (${res.status}): ${txt}`)
  }
}

// ---------------------------------------------------------------------------
// Digest builders
// ---------------------------------------------------------------------------

function buildMessages(updates: TelegramUpdate[]): TelegramMessage[] {
  return updates
    .filter(u => u.message)
    .map(u => {
      const msg = u.message!
      const fromName = msg.from
        ? [msg.from.first_name, msg.from.last_name].filter(Boolean).join(' ')
        : 'Unknown'
      const chatTitle =
        msg.chat.title ?? msg.chat.first_name ?? `Chat ${msg.chat.id}`
      return {
        update_id: u.update_id,
        date: msg.date,
        text: msg.text ?? msg.caption ?? '',
        from_name: fromName,
        chat_id: msg.chat.id,
        chat_title: chatTitle,
      }
    })
    .filter(m => m.text.length > 0)
}

function buildDigest(
  messages: TelegramMessage[],
  windowHours: number,
  sinceTimestamp: number,
  nowTimestamp: number,
): { text: string; html: string; chatCount: number } {
  // Group by chat
  const byChat = new Map<number, { title: string; msgs: TelegramMessage[] }>()
  for (const m of messages) {
    if (!byChat.has(m.chat_id)) {
      byChat.set(m.chat_id, { title: m.chat_title, msgs: [] })
    }
    byChat.get(m.chat_id)!.msgs.push(m)
  }

  const periodStart = new Date(sinceTimestamp * 1000).toISOString()
  const periodEnd = new Date(nowTimestamp * 1000).toISOString()

  let text = `Telegram Digest — last ${windowHours}h\n`
  text += `Period: ${periodStart} → ${periodEnd}\n`
  text += `Total: ${messages.length} messages across ${byChat.size} chat(s)\n\n`

  let html = `<h2>Telegram Digest — last ${windowHours}h</h2>`
  html += `<p><small>${periodStart} → ${periodEnd}</small></p>`
  html += `<p><strong>${messages.length} messages across ${byChat.size} chat(s)</strong></p>`

  for (const [, { title, msgs }] of byChat) {
    text += `--- ${title} (${msgs.length} messages) ---\n`
    html += `<h3>${escapeHtml(title)} (${msgs.length})</h3><ul>`
    for (const m of msgs) {
      const ts = new Date(m.date * 1000).toISOString()
      text += `[${ts}] ${m.from_name}: ${m.text}\n`
      html += `<li><small>${ts}</small> <strong>${escapeHtml(m.from_name)}:</strong> ${escapeHtml(m.text)}</li>`
    }
    text += '\n'
    html += '</ul>'
  }

  return { text, html, chatCount: byChat.size }
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

// ---------------------------------------------------------------------------
// setup() — verify credentials and reachability
// ---------------------------------------------------------------------------

export async function setup(): Promise<ToolResult<{ bot_username: string; webhook_store: string }>> {
  const start = Date.now()
  const token = process.env.TELEGRAM_BOT_TOKEN
  const resendKey = process.env.RESEND_API_KEY
  const resendFrom = process.env.RESEND_FROM
  const resendTo = process.env.RESEND_TO
  const serverUrl = process.env.SDLC_SERVER_URL ?? 'http://localhost:7777'
  const route = process.env.TELEGRAM_WEBHOOK_ROUTE ?? 'telegram'

  if (!token) {
    return { ok: false, error: 'TELEGRAM_BOT_TOKEN is not set', duration_ms: Date.now() - start }
  }
  if (!resendKey || !resendFrom || !resendTo) {
    return {
      ok: false,
      error: 'Missing required Resend secrets: RESEND_API_KEY, RESEND_FROM, RESEND_TO',
      duration_ms: Date.now() - start,
    }
  }

  log.info('Verifying Telegram bot token via getMe...')
  let botUsername: string
  try {
    const me = await telegramGetMe(token)
    botUsername = me.username
    log.info(`Bot verified: @${botUsername}`)
  } catch (e) {
    return { ok: false, error: `Telegram getMe failed: ${e}`, duration_ms: Date.now() - start }
  }

  log.info(`Probing sdlc webhook store at ${serverUrl}/api/webhooks/${route}/data...`)
  const reachable = await probeWebhookStore(serverUrl, route)
  const webhookStoreStatus = reachable ? 'reachable' : 'unavailable'
  log.info(`Webhook store: ${webhookStoreStatus}`)

  return {
    ok: true,
    data: { bot_username: botUsername, webhook_store: webhookStoreStatus },
    duration_ms: Date.now() - start,
  }
}

// ---------------------------------------------------------------------------
// run() — fetch from webhook store, build digest, send
// ---------------------------------------------------------------------------

export async function run(input: Input): Promise<ToolResult<DigestOutput>> {
  const start = Date.now()
  const resendKey = process.env.RESEND_API_KEY
  const resendFrom = process.env.RESEND_FROM
  const resendTo = process.env.RESEND_TO
  const serverUrl = process.env.SDLC_SERVER_URL ?? 'http://localhost:7777'
  const route = process.env.TELEGRAM_WEBHOOK_ROUTE ?? 'telegram'

  if (!resendKey || !resendFrom || !resendTo) {
    return {
      ok: false,
      error: 'Missing required Resend secrets: RESEND_API_KEY, RESEND_FROM, RESEND_TO',
      duration_ms: Date.now() - start,
    }
  }

  const windowHours = input.window_hours ?? Number(process.env.WINDOW_HOURS ?? '24')
  const dryRun = input.dry_run ?? false
  const now = Math.floor(Date.now() / 1000)
  const sinceTimestamp = now - windowHours * 3600
  const sinceIso = new Date(sinceTimestamp * 1000).toISOString()

  // 1. Fetch stored webhook payloads from the sdlc server
  log.info(`Fetching stored webhook payloads since ${sinceIso} (window: ${windowHours}h)...`)
  let updates: TelegramUpdate[]
  try {
    updates = await fetchStoredPayloads(serverUrl, route, sinceIso)
  } catch (e) {
    return { ok: false, error: `Webhook store query failed: ${e}`, duration_ms: Date.now() - start }
  }
  log.info(`Decoded ${updates.length} Telegram update(s) from webhook payloads`)

  // 2. Build TelegramMessage list from updates
  const windowMessages = buildMessages(updates)
  log.info(`${windowMessages.length} message(s) with text content`)

  // 3. Build digest
  const { text: digestText, html: digestHtml, chatCount } = buildDigest(
    windowMessages,
    windowHours,
    sinceTimestamp,
    now,
  )

  const periodStart = new Date(sinceTimestamp * 1000).toISOString()
  const periodEnd = new Date(now * 1000).toISOString()

  if (windowMessages.length === 0) {
    log.info('No messages in window — skipping send')
    return {
      ok: true,
      data: {
        dry_run: dryRun,
        total_messages: 0,
        chat_count: 0,
        period_start: periodStart,
        period_end: periodEnd,
        sent_to: [],
      },
      duration_ms: Date.now() - start,
    }
  }

  const recipients = resendTo.split(',').map((s: string) => s.trim()).filter(Boolean)

  // 4. Send or dry-run
  if (dryRun) {
    log.info(`Dry run — digest preview:\n${digestText}`)
    return {
      ok: true,
      data: {
        dry_run: true,
        total_messages: windowMessages.length,
        chat_count: chatCount,
        period_start: periodStart,
        period_end: periodEnd,
        sent_to: [],
      },
      duration_ms: Date.now() - start,
    }
  }

  const subject = `Telegram Digest — ${windowMessages.length} messages (last ${windowHours}h)`
  log.info(`Sending digest to: ${recipients.join(', ')}`)
  try {
    await sendEmail({
      apiKey: resendKey,
      from: resendFrom,
      to: recipients,
      subject,
      text: digestText,
      html: digestHtml,
    })
  } catch (e) {
    return { ok: false, error: `${e}`, duration_ms: Date.now() - start }
  }

  log.info('Digest sent successfully')

  return {
    ok: true,
    data: {
      dry_run: false,
      total_messages: windowMessages.length,
      chat_count: chatCount,
      period_start: periodStart,
      period_end: periodEnd,
      sent_to: recipients,
    },
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
  setup()
    .then(result => {
      console.log(JSON.stringify(result))
      exit(result.ok ? 0 : 1)
    })
    .catch(e => {
      console.log(JSON.stringify({ ok: false, error: String(e) }))
      exit(1)
    })
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
  console.error(`Unknown mode: ${mode}. Use --meta, --setup, or --run.`)
  exit(1)
}
