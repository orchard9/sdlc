/**
 * SDLC Agent Recruiter and Invoker
 *
 * Provides primitives for agentic tools:
 *
 * 1. ensureAgent(root, slug, roleDescription) — ensures a named agent exists
 *    in .claude/agents/<slug>.md, creating it if missing. Returns the
 *    absolute path to the agent file.
 *
 * 2. runAgent({ prompt, agentFile?, maxTurns? }) — invoke a Claude agent run
 *    via the local sdlc-server and await the result. The run appears in the
 *    activity feed. Requires SDLC_SERVER_URL and SDLC_AGENT_TOKEN env vars
 *    (injected automatically for every tool subprocess).
 *    IMPORTANT: Only safe from tools with `streaming: true` in their meta.
 *
 * 3. runAgentCli(agentPath, prompt, opts?) — synchronous variant that invokes
 *    the claude CLI directly. Use for non-streaming tools or when the server
 *    is not available.
 *
 * Recruit-if-missing pattern:
 *   When a tool needs an expert sub-agent, use ensureAgent() to create the
 *   agent file on first use, then runAgent() to invoke it. The agent file is
 *   shared across all tools that recruit the same slug — subsequent calls
 *   to ensureAgent() return the existing path immediately.
 *
 *   Example:
 *     const agentPath = ensureAgent(root, 'code-reviewer', 'Expert code reviewer ...')
 *     const feedback = await runAgent({ prompt, agentFile: agentPath, maxTurns: 10 })
 *
 * Agent file format (markdown with YAML frontmatter):
 *   ---
 *   name: Agent Name
 *   role: Role Title
 *   ---
 *   # Agent Name — Role Title
 *   <persona description>
 *
 * Agent files live in .claude/agents/<slug>.md and are shared across the
 * project — any tool or session that recruits an agent creates the same file.
 *
 * Error contract:
 * - ensureAgent: throws if file creation fails
 * - runAgent: throws if server is unreachable, token is invalid, or run fails
 * - runAgentCli: throws if claude CLI invocation fails or times out
 */

import { spawnSync } from 'node:child_process'
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs'
import { join } from 'node:path'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface RunAgentOptions {
  /** Maximum time to wait for the agent response in milliseconds. Default: 90000 (90s) */
  timeout_ms?: number
  /** Working directory for the claude CLI. Default: project root. */
  cwd?: string
}

// ---------------------------------------------------------------------------
// ensureAgent
// ---------------------------------------------------------------------------

/**
 * Ensure a named agent exists in .claude/agents/<slug>.md.
 *
 * If the file already exists, returns its path immediately.
 * If not, creates a minimal agent file from the roleDescription and returns the path.
 *
 * The agent file uses the standard YAML frontmatter + markdown format that
 * claude --agent reads from .claude/agents/.
 *
 * @param root - Project root directory
 * @param slug - Agent slug (used as filename: <slug>.md)
 * @param roleDescription - One or two sentence description of the agent's role and expertise
 * @returns Absolute path to the agent file
 */
export function ensureAgent(root: string, slug: string, roleDescription: string): string {
  const agentDir = join(root, '.claude', 'agents')
  const agentPath = join(agentDir, `${slug}.md`)

  if (existsSync(agentPath)) {
    return agentPath
  }

  // Create the agents directory if needed
  mkdirSync(agentDir, { recursive: true })

  // Derive a display name from the slug (kebab-case → Title Case)
  const displayName = slug
    .split('-')
    .map(w => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')

  // Extract a short role title from the description (first sentence, max 60 chars)
  const roleTitle = roleDescription.split('.')[0]?.slice(0, 60) ?? roleDescription.slice(0, 60)

  const content = `---
name: ${displayName}
role: ${roleTitle}
---

# ${displayName}

${roleDescription}

## How you communicate

Be direct and specific. Ground your observations in the actual state of the project.
When identifying concerns, describe the specific problem and its potential impact.
When asked for a structured JSON response, respond with valid JSON only — no markdown
fences, no explanation, just the JSON object.
`

  writeFileSync(agentPath, content, 'utf8')
  return agentPath
}

// ---------------------------------------------------------------------------
// runAgentCli (synchronous, direct claude CLI invocation)
// ---------------------------------------------------------------------------

/**
 * Invoke a claude agent synchronously via the claude CLI.
 *
 * Reads the agent file at agentPath and uses its content as the system prompt
 * via `claude --print --system-prompt <content>`. The user prompt is passed
 * as stdin.
 *
 * Use this for non-streaming tools where the server is not available, or
 * when you hold a local agent file. For streaming tools with server access,
 * prefer `runAgent()` instead.
 *
 * @param agentPath - Absolute path to the agent .md file
 * @param prompt - User prompt to send to the agent
 * @param opts - Optional configuration (timeout, cwd)
 * @returns Raw text response from the agent
 */
export function runAgentCli(
  agentPath: string,
  prompt: string,
  opts: RunAgentOptions = {},
): string {
  const timeout = opts.timeout_ms ?? 90_000
  const cwd = opts.cwd

  // Read the agent file
  let agentContent: string
  try {
    agentContent = readFileSync(agentPath, 'utf8')
  } catch (e) {
    throw new Error(`runAgentCli: could not read agent file at ${agentPath}: ${e}`)
  }

  // Extract the body after frontmatter (strip --- ... --- block)
  const systemPrompt = agentContent
    .replace(/^---[\s\S]*?---\n?/, '')
    .trim()

  // Invoke claude --print with system prompt. User prompt is passed via stdin.
  const result = spawnSync(
    'claude',
    [
      '--print',
      '--system-prompt', systemPrompt,
    ],
    {
      input: prompt,
      encoding: 'utf8',
      timeout,
      cwd,
      env: process.env,
      stdio: ['pipe', 'pipe', 'pipe'],
    }
  )

  if (result.error) {
    throw new Error(`runAgentCli: claude CLI error: ${result.error.message}`)
  }
  if (result.status !== 0) {
    const stderr = result.stderr?.toString() ?? ''
    throw new Error(
      `runAgentCli: claude CLI exited with status ${result.status ?? 'unknown'}${stderr ? `: ${stderr.slice(0, 500)}` : ''}`
    )
  }

  return (result.stdout?.toString() ?? '').trim()
}

// ---------------------------------------------------------------------------
// runAgentDispatch
// ---------------------------------------------------------------------------

export interface AgentDispatchResult {
  run_id: string
  run_key: string
  /** 'started' = run was accepted and is now running in the background.
   *  'conflict' = a run with the same run_key is already in flight. */
  status: 'started' | 'conflict'
}

/**
 * Dispatch a Claude agent run via the local sdlc-server and return immediately.
 *
 * Unlike `runAgentViaServer()` (which blocks until the agent completes), this
 * function calls POST /api/tools/agent-dispatch and returns as soon as the run
 * has been accepted — the agent continues running in the background.
 *
 * The run appears in the activity feed and emits SSE events just like any other
 * server-dispatched agent run.
 *
 * Requires SDLC_SERVER_URL and SDLC_AGENT_TOKEN to be set (injected automatically
 * by the server for every tool subprocess).
 *
 * @param prompt  - The text sent to the agent (slash command or free text)
 * @param runKey  - Deduplication key; if a run with this key is already in flight
 *                  the function returns `{ status: 'conflict' }` rather than throwing
 * @param label   - Human-readable label shown in the activity feed
 * @param opts    - Optional: maxTurns (default 40, server caps at 100)
 * @returns AgentDispatchResult with status 'started' or 'conflict'
 * @throws If the server is unreachable or returns an unexpected error
 */
export async function runAgentDispatch(
  prompt: string,
  runKey: string,
  label: string,
  opts: { maxTurns?: number } = {},
): Promise<AgentDispatchResult> {
  const serverUrl: string | undefined =
    typeof process !== 'undefined' ? process.env['SDLC_SERVER_URL'] : undefined
  const token: string | undefined =
    typeof process !== 'undefined' ? process.env['SDLC_AGENT_TOKEN'] : undefined

  if (!serverUrl || !token) {
    throw new Error(
      'runAgentDispatch: SDLC_SERVER_URL or SDLC_AGENT_TOKEN is not set. ' +
      'This function only works when the tool is invoked by sdlc-server ' +
      '(via `sdlc tool run` or the web UI). ' +
      'Both variables are injected automatically for every tool subprocess.',
    )
  }

  const response = await fetch(`${serverUrl}/api/tools/agent-dispatch`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`,
    },
    body: JSON.stringify({
      prompt,
      run_key: runKey,
      label,
      max_turns: opts.maxTurns ?? 40,
    }),
  })

  // 409 Conflict means the run is already in flight — return a conflict result
  // instead of throwing so the caller can handle it gracefully.
  if (response.status === 409) {
    return { run_id: '', run_key: runKey, status: 'conflict' }
  }

  let body: { run_id?: string; run_key?: string; status?: string; error?: string }
  try {
    body = await response.json() as typeof body
  } catch {
    throw new Error(
      `runAgentDispatch: server returned non-JSON response (status ${response.status})`,
    )
  }

  if (!response.ok || body.error) {
    throw new Error(
      `runAgentDispatch failed (${response.status}): ${body.error ?? 'unknown error'}`,
    )
  }

  return {
    run_id: body.run_id ?? '',
    run_key: body.run_key ?? runKey,
    status: 'started',
  }
}

// ---------------------------------------------------------------------------
// readAgentName
// ---------------------------------------------------------------------------

/**
 * Read the display name from an agent file's frontmatter.
 * Returns the slug as fallback if the file can't be read or has no name.
 */
export function readAgentName(agentPath: string, slug: string): string {
  try {
    const content = readFileSync(agentPath, 'utf8')
    const nameMatch = content.match(/^---[\s\S]*?name:\s*(.+?)[\r\n]/m)
    return nameMatch?.[1]?.trim() ?? slug
  } catch {
    return slug
  }
}

// ---------------------------------------------------------------------------
// runAgent (async, server-backed — the primary API for streaming tools)
// ---------------------------------------------------------------------------

export interface RunAgentOptions2 {
  /** The prompt to send to the agent. */
  prompt: string
  /**
   * Optional path (relative to project root) to an agent definition file (.md).
   * When provided, the file content is prepended to the prompt as system context.
   *
   * Recruit-if-missing pattern: use ensureAgent() to get or create the path,
   * then pass it here so the agent's persona is injected into the system prompt.
   *
   * If the file does not exist on the server, `runAgent` returns
   * `{ ok: false, error: 'agent file not found' }` instead of throwing.
   */
  agentFile?: string
  /** Maximum number of turns for the agent. Defaults to 20, capped at 100. */
  maxTurns?: number
}

/** Returned when runAgent() encounters a recoverable error (e.g. agent file not found). */
export interface RunAgentError {
  ok: false
  error: string
}

/**
 * Invoke a Claude agent run via the local sdlc-server and await the result.
 *
 * Calls POST /api/tools/agent-call on the local sdlc-server, which spawns a
 * real agent run (visible in the run history and activity feed). Blocks until
 * the run completes (up to 10 minutes) and returns the result text.
 *
 * IMPORTANT: Only safe to call from tools with `streaming: true` in their meta.
 * Calling it from a synchronous (blocking) tool will deadlock the server's thread pool.
 *
 * Requires SDLC_SERVER_URL and SDLC_AGENT_TOKEN to be set. Both are injected
 * automatically by the server for every tool subprocess — no setup needed.
 *
 * @returns The agent's final result text on success, or `{ ok: false, error }` if
 *          the agentFile was not found. Throws for all other errors (server unreachable,
 *          invalid token, agent run failed, etc.).
 *
 * @example
 * ```typescript
 * import { runAgent } from '../_shared/agent.ts'
 *
 * const result = await runAgent({
 *   prompt: 'Summarize the last 5 git commits in one paragraph.',
 *   maxTurns: 10,
 * })
 * // result is a string on success
 * if (typeof result === 'object' && !result.ok) {
 *   console.error('agent call failed:', result.error)
 * }
 * ```
 *
 * Recruit-if-missing pattern:
 * ```typescript
 * import { ensureAgent, runAgent } from '../_shared/agent.ts'
 *
 * // Ensure the reviewer agent file exists (creates it on first use)
 * const agentPath = ensureAgent(root, 'code-reviewer', 'Expert code reviewer ...')
 * // Run the agent with its persona loaded as context
 * const review = await runAgent({ prompt: diffText, agentFile: agentPath, maxTurns: 15 })
 * ```
 */
export async function runAgent(opts: RunAgentOptions2): Promise<string | RunAgentError> {
  // Read env vars set by the server for every tool subprocess
  const serverUrl: string | undefined =
    typeof process !== 'undefined' ? process.env['SDLC_SERVER_URL'] : undefined
  const token: string | undefined =
    typeof process !== 'undefined' ? process.env['SDLC_AGENT_TOKEN'] : undefined

  if (!serverUrl || !token) {
    throw new Error(
      'runAgent: SDLC_SERVER_URL or SDLC_AGENT_TOKEN is not set. ' +
      'This function only works when the tool is invoked by sdlc-server ' +
      '(via `sdlc tool run` or the web UI). ' +
      'Both variables are injected automatically for every tool subprocess.',
    )
  }

  const response = await fetch(`${serverUrl}/api/tools/agent-call`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`,
    },
    body: JSON.stringify({
      prompt: opts.prompt,
      agent_file: opts.agentFile,
      max_turns: opts.maxTurns ?? 20,
    }),
  })

  let body: { result?: string; error?: string; cost_usd?: number; turns?: number }
  try {
    body = await response.json() as typeof body
  } catch {
    throw new Error(
      `runAgent: server returned non-JSON response (status ${response.status})`,
    )
  }

  // Graceful handling for agent file not found (400 from the server).
  // Return a structured error object rather than throwing so the caller
  // can distinguish "agent file missing" from "server down" or "run failed".
  if (response.status === 400 && body.error && body.error.includes('agent_file')) {
    return { ok: false, error: 'agent file not found' }
  }

  if (!response.ok || body.error) {
    throw new Error(
      `runAgent failed (${response.status}): ${body.error ?? 'unknown error'}`,
    )
  }

  return body.result ?? ''
}

// ---------------------------------------------------------------------------
// runAgentViaServer (alias for runAgent — kept for backward compatibility)
// ---------------------------------------------------------------------------

/** @deprecated Use `runAgent()` instead. */
export type RunAgentViaServerOptions = RunAgentOptions2

/**
 * Alias for `runAgent()`. Kept for backward compatibility.
 * @deprecated Use `runAgent({ prompt, agentFile?, maxTurns? })` instead.
 */
export async function runAgentViaServer(opts: RunAgentViaServerOptions): Promise<string | RunAgentError> {
  return runAgent(opts)
}
