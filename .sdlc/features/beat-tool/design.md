# Design: beat-tool

## Architecture Overview

The beat tool is a TypeScript SDLC tool that orchestrates three shared primitives into a single agentic evaluation flow. It lives at `.sdlc/tools/beat/tool.ts` and follows the established tool protocol (`--meta`, `--run`).

```
sdlc tool run beat --input '{"scope":"project","mode":"evaluate"}'
         │
         ▼
    tool.ts run()
         │
         ├─── [gather]    _shared/sdlc.ts  ──► sdlc CLI (feature list, milestone list, VISION.md)
         │
         ├─── [recruit]   _shared/agent.ts ──► sdlc ponder recruit (if agent missing)
         │                                     load .claude/agents/<slug>.md
         │
         ├─── [evaluate]  _shared/agent.ts ──► claude --print <agent> <prompt>
         │                                     parse verdict JSON from response
         │
         ├─── [write]     writeBeat()      ──► .sdlc/beat.yaml (append)
         │
         └─── [done]      ToolResult        ► stdout
```

## Module Structure

### `.sdlc/tools/beat/tool.ts`
The main tool file. Implements `meta`, `run`, and the two modes (evaluate/week). Imports from `_shared/`.

### `.sdlc/tools/_shared/sdlc.ts` (new shared primitive)
Wraps the `sdlc` CLI to expose project state as typed TypeScript objects:

```typescript
export function readVision(root: string): string
export function readFeatures(root: string): Feature[]
export function readMilestones(root: string): Milestone[]
export function readFeatureDetail(root: string, slug: string): FeatureDetail
```

All functions use `execSync` to shell out to `sdlc` CLI commands. They throw on failure (callers catch and emit NDJSON error events).

### `.sdlc/tools/_shared/agent.ts` (new shared primitive)
Recruits and invokes named agents:

```typescript
export function ensureAgent(root: string, slug: string, roleDescription: string): string
// Returns: path to agent file (absolute)
// Side effect: calls `sdlc ponder recruit <slug> --role "<description>"` if file missing

export async function runAgent(
  agentPath: string,
  prompt: string,
  opts?: { timeout_ms?: number }
): Promise<string>
// Returns: raw text output from the agent
// Uses: `claude --print --agent <agentPath> <prompt>` or equivalent
```

## Data Flow: evaluate mode

1. **Gather state** — call `readVision`, `readFeatures`, `readMilestones`
2. **Filter by scope** — if `scope: 'feature:<slug>'`, load just that feature; if domain, filter by milestone/tag
3. **Compose prompt** — build evaluation prompt from vision + state summary
4. **Ensure agent** — load or recruit `cto-cpo-lens` agent (project scope) or `tech-lead-lens` (feature scope)
5. **Run agent** — invoke via `runAgent()`, parse structured output (verdict + concerns)
6. **Write beat** — append record to `.sdlc/beat.yaml`
7. **Return result** — emit `done` event with full `ToolResult`

## Data Flow: week mode

1. **Read beats** — load `.sdlc/beat.yaml`, filter to last N days (default 14)
2. **Synthesize** — group concerns by recurrence, score by frequency × severity
3. **Return top-5** — emit `done` with `week_items` array (no agent invocation, no write)

## beat.yaml Format

```yaml
beats:
  - id: beat-001
    scope: project
    mode: evaluate
    timestamp: 2026-03-03T12:00:00Z
    verdict: on-track          # on-track | drifting | off-course
    score: 78                  # 0-100 alignment score
    concerns:
      - "3 features in implementation >14 days"
      - "No QA plan for citadel-webhook-handler"
```

IDs are sequential: `beat-001`, `beat-002`, etc. The file is append-only; never rewrite existing records.

## NDJSON Streaming Protocol

The tool writes NDJSON to stdout as it progresses. Each line is a valid JSON object:

```
{"event":"gathering","message":"Reading project state (14 features, 6 milestones)..."}
{"event":"recruiting","message":"Loading cto-cpo-lens agent from .claude/agents/cto-cpo-lens.md"}
{"event":"evaluating","message":"Agent evaluating 14 features against VISION.md..."}
{"event":"writing","message":"Persisting beat-001 to .sdlc/beat.yaml"}
{"event":"done","result":{"ok":true,"data":{"verdict":"on-track","score":78,"concerns":[...],"beat_id":"beat-001"}}}
```

Event types: `gathering`, `recruiting`, `evaluating`, `writing`, `done`, `error`.

The `done` event always carries the full `ToolResult`. Callers that want only the final result can run the tool and parse only the last line.

## Agent Prompt Design

For `scope: 'project'` evaluation, the prompt sent to the agent is:

```
You are reviewing the sdlc project against its vision. Your task is to produce
a leadership-level verdict on project direction.

VISION:
<VISION.md content>

CURRENT STATE:
- <N> features across <phases>
- Active milestones: <list>
- Feature summary: <key items>

Respond with JSON only:
{
  "verdict": "on-track" | "drifting" | "off-course",
  "score": <0-100>,
  "concerns": ["<concern 1>", "<concern 2>", ...]
}
```

The tool parses the JSON from the agent response. If parsing fails, it retries once with an explicit JSON-only instruction. On second failure, it returns an error result.

## Agent Selection

| Scope | Agent slug | Role description |
|---|---|---|
| `project` | `cto-cpo-lens` | Strategic CTO/CPO evaluating product direction vs. vision |
| `feature:<slug>` | `tech-lead-lens` | Tech lead evaluating feature health, blockers, timeline |
| `<domain>` | `cto-cpo-lens` | Same as project, filtered to domain context |

The `ensureAgent` function checks if `.claude/agents/<slug>.md` exists. If missing, it calls `sdlc ponder recruit <slug> --role "<description>"` to create the agent file.

## Error Handling

- `readVision` falls back to an empty string if VISION.md is missing (non-fatal)
- `readFeatures` returns empty array if CLI fails (logs warning, emits `gathering` event with note)
- `runAgent` times out at 60s by default; timeout is configurable
- If agent recruitment fails, the tool returns an error result (does not attempt evaluation)
- beat.yaml write failure returns error result (does not silently succeed)

## Configuration

No `config.yaml` required for the beat tool. All configuration is via input schema. Future extension: add `config.yaml` support for default scope, window size for week mode, and agent timeout.

## File Layout

```
.sdlc/tools/
  _shared/
    sdlc.ts        (new — project state reader)
    agent.ts       (new — agent recruiter/invoker)
    types.ts       (existing)
    log.ts         (existing)
    config.ts      (existing)
    runtime.ts     (existing)
  beat/
    tool.ts        (new — main tool)
    README.md      (new — usage docs)
.sdlc/
  beat.yaml        (created on first run)
```

## Sequence Diagram

```
User                beat/tool.ts         _shared/sdlc.ts     _shared/agent.ts    claude CLI
 │                       │                      │                    │                │
 │  sdlc tool run beat   │                      │                    │                │
 │──────────────────────►│                      │                    │                │
 │                       │─── readVision() ────►│                    │                │
 │  {"event":"gathering"}│◄────── vision ────────│                    │                │
 │◄──────────────────────│─── readFeatures() ──►│                    │                │
 │                       │◄──── features ────────│                    │                │
 │                       │─── ensureAgent() ─────────────────────────►                │
 │  {"event":"recruiting"}│◄─── agentPath ─────────────────────────────               │
 │◄──────────────────────│─── runAgent() ─────────────────────────────────────────────►
 │  {"event":"evaluating"}│◄──── response ──────────────────────────────────────────────│
 │◄──────────────────────│─── writeBeat() ──────────────────►                         │
 │  {"event":"writing"}  │                                                             │
 │◄──────────────────────│─── emit done ─────────────────────────────────────────────►│
 │  {"event":"done",...} │                                                             │
```

## Dependencies

- No new npm packages required
- Uses `node:fs`, `node:child_process` (already used by dev-driver)
- TypeScript with `node:*` imports (same as all other tools)
- Runtime: Bun (primary), Node.js (fallback) — same as other tools
