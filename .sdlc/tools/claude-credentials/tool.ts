/**
 * Claude Credentials
 * ==================
 * Manages the PostgreSQL-backed Claude OAuth credential pool.
 *
 * WHAT IT DOES
 * ------------
 * --run:   Reads JSON from stdin:
 *          { "action": "list" }
 *            → Lists all credentials in the pool (account names, status, usage).
 *              Token values are never returned.
 *          { "action": "add", "account_name": "you@example.com", "token": "sk-ant-..." }
 *            → Adds a new Claude OAuth token to the pool.
 *          { "action": "delete", "id": 5 }
 *            → Permanently removes a credential by id.
 *          { "action": "toggle", "id": 5, "is_active": false }
 *            → Enables or disables a credential without deleting it.
 *
 * --meta:  Writes ToolMeta JSON to stdout.
 *
 * SECURITY
 * --------
 * persist_interactions is false — inputs (which include tokens) are never
 * written to disk in the interaction log. The token field is accepted by
 * the server and stored directly in Postgres; it is never returned over the wire.
 *
 * REQUIREMENTS
 * ------------
 * Requires SDLC_SERVER_URL and SDLC_AGENT_TOKEN to be set. Both are injected
 * automatically by the server for every tool subprocess.
 * Requires DATABASE_URL to be configured on the server for the pool to be active.
 *
 * HOW TO GET A CLAUDE TOKEN
 * -------------------------
 * Run `claude setup-token` in a shell where you are logged in to Claude.
 * Copy the printed token and paste it into the "add" action's token field.
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'

const log = makeLogger('claude-credentials')

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'claude-credentials',
  display_name: 'Claude Credentials',
  description: 'Manages the Claude OAuth credential pool — list, add, remove, and toggle tokens',
  version: '1.0.0',
  requires_setup: false,
  // IMPORTANT: tokens must never be stored in the interaction log
  persist_interactions: false,
  input_schema: {
    type: 'object',
    required: ['action'],
    properties: {
      action: {
        type: 'string',
        enum: ['list', 'add', 'delete', 'toggle'],
        description: 'Operation to perform',
      },
      account_name: {
        type: 'string',
        description: '(add) Email or label for the Claude account',
      },
      token: {
        type: 'string',
        description: '(add) Claude OAuth token from `claude setup-token`',
      },
      id: {
        type: 'number',
        description: '(delete / toggle) Credential id from the list',
      },
      is_active: {
        type: 'boolean',
        description: '(toggle) New active state for the credential',
      },
    },
  },
  output_schema: {
    type: 'object',
    description: 'Action-dependent result — see action descriptions above',
  },
  form_layout: [
    {
      field: 'action',
      label: 'Action',
      type: 'select',
      options: ['list', 'add', 'delete', 'toggle'],
      default: 'list',
    },
    {
      field: 'account_name',
      label: 'Account name / email',
      type: 'text',
      placeholder: 'you@example.com',
      show_when: { action: 'add' },
    },
    {
      field: 'token',
      label: 'OAuth token (from `claude setup-token`)',
      type: 'password',
      placeholder: 'Paste token here',
      show_when: { action: 'add' },
    },
    {
      field: 'id',
      label: 'Credential ID',
      type: 'number',
      show_when: { action: ['delete', 'toggle'] },
    },
    {
      field: 'is_active',
      label: 'Active',
      type: 'boolean',
      show_when: { action: 'toggle' },
    },
  ],
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface CredentialRow {
  id: number
  account_name: string
  is_active: boolean
  last_used_at: string
  use_count: number
}

interface PoolStatus {
  total: number
  active: number
}

interface StatusResponse {
  connected: boolean
  status?: PoolStatus
  message?: string
}

// ---------------------------------------------------------------------------
// API helpers
// ---------------------------------------------------------------------------

function getServerConfig(): { url: string; token: string } {
  const url = process.env['SDLC_SERVER_URL']
  const token = process.env['SDLC_AGENT_TOKEN']
  if (!url || !token) {
    throw new Error(
      'SDLC_SERVER_URL or SDLC_AGENT_TOKEN is not set. ' +
        'This tool must be run via the sdlc-server (web UI or `sdlc tool run`).',
    )
  }
  return { url, token }
}

async function apiGet<T>(path: string): Promise<T> {
  const { url, token } = getServerConfig()
  const resp = await fetch(`${url}${path}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
  if (!resp.ok) {
    const text = await resp.text()
    throw new Error(`GET ${path} failed (${resp.status}): ${text}`)
  }
  return resp.json() as Promise<T>
}

async function apiPost<T>(path: string, body: unknown): Promise<T> {
  const { url, token } = getServerConfig()
  const resp = await fetch(`${url}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
    body: JSON.stringify(body),
  })
  if (!resp.ok) {
    const text = await resp.text()
    throw new Error(`POST ${path} failed (${resp.status}): ${text}`)
  }
  return resp.json() as Promise<T>
}

async function apiPatch<T>(path: string, body: unknown): Promise<T> {
  const { url, token } = getServerConfig()
  const resp = await fetch(`${url}${path}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
    body: JSON.stringify(body),
  })
  if (!resp.ok) {
    const text = await resp.text()
    throw new Error(`PATCH ${path} failed (${resp.status}): ${text}`)
  }
  return resp.json() as Promise<T>
}

async function apiDelete(path: string): Promise<void> {
  const { url, token } = getServerConfig()
  const resp = await fetch(`${url}${path}`, {
    method: 'DELETE',
    headers: { Authorization: `Bearer ${token}` },
  })
  if (!resp.ok && resp.status !== 204) {
    const text = await resp.text()
    throw new Error(`DELETE ${path} failed (${resp.status}): ${text}`)
  }
}

// ---------------------------------------------------------------------------
// Action handlers
// ---------------------------------------------------------------------------

async function actionList(): Promise<ToolResult> {
  const poolStatus = await apiGet<StatusResponse>('/api/credential-pool')
  if (!poolStatus.connected) {
    return {
      ok: false,
      error: `Pool not connected: ${poolStatus.message ?? 'unknown'}`,
    }
  }
  const credentials = await apiGet<CredentialRow[]>('/api/credential-pool/credentials')
  return {
    ok: true,
    data: {
      pool: poolStatus.status,
      credentials: credentials.map(c => ({
        id: c.id,
        account_name: c.account_name,
        is_active: c.is_active,
        use_count: c.use_count,
        last_used_at: c.last_used_at,
      })),
    },
  }
}

async function actionAdd(account_name: string, token: string): Promise<ToolResult> {
  if (!account_name?.trim()) {
    return { ok: false, error: 'account_name is required' }
  }
  if (!token?.trim()) {
    return { ok: false, error: 'token is required' }
  }
  const result = await apiPost<{ id: number; account_name: string }>(
    '/api/credential-pool/credentials',
    { account_name: account_name.trim(), token: token.trim() },
  )
  log.info(`Added credential id=${result.id} for ${result.account_name}`)
  return {
    ok: true,
    data: { id: result.id, account_name: result.account_name, message: 'Credential added to pool' },
  }
}

async function actionDelete(id: number): Promise<ToolResult> {
  if (!id) {
    return { ok: false, error: 'id is required' }
  }
  await apiDelete(`/api/credential-pool/credentials/${id}`)
  log.info(`Deleted credential id=${id}`)
  return { ok: true, data: { id, message: 'Credential deleted' } }
}

async function actionToggle(id: number, is_active: boolean): Promise<ToolResult> {
  if (!id) {
    return { ok: false, error: 'id is required' }
  }
  if (is_active === undefined || is_active === null) {
    return { ok: false, error: 'is_active is required' }
  }
  const result = await apiPatch<{ id: number; is_active: boolean }>(
    `/api/credential-pool/credentials/${id}`,
    { is_active },
  )
  log.info(`Toggled credential id=${result.id} is_active=${result.is_active}`)
  return {
    ok: true,
    data: { id: result.id, is_active: result.is_active, message: `Credential ${is_active ? 'enabled' : 'disabled'}` },
  }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

const args = getArgs()

if (args.includes('--meta')) {
  process.stdout.write(JSON.stringify(meta) + '\n')
  exit(0)
} else if (args.includes('--run')) {
  const t0 = Date.now()
  readStdin()
    .then(async (raw: string) => {
      let input: {
        action: string
        account_name?: string
        token?: string
        id?: number
        is_active?: boolean
      }
      try {
        input = JSON.parse(raw)
      } catch {
        const result: ToolResult = { ok: false, error: 'invalid JSON input' }
        process.stdout.write(JSON.stringify(result) + '\n')
        exit(1)
        return
      }

      let result: ToolResult
      try {
        switch (input.action) {
          case 'list':
            result = await actionList()
            break
          case 'add':
            result = await actionAdd(input.account_name ?? '', input.token ?? '')
            break
          case 'delete':
            result = await actionDelete(input.id ?? 0)
            break
          case 'toggle':
            result = await actionToggle(input.id ?? 0, input.is_active ?? false)
            break
          default:
            result = { ok: false, error: `unknown action: ${input.action}` }
        }
      } catch (e) {
        result = { ok: false, error: String(e) }
      }

      result.duration_ms = Date.now() - t0
      process.stdout.write(JSON.stringify(result) + '\n')
      exit(result.ok ? 0 : 1)
    })
    .catch((e: unknown) => {
      process.stdout.write(JSON.stringify({ ok: false, error: String(e) }) + '\n')
      exit(1)
    })
} else {
  process.stderr.write('Usage: tool.ts --meta | --run\n')
  exit(1)
}
