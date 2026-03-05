/**
 * telegram-recap
 * ==============
 * Fetches Telegram messages via the Bot API, optionally persists them to CouchDB,
 * and emails a digest via Resend. Fully self-contained — no sdlc CLI calls.
 *
 * WHAT IT DOES
 * ------------
 * --setup:  Verifies TELEGRAM_BOT_TOKEN via getMe, checks Resend env vars,
 *           and optionally probes CouchDB (non-fatal if unavailable).
 *           Returns { ok: true, data: { bot_username, couchdb: '...' } }
 *
 * --run:    Reads JSON from stdin: { window_hours?, dry_run? }
 *           Calls Telegram getUpdates, optionally persists to CouchDB,
 *           builds digest, sends via Resend.
 *           Returns { ok: true, data: { total_messages, chat_count, ... } }
 *
 * --meta:   Writes ToolMeta JSON to stdout.
 *
 * SECRETS (all from: sdlc secrets env export telegram)
 * -----------------------------------------------------
 * TELEGRAM_BOT_TOKEN   Required
 * RESEND_API_KEY       Required
 * RESEND_FROM          Required
 * RESEND_TO            Required (comma-separated)
 * COUCHDB_URL          Optional (e.g. http://couchdb.threesix.svc.cluster.local:5984)
 * COUCHDB_USER         Optional
 * COUCHDB_PASSWORD     Optional
 * WINDOW_HOURS         Optional (default 24)
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
    'Fetch and email a Telegram chat digest — pulls messages from the configured window and sends via Resend',
  version: '2.0.0',
  requires_setup: true,
  setup_description:
    'Verifies TELEGRAM_BOT_TOKEN via getMe, checks Resend credentials, and probes optional CouchDB',
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
    { env_var: 'TELEGRAM_BOT_TOKEN', description: 'Telegram bot API token (from @BotFather)', required: true },
    { env_var: 'RESEND_API_KEY', description: 'Resend API key (starts with re_*)', required: true },
    { env_var: 'RESEND_FROM', description: 'Verified sender address (e.g. digest@yourdomain.com)', required: true },
    { env_var: 'RESEND_TO', description: 'Recipient address(es), comma-separated', required: true },
    { env_var: 'COUCHDB_URL', description: 'CouchDB URL for message persistence (optional)', required: false },
    { env_var: 'COUCHDB_USER', description: 'CouchDB username (optional)', required: false },
    { env_var: 'COUCHDB_PASSWORD', description: 'CouchDB password (optional)', required: false },
    { env_var: 'WINDOW_HOURS', description: 'Default digest window in hours (optional, default 24)', required: false },
  ],
  tags: ['telegram', 'email', 'digest'],
  result_actions: [
    {
      label: 'Send test digest',
      icon: 'send',
      condition: '$.ok == true',
      prompt_template: 'Run the telegram-recap tool with dry_run: true to preview the digest.',
      confirm: 'This will fetch messages and display the digest without sending email.',
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
// CouchDB helpers (graceful degradation)
// ---------------------------------------------------------------------------

function couchdbAuth(): string | undefined {
  const user = process.env.COUCHDB_USER
  const pass = process.env.COUCHDB_PASSWORD
  if (user && pass) {
    return 'Basic ' + btoa(`${user}:${pass}`)
  }
  return undefined
}

async function couchdbPing(baseUrl: string): Promise<boolean> {
  try {
    const headers: Record<string, string> = { 'Content-Type': 'application/json' }
    const auth = couchdbAuth()
    if (auth) headers['Authorization'] = auth
    const res = await fetch(baseUrl, { headers, signal: AbortSignal.timeout(5000) })
    return res.ok
  } catch {
    return false
  }
}

async function couchdbEnsureDb(baseUrl: string, db: string): Promise<void> {
  const url = `${baseUrl}/${db}`
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  const auth = couchdbAuth()
  if (auth) headers['Authorization'] = auth
  // PUT is idempotent — 201 = created, 412 = already exists, both are fine
  await fetch(url, { method: 'PUT', headers, signal: AbortSignal.timeout(5000) })
}

async function couchdbUpsertMessages(baseUrl: string, db: string, messages: TelegramMessage[]): Promise<void> {
  if (messages.length === 0) return
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  const auth = couchdbAuth()
  if (auth) headers['Authorization'] = auth

  // Fetch existing _revs to allow updates
  const ids = messages.map(m => String(m.update_id))
  const keysRes = await fetch(`${baseUrl}/${db}/_all_docs`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ keys: ids }),
    signal: AbortSignal.timeout(10000),
  })
  const keysJson = keysRes.ok ? (await keysRes.json() as { rows: Array<{ id: string; value?: { rev: string }; error?: string }> }) : { rows: [] }
  const revMap: Record<string, string> = {}
  for (const row of keysJson.rows) {
    if (row.value?.rev) revMap[row.id] = row.value.rev
  }

  const docs = messages.map(m => ({
    _id: String(m.update_id),
    ...(revMap[String(m.update_id)] ? { _rev: revMap[String(m.update_id)] } : {}),
    ...m,
  }))

  const bulkRes = await fetch(`${baseUrl}/${db}/_bulk_docs`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ docs }),
    signal: AbortSignal.timeout(10000),
  })
  if (bulkRes.ok) {
    const bulkJson = await bulkRes.json() as Array<{ ok?: boolean; id?: string; error?: string }>
    for (const r of bulkJson) {
      if (!r.ok) log.warn(`Failed to upsert doc ${r.id}: ${r.error}`)
    }
  }
}

async function couchdbEnsureIndex(baseUrl: string, db: string): Promise<void> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  const auth = couchdbAuth()
  if (auth) headers['Authorization'] = auth
  // PUT _index is idempotent — safe to call on every run
  await fetch(`${baseUrl}/${db}/_index`, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      index: { fields: ['date'] },
      name: 'by-date',
      type: 'json',
    }),
    signal: AbortSignal.timeout(5000),
  })
}

async function couchdbQueryWindow(
  baseUrl: string,
  db: string,
  sinceTimestamp: number,
): Promise<TelegramMessage[]> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  const auth = couchdbAuth()
  if (auth) headers['Authorization'] = auth

  const res = await fetch(`${baseUrl}/${db}/_find`, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      selector: { date: { $gte: sinceTimestamp } },
      limit: 10000,
      sort: [{ date: 'asc' }],
    }),
    signal: AbortSignal.timeout(10000),
  })
  if (!res.ok) return []
  const json = await res.json() as { docs?: TelegramMessage[] }
  return json.docs ?? []
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

async function telegramGetUpdates(token: string, offset?: number): Promise<TelegramUpdate[]> {
  const body: Record<string, unknown> = { timeout: 0, allowed_updates: ['message'] }
  if (offset !== undefined) body.offset = offset
  const res = await fetch(`https://api.telegram.org/bot${token}/getUpdates`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
    signal: AbortSignal.timeout(30000),
  })
  if (!res.ok) {
    const txt = await res.text()
    throw new Error(`getUpdates failed (${res.status}): ${txt}`)
  }
  const json = await res.json() as { ok: boolean; result?: TelegramUpdate[] }
  if (!json.ok) throw new Error('getUpdates returned ok=false')
  return json.result ?? []
}

async function telegramAdvanceOffset(token: string, maxUpdateId: number): Promise<void> {
  await fetch(`https://api.telegram.org/bot${token}/getUpdates`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ offset: maxUpdateId + 1, timeout: 0 }),
    signal: AbortSignal.timeout(10000),
  })
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
// setup() — verify credentials
// ---------------------------------------------------------------------------

export async function setup(): Promise<ToolResult<{ bot_username: string; couchdb: string }>> {
  const start = Date.now()
  const token = process.env.TELEGRAM_BOT_TOKEN
  const resendKey = process.env.RESEND_API_KEY
  const resendFrom = process.env.RESEND_FROM
  const resendTo = process.env.RESEND_TO
  const couchdbUrl = process.env.COUCHDB_URL

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

  let couchdbStatus = 'not_configured'
  if (couchdbUrl) {
    log.info(`Probing CouchDB at ${couchdbUrl}...`)
    const reachable = await couchdbPing(couchdbUrl)
    couchdbStatus = reachable ? 'connected' : 'unavailable'
    log.info(`CouchDB: ${couchdbStatus}`)
  }

  return {
    ok: true,
    data: { bot_username: botUsername, couchdb: couchdbStatus },
    duration_ms: Date.now() - start,
  }
}

// ---------------------------------------------------------------------------
// run() — fetch, persist, digest, send
// ---------------------------------------------------------------------------

export async function run(input: Input): Promise<ToolResult<DigestOutput>> {
  const start = Date.now()
  const token = process.env.TELEGRAM_BOT_TOKEN
  const resendKey = process.env.RESEND_API_KEY
  const resendFrom = process.env.RESEND_FROM
  const resendTo = process.env.RESEND_TO
  const couchdbUrl = process.env.COUCHDB_URL

  if (!token) return { ok: false, error: 'TELEGRAM_BOT_TOKEN is not set', duration_ms: Date.now() - start }
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

  // 1. Fetch updates from Telegram
  log.info('Fetching updates from Telegram...')
  let updates: TelegramUpdate[]
  try {
    updates = await telegramGetUpdates(token)
  } catch (e) {
    return { ok: false, error: `Telegram getUpdates failed: ${e}`, duration_ms: Date.now() - start }
  }
  log.info(`Received ${updates.length} updates from Telegram`)

  const freshMessages = buildMessages(updates)

  // 2. CouchDB persistence + window query (optional, graceful degradation)
  let windowMessages: TelegramMessage[]
  const couchdbDb = 'telegram-messages'

  if (couchdbUrl) {
    const reachable = await couchdbPing(couchdbUrl)
    if (reachable) {
      log.info('CouchDB reachable — persisting and querying...')
      try {
        await couchdbEnsureDb(couchdbUrl, couchdbDb)
        await couchdbEnsureIndex(couchdbUrl, couchdbDb)
        await couchdbUpsertMessages(couchdbUrl, couchdbDb, freshMessages)
        windowMessages = await couchdbQueryWindow(couchdbUrl, couchdbDb, sinceTimestamp)
        log.info(`CouchDB: ${windowMessages.length} messages in window`)
      } catch (e) {
        log.warn(`CouchDB operation failed (falling back to current poll): ${e}`)
        windowMessages = freshMessages.filter(m => m.date >= sinceTimestamp)
      }
    } else {
      log.warn('CouchDB unavailable — using current poll only')
      windowMessages = freshMessages.filter(m => m.date >= sinceTimestamp)
    }
  } else {
    // No CouchDB — use current poll filtered to window
    windowMessages = freshMessages.filter(m => m.date >= sinceTimestamp)
  }

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

  const recipients = resendTo.split(',').map(s => s.trim()).filter(Boolean)

  // 5. Send or dry-run
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

  // 5. Advance Telegram offset — only after successful send to prevent message loss
  if (updates.length > 0) {
    const maxUpdateId = Math.max(...updates.map(u => u.update_id))
    try {
      await telegramAdvanceOffset(token, maxUpdateId)
      log.info(`Advanced Telegram offset to ${maxUpdateId + 1}`)
    } catch (e) {
      log.warn(`Failed to advance Telegram offset: ${e}`)
    }
  }

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
