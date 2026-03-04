/**
 * citadel-annotate-log — Annotate a Citadel Log Entry
 * =====================================================
 * Enables Pantheon agents to annotate Citadel log entries with typed notes
 * (root_cause, bug, note, false_positive, incident) directly from Discord conversations.
 *
 * WHAT IT DOES
 * ------------
 * --meta:   Writes ToolMeta JSON to stdout.
 * --run:    Reads JSON from stdin: { log_id, content, annotation_type }
 *           Validates input, POSTs to Citadel /api/v1/annotations with author_type: "ai_agent",
 *           returns { annotation_id, created_at, log_id, annotation_type }.
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

const log = makeLogger('citadel-annotate-log')

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const VALID_ANNOTATION_TYPES = ['note', 'bug', 'root_cause', 'false_positive', 'incident'] as const
type AnnotationType = typeof VALID_ANNOTATION_TYPES[number]

const DEFAULT_BASE_URL = 'https://citadel.example.com'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface AnnotateLogInput {
  /** Citadel log entry ID to annotate */
  log_id: string
  /** Annotation body — markdown supported */
  content: string
  /** Semantic type of the annotation */
  annotation_type: AnnotationType
}

interface AnnotateLogOutput {
  /** Citadel-assigned annotation ID */
  annotation_id: string
  /** ISO 8601 timestamp when the annotation was created */
  created_at: string
  /** The log_id that was annotated (echoed for confirmation) */
  log_id: string
  /** The annotation_type that was submitted */
  annotation_type: string
}

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'citadel-annotate-log',
  display_name: 'Citadel Annotate Log',
  description: 'Annotate a Citadel log entry with a typed note — root cause, bug, incident link, or false positive',
  version: '1.0.0',
  requires_setup: false,
  input_schema: {
    type: 'object',
    required: ['log_id', 'content', 'annotation_type'],
    properties: {
      log_id: {
        type: 'string',
        description: 'Citadel log entry ID to annotate',
      },
      content: {
        type: 'string',
        description: 'Annotation body — markdown supported',
      },
      annotation_type: {
        type: 'string',
        enum: [...VALID_ANNOTATION_TYPES],
        description: 'Semantic type: note | bug | root_cause | false_positive | incident',
      },
    },
  },
  output_schema: {
    type: 'object',
    required: ['annotation_id', 'created_at', 'log_id', 'annotation_type'],
    properties: {
      annotation_id: {
        type: 'string',
        description: 'Citadel-assigned annotation ID',
      },
      created_at: {
        type: 'string',
        description: 'ISO 8601 timestamp when the annotation was created',
      },
      log_id: {
        type: 'string',
        description: 'The log_id that was annotated',
      },
      annotation_type: {
        type: 'string',
        description: 'The annotation_type that was submitted',
      },
    },
  },
}

// ---------------------------------------------------------------------------
// HTTP client
// ---------------------------------------------------------------------------

async function postAnnotation(
  baseUrl: string,
  apiKey: string,
  body: {
    log_id: string
    content: string
    annotation_type: string
    author_type: 'ai_agent'
  }
): Promise<{ status: number; json: unknown }> {
  const url = `${baseUrl}/api/v1/annotations`
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${apiKey}`,
    },
    body: JSON.stringify(body),
  })
  const json = await response.json().catch(() => null)
  return { status: response.status, json }
}

// ---------------------------------------------------------------------------
// Core run logic
// ---------------------------------------------------------------------------

async function run(input: Partial<AnnotateLogInput>): Promise<ToolResult<AnnotateLogOutput>> {
  const startMs = Date.now()

  // 1. Validate API key
  const apiKey = getEnv('CITADEL_API_KEY')
  if (!apiKey) {
    return {
      ok: false,
      error: 'CITADEL_API_KEY is not set — configure it in ToolCredential store',
    }
  }

  // 2. Validate required fields
  const { log_id, content, annotation_type } = input
  if (!log_id || !content) {
    return { ok: false, error: 'log_id and content are required' }
  }

  // 3. Validate annotation_type
  if (!annotation_type || !(VALID_ANNOTATION_TYPES as readonly string[]).includes(annotation_type)) {
    return {
      ok: false,
      error: `annotation_type must be one of: ${VALID_ANNOTATION_TYPES.join(', ')}`,
    }
  }

  const baseUrl = getEnv('CITADEL_BASE_URL') ?? DEFAULT_BASE_URL

  log.info(`posting annotation { log_id: ${log_id}, annotation_type: ${annotation_type} }`)

  // 4. POST to Citadel
  let response: { status: number; json: unknown }
  try {
    response = await postAnnotation(baseUrl, apiKey, {
      log_id,
      content,
      annotation_type,
      author_type: 'ai_agent',
    })
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err)
    log.error(`network error: ${message}`)
    return { ok: false, error: `Network error: ${message}` }
  }

  const { status, json } = response

  // 5. Handle response
  if (status === 201) {
    const data = json as Record<string, unknown>
    const annotation_id = String(data.annotation_id ?? data.id ?? '')
    const created_at = String(data.created_at ?? new Date().toISOString())
    log.info(`annotation created { annotation_id: ${annotation_id} }`)
    return {
      ok: true,
      data: {
        annotation_id,
        created_at,
        log_id,
        annotation_type,
      },
      duration_ms: Date.now() - startMs,
    }
  }

  // Non-2xx error
  let errorDetail: string
  if (status === 401) {
    errorDetail = 'invalid or missing API key'
  } else if (status === 404) {
    errorDetail = 'log_id not found'
  } else if (status === 429) {
    errorDetail = 'rate limited'
  } else {
    errorDetail = typeof json === 'string' ? json : JSON.stringify(json)
  }

  log.error(`api error ${status}: ${errorDetail}`)
  return {
    ok: false,
    error: `Citadel API error ${status}: ${errorDetail}`,
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
    .then(raw => run(JSON.parse(raw || '{}') as Partial<AnnotateLogInput>))
    .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
    .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta, --setup, or --run.`)
  exit(1)
}
