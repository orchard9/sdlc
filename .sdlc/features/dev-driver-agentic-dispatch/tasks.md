# Tasks: dev-driver-agentic-dispatch

## T1 — Add `POST /api/tools/agent-dispatch` endpoint (Rust)

Add `AgentDispatchRequest` struct and `agent_dispatch` handler to
`crates/sdlc-server/src/routes/tools.rs`.

Accepts `{ prompt, run_key, label, max_turns? }` JSON body. Validates bearer token
against `app.agent_token`. Validates that `prompt`, `run_key`, and `label` are
non-empty. Checks `agent_runs` map for duplicate `run_key` — returns 409 if found.
Calls `spawn_agent_run(run_key, prompt, opts, &app, "dev-driver", label, None)`.
Returns `202 Accepted` with `{ run_id, run_key, status: "started" }`.

Register the route in `crates/sdlc-server/src/lib.rs`:
```rust
.route("/api/tools/agent-dispatch", post(tools::agent_dispatch))
```
Place before the `{name}` wildcard tools routes.

## T2 — Add `runAgentDispatch()` to `_shared/agent.ts`

`_shared/agent.ts` already exists. Add to it:
- `AgentDispatchResult` interface: `{ run_id: string, run_key: string, status: 'started' | 'conflict' }`
- `runAgentDispatch(prompt, runKey, label, opts?)` async function

The function:
- Reads `SDLC_SERVER_URL` and `SDLC_AGENT_TOKEN` from `process.env`
- POSTs to `${serverUrl}/api/tools/agent-dispatch` with `Authorization: Bearer <token>`
- HTTP 202 → returns `{ status: 'started', run_id, run_key }`
- HTTP 409 → returns `{ status: 'conflict', run_id: '', run_key }` (no throw)
- Other errors → throws `Error`
- Timeout: 10 seconds

## T3 — Rewrite `dev-driver/tool.ts` dispatch

Remove from `dev-driver/tool.ts`:
- `LockFile` interface, `LOCK_TTL_MINS`, `lockPath`, `readLock`, `isLockActive`, `lockAgeMins`, `writeLock`
- `spawnClaude()` function
- `spawn` import from `node:child_process`
- `writeFileSync`, `readFileSync`, `existsSync`, `join` imports if no longer needed after lock removal

Rewrite Level 1 (was "Flight lock"): use `hasActiveRuns()` only. Remove the lock-file
read/check block. `hasActiveRuns()` already exists — it becomes the sole Level 1 check.

Rewrite Level 3: replace `writeLock()` + `spawnClaude()` pair with:
```typescript
const r = await runAgentDispatch(
  `/sdlc-next ${feature.feature}`,
  `dev-driver:feature:${feature.feature}`,
  `dev-driver: advance ${feature.feature}`,
  { maxTurns: 40 },
)
if (r.status === 'conflict') {
  return { ok: true, data: { action: 'waiting', reason: 'agent run in progress' }, ... }
}
```
Include `run_id` in the `feature_advanced` output.

Rewrite Level 4: replace `writeLock()` + `spawnClaude()` pair with:
```typescript
const r = await runAgentDispatch(
  `/sdlc-run-wave ${milestone}`,
  `dev-driver:wave:${milestone}`,
  `dev-driver: run wave ${milestone}`,
  { maxTurns: 100 },
)
if (r.status === 'conflict') {
  return { ok: true, data: { action: 'waiting', reason: 'agent run in progress' }, ... }
}
```

Update `DevDriverOutput` type: add optional `run_id?: string` to `feature_advanced`
and `wave_started` variants. Update `output_schema` in `meta` to match.

## T4 — Update `dev-driver/README.md`

Update to:
- Remove the "LOCK FILE" section entirely
- Add "DISPATCH" section: explain `POST /api/tools/agent-dispatch`, RunRecord, SSE
- Update Level 1 description: "Flight lock" → "Active run check"
- Update Level 3 + Level 4 descriptions: show `runAgentDispatch()` call, not `spawn`
- Update output examples: add `run_id` to `feature_advanced` and `wave_started`
- Remove lock file path (`.sdlc/.dev-driver.lock`) from all references
