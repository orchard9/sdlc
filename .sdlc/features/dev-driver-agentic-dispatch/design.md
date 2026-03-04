# Design: dev-driver-agentic-dispatch

## Architecture

The change replaces a fire-and-forget subprocess pattern with a proper HTTP call into
the existing server agent infrastructure. All agent dispatch in the codebase already
flows through `spawn_agent_run` — this feature brings `dev-driver` in line with that
invariant.

```
Before:
  dev-driver/tool.ts
    └── spawnClaude("/sdlc-next slug")
          └── node child_process.spawn("claude", ["--print", ...], { detached: true })
                  ⟶ invisible to server, no RunRecord, no SSE

After:
  dev-driver/tool.ts
    └── runAgentDispatch(prompt, runKey, label)   ← _shared/agent.ts (new function)
          └── POST /api/tools/agent-dispatch       ← new non-blocking endpoint
                └── spawn_agent_run(key, prompt, opts, app, "dev-driver", label, None)
                        ⟶ RunRecord persisted, SSE streamed, activity feed updated
                        ⟶ returns 202 immediately (fire-and-forget)
```

### Why a New Endpoint (Not the Existing `agent-call`)

`POST /api/tools/agent-call` already exists but **blocks synchronously** until the
agent completes (up to 10 min). Dev-driver is a short-lived tool: it dispatches work
and exits immediately. Blocking would defeat the purpose and eventually time out for
long-running `/sdlc-run-wave` dispatches.

A new `POST /api/tools/agent-dispatch` endpoint returns `202 Accepted` immediately
after inserting the run into `agent_runs` — the agent runs in the background.

## Component Design

### 1. `POST /api/tools/agent-dispatch` (Rust, `routes/tools.rs`)

New handler added to `tools.rs`, registered in `lib.rs` before the `{name}` wildcard.

```rust
#[derive(serde::Deserialize)]
pub struct AgentDispatchRequest {
    pub prompt: String,      // the slash command or free text prompt
    pub run_key: String,     // deduplication key (e.g. "dev-driver:feature:my-feature")
    pub label: String,       // human-readable label for activity feed
    pub max_turns: Option<u32>,  // default 40, capped at 100
}
```

- Validates `Authorization: Bearer <SDLC_AGENT_TOKEN>` header → 401 if invalid
- Checks `agent_runs` map for existing run with `run_key` → 409 Conflict if found
- Calls `spawn_agent_run(run_key, prompt, opts, &app, "dev-driver", label, None)`
- Returns `202 Accepted` immediately:
  ```json
  { "run_id": "20260303-120000-abc", "run_key": "dev-driver:feature:my-feature", "status": "started" }
  ```

409 Conflict response (key already in flight):
```json
{ "error": "Agent already running for 'dev-driver:feature:my-feature'" }
```

Uses `sdlc_query_options(app.root.clone(), max_turns)`.

**Validation:**
- `prompt` must be non-empty
- `run_key` must be non-empty
- `label` must be non-empty

### 2. `_shared/agent.ts` — New Function: `runAgentDispatch()`

The file already exists with `runAgentViaServer()` (blocking). Add a new non-blocking
export:

```typescript
export interface AgentDispatchResult {
  run_id: string
  run_key: string
  status: 'started' | 'conflict'
}

export async function runAgentDispatch(
  prompt: string,
  runKey: string,
  label: string,
  opts?: { maxTurns?: number },
): Promise<AgentDispatchResult>
```

- Reads `SDLC_SERVER_URL` and `SDLC_AGENT_TOKEN` from `process.env`
- POSTs to `${serverUrl}/api/tools/agent-dispatch`
- HTTP 202 → `{ status: 'started', run_id, run_key }`
- HTTP 409 → `{ status: 'conflict', run_id: '', run_key }` (not thrown — caller decides)
- Other HTTP errors → throw `Error`
- Timeout: 10 seconds (lightweight HTTP op)

### 3. `dev-driver/tool.ts` Changes

**Removed:**
- `LockFile` interface
- `LOCK_TTL_MINS` constant
- `lockPath()`, `readLock()`, `isLockActive()`, `lockAgeMins()`, `writeLock()` functions
- `spawnClaude()` function
- `existsSync`, `readFileSync`, `writeFileSync` imports (if no longer needed)
- `spawn` import from `node:child_process`

**Changed Level 1 (Lock file check → Active run check):**

The TTL-based lock file Level 1 is replaced with the existing `hasActiveRuns()` check
(which queries `sdlc run list --status running`). This is already present at Level 3 as
a defensive gate — it moves to Level 1.

```typescript
// Level 1: Active run check (replaces lock file)
if (hasActiveRuns(root)) {
  log.info('active sdlc agent run detected — waiting')
  return { ok: true, data: { action: 'waiting', reason: 'agent run in progress' }, ... }
}
```

**Changed Level 3 (spawnClaude → runAgentDispatch):**

```typescript
const r = await runAgentDispatch(
  `/sdlc-next ${feature.feature}`,
  `dev-driver:feature:${feature.feature}`,
  `dev-driver: advance ${feature.feature}`,
  { maxTurns: 40 },
)
if (r.status === 'conflict') {
  log.info(`agent already running for ${feature.feature} — waiting`)
  return { ok: true, data: { action: 'waiting', reason: 'agent run in progress' }, ... }
}
```

**Changed Level 4 (spawnClaude → runAgentDispatch):**

```typescript
const r = await runAgentDispatch(
  `/sdlc-run-wave ${milestone}`,
  `dev-driver:wave:${milestone}`,
  `dev-driver: run wave ${milestone}`,
  { maxTurns: 100 },
)
if (r.status === 'conflict') {
  log.info(`wave already running for ${milestone} — waiting`)
  return { ok: true, data: { action: 'waiting', reason: 'agent run in progress' }, ... }
}
```

**Output type update:**
`feature_advanced` output gains optional `run_id` field from the dispatch result.

### 4. Key/Namespace Design

Keys follow `dev-driver:<type>:<identifier>` pattern:

| Dispatch | Key |
|---|---|
| Feature advancement | `dev-driver:feature:<slug>` |
| Wave start | `dev-driver:wave:<milestone>` |

The `spawn_agent_run` duplicate check uses exact key equality — these namespaced keys
prevent false conflicts with other agent run types (`sdlc-run:feature:slug`, etc.).

## Sequence Diagram

```
orchestrator tick
      |
      v
dev-driver tool.ts
  1. hasActiveRuns() → sdlc run list --status running
     if any → return waiting
  2. runQualityCheck() → quality-check tool
     if failing → return quality_failing
  3. findActionableFeature() → sdlc next --json
     if found →
       runAgentDispatch("/sdlc-next slug", "dev-driver:feature:slug", label)
         │
         └─ POST /api/tools/agent-dispatch
               │
               └─ spawn_agent_run("dev-driver:feature:slug", prompt, opts, app)
                    │
                    ├─ RunRecord created (status: running)
                    ├─ SSE: RunStarted { id, key, label }
                    └─ agent task spawned (async background)
         ←── 202 { run_id, run_key, status: "started" }   [returns immediately]
       return feature_advanced { slug, phase, directive, run_id }
  4. findReadyWave()
     if found →
       runAgentDispatch("/sdlc-run-wave milestone", "dev-driver:wave:milestone", label)
       return wave_started { milestone }
  5. return idle
```

## Failure Modes

| Failure | Behavior |
|---|---|
| Server not running | `runAgentDispatch` throws → tool exits with `ok: false` |
| Run already in flight (409) | Returns `status: 'conflict'` → tool returns `waiting` |
| Server 5xx | `runAgentDispatch` throws → tool exits with `ok: false` |
| `sdlc run list` unavailable | `hasActiveRuns` returns false (fail-open, existing behavior) |

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-server/src/routes/tools.rs` | Add `AgentDispatchRequest` + `agent_dispatch` handler (~60 lines) |
| `crates/sdlc-server/src/lib.rs` | Register `POST /api/tools/agent-dispatch` route |
| `.sdlc/tools/_shared/agent.ts` | Add `AgentDispatchResult` + `runAgentDispatch()` function |
| `.sdlc/tools/dev-driver/tool.ts` | Remove lock file code, replace `spawnClaude` with `runAgentDispatch` |
| `.sdlc/tools/dev-driver/README.md` | Update dispatch docs, remove lock file section |
