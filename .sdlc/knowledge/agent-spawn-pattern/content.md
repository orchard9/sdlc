# Agent Spawn Pattern

## Overview

Every server endpoint that invokes a Claude agent **must** use `spawn_agent_run` from
`crates/sdlc-server/src/routes/runs.rs`. This is the single canonical pattern for all
agent-driven features in sdlc-server. It is enforced by `CLAUDE.md` — never use raw
`tokio::spawn` for agent work.

## What `spawn_agent_run` Provides

```
spawn_agent_run(key, prompt, opts, &app, run_type, label, completion_event)
```

| Concern | Behavior |
|---|---|
| Async task spawn | Non-blocking HTTP response — agent runs in background |
| SSE event streaming | Per-run broadcast channel keyed by `key`; frontend subscribes via `/api/run/{slug}/events` |
| RunRecord persistence | JSON record in `.sdlc/.runs/{id}.json`; events sidecar in `.sdlc/.runs/{id}.events.json` |
| Lifecycle management | Emits `RunStarted`, `RunFinished` SSE; updates status (running → completed/failed/stopped) |
| Duplicate prevention | Atomic check: returns 409 Conflict if an agent for the same key is already running |
| Domain completion event | Optional `SseMessage` variant emitted after `RunFinished` (e.g. `PonderRunCompleted`) |
| Retention | Enforces 50-run cap via `enforce_retention` after each run completes |

## Signature

```rust
pub(crate) async fn spawn_agent_run(
    key: String,               // unique run key, e.g. "feature:my-slug" or "knowledge:foo"
    prompt: String,            // agent prompt
    opts: QueryOptions,        // from sdlc_query_options() or extended variant
    app: &AppState,
    run_type: &str,            // e.g. "feature", "milestone_uat", "knowledge"
    label: &str,               // display label for the activity feed
    completion_event: Option<SseMessage>, // domain-specific SSE emitted after RunFinished
) -> Result<Json<serde_json::Value>, AppError>
```

Returns `{ "status": "started", "message": "...", "run_id": "..." }`.

## Key Naming Convention

Keys use a `<domain>:<slug>` pattern. Examples from the codebase:

| Feature | Key |
|---|---|
| Feature run | `slug` (bare, e.g. `"my-feature"`) |
| Milestone UAT | `"milestone-uat:{slug}"` |
| Milestone prepare | `"milestone-prepare:{slug}"` |
| Milestone run-wave | `"milestone-run-wave:{slug}"` |
| Knowledge research | `"knowledge:{slug}"` |
| Knowledge maintenance | `"knowledge:maintain"` |
| Knowledge harvest | `"knowledge:harvest:{slug}"` |
| Advisory | `"advisory"` |
| Tool plan | `"foo-plan:{name}"` |
| Tool act | `"foo-act:{name}"` |

The `target` field in RunRecord is extracted from `key.split(':').last()`.

## Query Options

Two base option builders exist in `runs.rs`:

### `sdlc_query_options(root, max_turns)`

Standard options for all feature/agent work:
- `PermissionMode::AcceptEdits` — agent can write files without prompts
- MCP server: `sdlc mcp` (the sdlc MCP server via the current binary)
- Allowed tools: `Bash`, `Read`, `Write`, `Edit`, `Glob`, `Grep`, plus all `mcp__sdlc__*` tools
- `cwd` set to project root

### `sdlc_guideline_query_options(root, max_turns)`

Extends `sdlc_query_options` with `WebSearch` and `WebFetch` for Prior Art Mapper.

### UAT Extension

Milestone UAT endpoints extend `sdlc_query_options` by pushing a Playwright MCP server
and the full set of `mcp__playwright__browser_*` tools. **Never** add Playwright tools
to the base options — they are UAT-only.

## SSE Architecture

The active-runs map is `Arc<Mutex<HashMap<String, AgentRunEntry>>>` where
`AgentRunEntry = (broadcast::Sender<String>, AbortHandle)`.

Event flow:
```
agent task → tokio::spawn → claude_agent::query() stream
    → each Message serialized to JSON → sent on broadcast channel
    → SSE subscribers receive via GET /api/run/{key}/events
    → on complete: RunFinished + optional domain event emitted on app.event_tx
```

Frontend uses `AgentRunContext` to track `isRunning(key)` based on `RunStarted`/`RunFinished` SSE.

## RunRecord Schema

```rust
pub struct RunRecord {
    pub id: String,          // timestamp-based: "20260302-143022-abc"
    pub key: String,
    pub run_type: String,
    pub target: String,      // last segment of key after ':'
    pub label: String,
    pub status: String,      // "running" | "completed" | "failed" | "stopped"
    pub started_at: String,  // RFC3339
    pub completed_at: Option<String>,
    pub cost_usd: Option<f64>,
    pub turns: Option<u64>,
    pub error: Option<String>,
    pub prompt: Option<String>, // truncated to 2000 chars for display
}
```

On server restart, any `running` records are marked `stopped` during `load_run_history`.

## Adding a New Agent Endpoint

1. **Add query options** — use `sdlc_query_options` or extend it
2. **Add SSE completion variant** in `state.rs` → `SseMessage`
3. **Call `spawn_agent_run`** with a unique key, prompt, opts, run_type, label, and completion event
4. **Add route triplet** (start, events, stop) using `get_run_events` and `stop_run_by_key` helpers
5. **Register routes** in `lib.rs` — specific routes before `{slug}` wildcards

## Plan-Act Pattern (Two-Phase)

For operations needing user review before irreversible writes, use the Plan-Act pattern:

```
POST /api/<domain>/plan  →  agent reads context, streams structured plan
POST /api/<domain>/act   →  agent receives plan (possibly adjusted), implements it
```

Both phases use `spawn_agent_run`. The frontend captures Phase 1 output via
`AmaAnswerPanel`'s `onDone` callback, lets the user adjust, then passes the text to Phase 2.

**Frontend modal state machine:** `form → planning → adjusting → acting → done`

Existing usages:
- Quality gates: `POST /api/tools/quality-check/reconfigure` + `/fix`
- Tool creation: `POST /api/tools/plan` + `POST /api/tools/build`

See `docs/plan-act-pattern.md` for the complete recipe.

## Anti-Pattern: Legacy Exception

`suggest.rs` uses `max_turns: 1` with no tools — this predates the SSE infrastructure
and is a known exception. **Do not copy this pattern.**

## Files

| File | Role |
|---|---|
| `crates/sdlc-server/src/routes/runs.rs` | `spawn_agent_run`, `sdlc_query_options`, all run/milestone endpoints |
| `crates/sdlc-server/src/state.rs` | `AppState`, `RunRecord`, `SseMessage`, persistence helpers |
| `crates/sdlc-server/src/routes/advisory.rs` | Advisory agent — imports `spawn_agent_run` |
| `crates/sdlc-server/src/routes/knowledge.rs` | Knowledge research/maintenance agents — imports `spawn_agent_run` |
| `crates/sdlc-server/src/lib.rs` | Route registration |
| `docs/plan-act-pattern.md` | Two-phase plan→act recipe |
