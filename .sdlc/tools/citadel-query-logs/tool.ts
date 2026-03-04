/**
 * citadel-query-logs — Query Citadel Logs via CPL
 * =================================================
 * Enables Pantheon agents to query Citadel log storage using CPL (Citadel
 * Processing Language) directly from Discord incident workflows.
 *
 * WHAT IT DOES
 * ------------
 * --meta:   Writes ToolMeta JSON to stdout (includes embedded CPL skill instructions).
 * --run:    Reads JSON from stdin: { query: string, time_range: string, limit?: number }
 *           Validates input, calls Citadel GET /api/v1/query, returns structured
 *           log entries and episode context.
 * --setup:  No-op — returns { ok: true } immediately (no setup required).
 *
 * CONFIGURATION
 * -------------
 * CITADEL_API_KEY  (required) — Bearer token for Citadel API
 * CITADEL_BASE_URL (optional) — Base URL, defaults to https://citadel.example.com
 *
 * WHAT IT WRITES
 * --------------
 * Nothing — stateless. No local files are created or modified on --run.
 * STDERR: structured log lines via _shared/log.ts
 * STDOUT: JSON only (ToolResult shape from _shared/types.ts)
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, getEnv, exit } from '../_shared/runtime.ts'

const log = makeLogger('citadel-query-logs')

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEFAULT_BASE_URL = 'https://citadel.example.com'
const DEFAULT_LIMIT = 50
const MAX_LIMIT = 500
const TIMEOUT_MS = 10_000

// ---------------------------------------------------------------------------
// Embedded CPL skill instructions
// ---------------------------------------------------------------------------

const CPL_SKILL_INSTRUCTIONS = `
citadel_query_logs — Search Citadel logs using CPL (Citadel Processing Language).

CPL QUICK REFERENCE
  Field filters:
    level:error         filter by log level (debug, info, warn, error, fatal)
    service:auth        filter by service name
    host:api-01         filter by host
    trace_id:abc123     correlate across services by trace ID

  Time ranges (passed as time_range param):
    1h    last 1 hour
    30m   last 30 minutes
    24h   last 24 hours
    7d    last 7 days

  Boolean operators:
    level:error AND service:auth                  — both conditions
    level:error OR level:fatal                    — either condition
    level:error NOT service:debug-svc             — exclude a condition
    (level:error OR level:fatal) AND service:auth — grouped expression

  Trace ID correlation:
    Use trace_id:<id> to fetch all log lines across all services that share
    the same trace, showing the full request journey end-to-end.

COMMON INCIDENT PATTERNS
  All errors in auth last hour:       { query: "level:error service:auth", time_range: "1h" }
  All fatals across all services:     { query: "level:fatal", time_range: "30m" }
  Correlate a specific trace:         { query: "trace_id:abc-def-123", time_range: "2h" }
  Cross-service error investigation:  { query: "(level:error OR level:fatal) AND service:payment OR service:gateway", time_range: "1h" }

PARAMETERS
  query       (required) CPL expression string
  time_range  (required) Relative time range: 1h, 30m, 24h, 7d
  limit       (optional) Max entries returned, default ${DEFAULT_LIMIT}, max ${MAX_LIMIT}
`.trim()

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface QueryLogsInput {
  /** CPL query expression, e.g. "level:error service:auth" */
  query: string
  /** Relative time range, e.g. "1h", "30m", "24h" */
  time_range: string
  /** Maximum log entries to return (default: 50, max: 500) */
  limit?: number
}

interface LogEntry {
  timestamp: string
  level: string
  service: string
  message: string
  trace_id: string
  metadata: Record<string, unknown>
}

interface EpisodeContext {
  id: string
  title: string
  severity: string
  started_at: string
}

interface QueryLogsOutput {
  entries: LogEntry[]
  episode_context: EpisodeContext[]
  total_matched: number
  query_duration_ms: number
}

/** Citadel raw API response shape */
interface CitadelQueryResponse {
  entries?: Array<Record<string, unknown>>
  episodes?: Array<Record<string, unknown>>
  total?: number
  duration_ms?: number
  error?: string
}

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'citadel-query-logs',
  display_name: 'Citadel Query Logs',
  description: 'Query Citadel log storage using CPL expressions to investigate incidents from Discord',
  version: '1.0.0',
  requires_setup: false,
  input_schema: {
    type: 'object',
    required: ['query', 'time_range'],
    properties: {
      query: {
        type: 'string',
        description: `CPL query expression. ${CPL_SKILL_INSTRUCTIONS}`,
      },
      time_range: {
        type: 'string',
        description: 'Relative time range: 1h, 30m, 24h, 7d',
      },
      limit: {
        type: 'integer',
        description: `Max log entries to return. Default ${DEFAULT_LIMIT}, max ${MAX_LIMIT}.`,
        default: DEFAULT_LIMIT,
        minimum: 1,
        maximum: MAX_LIMIT,
      },
    },
  },
  output_schema: {
    type: 'object',
    required: ['entries', 'episode_context', 'total_matched', 'query_duration_ms'],
    properties: {
      entries: {
        type: 'array',
        description: 'Matching log entries',
        items: {
          type: 'object',
          properties: {
            timestamp: { type: 'string' },
            level: { type: 'string' },
            service: { type: 'string' },
            message: { type: 'string' },
            trace_id: { type: 'string' },
            metadata: { type: 'object' },
          },
        },
      },
      episode_context: {
        type: 'array',
        description: 'Citadel episodes associated with the queried time range and services',
        items: {
          type: 'object',
          properties: {
            id: { type: 'string' },
            title: { type: 'string' },
            severity: { type: 'string' },
            started_at: { type: 'string' },
          },
        },
      },
      total_matched: {
        type: 'integer',
        description: 'Total count of matching entries (may exceed limit)',
      },
      query_duration_ms: {
        type: 'integer',
        description: 'How long the Citadel query took in milliseconds',
      },
    },
  },
}

// ---------------------------------------------------------------------------
// HTTP client
// ---------------------------------------------------------------------------

async function queryCitadel(
  baseUrl: string,
  apiKey: string,
  query: string,
  timeRange: string,
  limit: number,
  attempt: number = 1,
): Promise<{ status: number; json: CitadelQueryResponse | null }> {
  const params = new URLSearchParams({
    q: query,
    range: timeRange,
    limit: String(limit),
  })
  const url = `${baseUrl}/api/v1/query?${params.toString()}`

  const controller = new AbortController()
  const timeout = setTimeout(() => controller.abort(), TIMEOUT_MS)

  try {
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Accept': 'application/json',
      },
      signal: controller.signal,
    })
    clearTimeout(timeout)
    const json = await response.json().catch(() => null) as CitadelQueryResponse | null
    return { status: response.status, json }
  } catch (err) {
    clearTimeout(timeout)
    const isAbort = err instanceof Error && err.name === 'AbortError'

    // Retry once on timeout or network error
    if (attempt === 1) {
      log.info(`attempt ${attempt} failed (${isAbort ? 'timeout' : 'network error'}), retrying...`)
      return queryCitadel(baseUrl, apiKey, query, timeRange, limit, 2)
    }

    throw err
  }
}

// ---------------------------------------------------------------------------
// Response mapping
// ---------------------------------------------------------------------------

function mapEntry(raw: Record<string, unknown>): LogEntry {
  return {
    timestamp: String(raw.timestamp ?? raw.ts ?? ''),
    level: String(raw.level ?? raw.severity ?? ''),
    service: String(raw.service ?? raw.svc ?? ''),
    message: String(raw.message ?? raw.msg ?? ''),
    trace_id: String(raw.trace_id ?? raw.traceId ?? ''),
    metadata: (raw.metadata ?? raw.meta ?? {}) as Record<string, unknown>,
  }
}

function mapEpisode(raw: Record<string, unknown>): EpisodeContext {
  return {
    id: String(raw.id ?? raw.episode_id ?? ''),
    title: String(raw.title ?? raw.name ?? ''),
    severity: String(raw.severity ?? raw.level ?? ''),
    started_at: String(raw.started_at ?? raw.startedAt ?? raw.created_at ?? ''),
  }
}

// ---------------------------------------------------------------------------
// Core run logic
// ---------------------------------------------------------------------------

async function run(input: Partial<QueryLogsInput>): Promise<ToolResult<QueryLogsOutput>> {
  const startMs = Date.now()

  // 1. Validate API key first (most common misconfiguration)
  const apiKey = getEnv('CITADEL_API_KEY')
  if (!apiKey) {
    return {
      ok: false,
      error: 'CITADEL_API_KEY is not set — configure it in ToolCredential store',
    }
  }

  // 2. Validate required fields
  const { query, time_range } = input
  if (!query || query.trim() === '') {
    return { ok: false, error: 'query is required — provide a CPL expression' }
  }
  if (!time_range || time_range.trim() === '') {
    return { ok: false, error: 'time_range is required — e.g. "1h", "30m", "24h"' }
  }

  // 3. Apply limit with cap
  const rawLimit = typeof input.limit === 'number' ? input.limit : DEFAULT_LIMIT
  const limit = Math.max(1, Math.min(rawLimit, MAX_LIMIT))

  const baseUrl = (getEnv('CITADEL_BASE_URL') ?? DEFAULT_BASE_URL).replace(/\/$/, '')

  log.info(`querying { query: "${query}", time_range: ${time_range}, limit: ${limit} }`)

  // 4. Call Citadel
  let response: { status: number; json: CitadelQueryResponse | null }
  try {
    response = await queryCitadel(baseUrl, apiKey, query, time_range, limit)
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err)
    const isTimeout = err instanceof Error && err.name === 'AbortError'
    log.error(`network error: ${message}`)
    return {
      ok: false,
      error: isTimeout
        ? 'Citadel query timed out after retrying. Check network connectivity.'
        : `Network error: ${message}`,
      duration_ms: Date.now() - startMs,
    }
  }

  const { status, json } = response

  // 5. Handle 429 rate limit — surface with guidance
  if (status === 429) {
    const retryAfter = (json as Record<string, unknown>)?.retry_after
    const msg = retryAfter
      ? `Rate limited by Citadel. Retry after ${retryAfter}s.`
      : 'Rate limited by Citadel. Try again shortly.'
    log.error(`rate limited: ${msg}`)
    return { ok: false, error: msg, duration_ms: Date.now() - startMs }
  }

  // 6. Handle CPL parse error (400)
  if (status === 400) {
    const errorMsg = json?.error ?? 'CPL query parse error (no details from Citadel)'
    log.error(`cpl error: ${errorMsg}`)
    return {
      ok: true,
      data: {
        entries: [],
        episode_context: [],
        total_matched: 0,
        query_duration_ms: json?.duration_ms ?? (Date.now() - startMs),
        // @ts-ignore — query_error is an extended field for CPL errors
        query_error: errorMsg,
      },
    }
  }

  // 7. Handle auth failure (401)
  if (status === 401) {
    log.error('authentication failed')
    return {
      ok: false,
      error: 'Authentication failed. Check that CITADEL_API_KEY is correctly set in ToolCredential.',
      // @ts-ignore — error_code is an extended field
      error_code: 'auth_failed',
      duration_ms: Date.now() - startMs,
    }
  }

  // 8. Handle other non-200 responses
  if (status < 200 || status >= 300) {
    const errorDetail = json?.error ?? JSON.stringify(json)
    log.error(`api error ${status}: ${errorDetail}`)
    return {
      ok: false,
      error: `Citadel API error ${status}: ${errorDetail}`,
      duration_ms: Date.now() - startMs,
    }
  }

  // 9. Map successful response
  const rawEntries = (json?.entries ?? []) as Array<Record<string, unknown>>
  const rawEpisodes = (json?.episodes ?? []) as Array<Record<string, unknown>>

  const entries = rawEntries.map(mapEntry)
  const episodeContext = rawEpisodes.map(mapEpisode)
  const totalMatched = json?.total ?? entries.length
  const queryDurationMs = json?.duration_ms ?? (Date.now() - startMs)

  log.info(`query returned ${entries.length} entries, total_matched=${totalMatched}`)

  return {
    ok: true,
    data: {
      entries,
      episode_context: episodeContext,
      total_matched: totalMatched,
      query_duration_ms: queryDurationMs,
    },
    duration_ms: Date.now() - startMs,
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
  console.log(JSON.stringify({ ok: true }))
  exit(0)
} else if (mode === '--run') {
  readStdin()
    .then(raw => run(JSON.parse(raw || '{}') as Partial<QueryLogsInput>))
    .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
    .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta, --setup, or --run.`)
  exit(1)
}
