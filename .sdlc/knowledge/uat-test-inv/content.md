# Agent Spawn Pattern

## Overview

`spawn_agent_run` is the **single mandatory pattern** for all server endpoints that invoke Claude agents. It lives in `crates/sdlc-server/src/routes/runs.rs` and provides a complete, consistent infrastructure for agent lifecycle management.

> "All agent-invoked operations use `spawn_agent_run` in `routes/runs.rs` — never raw" — ARCHITECTURE.md

---

## Function Signature

```rust
pub(crate) async fn spawn_agent_run(
    key: String,
    prompt: String,
    opts: QueryOptions,
    app: &AppState,
    run_type: &str,
    label: &str,
    completion_event: Option<SseMessage>,
) -> Result<Json<serde_json::Value>, AppError>
```

**Parameters:**

| Param | Description |
|---|---|
| `key` | Unique run identifier, format `"domain:slug"` (e.g. `"feature:my-slug"`, `"milestone-uat:v08"`) |
| `prompt` | Instruction passed directly to Claude |
| `opts` | `QueryOptions` — tool permissions, MCP servers, turn limit |
| `app` | `AppState` — access to broadcast channels, run history, root path |
| `run_type` | String category for display: `"feature"`, `"milestone_uat"`, `"ponder"`, `"advisory"`, etc. |
| `label` | Human-readable label shown in the activity feed |
| `completion_event` | Optional domain-specific SSE emitted after `RunFinished` (e.g. `MilestoneUatCompleted`) |

---

## What It Provides

1. **Async task spawn** — agent runs on a `tokio::spawn` task; HTTP response returns immediately with `{ "status": "started", "run_id": "...", "message": "..." }`
2. **SSE event streaming** — broadcast channel sends raw agent events to subscribers via `/api/run/{slug}/events`
3. **RunRecord persistence** — records lifecycle metadata to `.sdlc/.runs/{id}.json`
4. **Events sidecar** — all raw agent events written to `.sdlc/.runs/{id}.events.json`
5. **Duplicate prevention** — returns 409 Conflict if an agent is already running for the same `key`
6. **Retention enforcement** — keeps last 50 runs (`enforce_retention`)
7. **Lifecycle SSE events** — emits `RunStarted` on spawn, `RunFinished` on completion, then optional `completion_event`

---

## RunRecord Structure

```rust
pub struct RunRecord {
    pub id: String,           // e.g. "20260227-143022-abc"
    pub key: String,          // e.g. "feature:my-slug"
    pub run_type: String,     // "feature", "milestone_uat", "ponder", etc.
    pub target: String,       // last segment of key (the slug)
    pub label: String,
    pub status: String,       // "running" | "completed" | "failed" | "stopped"
    pub started_at: String,   // RFC3339
    pub completed_at: Option<String>,
    pub cost_usd: Option<f64>,
    pub turns: Option<u64>,
    pub error: Option<String>,
    pub prompt: Option<String>, // truncated to 2000 chars
}
```

---

## Standard Query Options: `sdlc_query_options`

```rust
pub(crate) fn sdlc_query_options(root: PathBuf, max_turns: u32) -> QueryOptions {
    QueryOptions {
        permission_mode: PermissionMode::AcceptEdits,
        mcp_servers: vec![McpServerConfig {
            name: "sdlc", command: "<current_exe>", args: vec!["mcp"],
        }],
        allowed_tools: vec![
            "Bash", "Read", "Write", "Edit", "Glob", "Grep",
            "mcp__sdlc__sdlc_get_directive", "mcp__sdlc__sdlc_write_artifact",
            "mcp__sdlc__sdlc_approve_artifact", "mcp__sdlc__sdlc_reject_artifact",
            "mcp__sdlc__sdlc_add_task", "mcp__sdlc__sdlc_complete_task",
            "mcp__sdlc__sdlc_add_comment", "mcp__sdlc__sdlc_merge",
        ],
        cwd: Some(root),
        max_turns: Some(max_turns),
    }
}
```

- `PermissionMode::AcceptEdits` — file writes auto-approved; no confirmation prompts
- MCP server is the `sdlc` binary itself (`sdlc mcp` subcommand)

---

## Specialized Option Variants

### Guideline Investigations (`sdlc_guideline_query_options`)
Extends `sdlc_query_options` with `WebSearch` and `WebFetch` for the Prior Art Mapper perspective.

### UAT Agent (Playwright MCP)
`start_milestone_uat` extends standard options with a Playwright MCP server and browser tools:
- `mcp__playwright__browser_navigate`, `browser_click`, `browser_type`, `browser_snapshot`, `browser_take_screenshot`, `browser_console_messages`, `browser_wait_for`

**Pattern:**
```rust
let mut opts = sdlc_query_options(app.root.clone(), 200);
opts.mcp_servers.push(McpServerConfig { name: "playwright", command: "npx", args: vec!["@playwright/mcp@latest"] });
opts.allowed_tools.extend([/* playwright tools */]);
```

---

## Key Naming Convention

Keys follow the format `"domain:slug"`:

| Key | Domain |
|---|---|
| `"my-slug"` | Feature runs (just the slug) |
| `"milestone-uat:my-milestone"` | Milestone UAT |
| `"ponder:my-slug:NNN"` | Ponder sessions (includes session number) |
| `"advisory"` | Advisory analysis runs |
| `"foo-plan:my-name"` | Plan phase (Plan-Act pattern) |
| `"foo-act:my-name"` | Act phase (Plan-Act pattern) |

---

## SSE Lifecycle Event Flow

```
1. spawn_agent_run called
   └─ RunRecord created in memory + persisted
   └─ tokio::spawn fires the agent task
   └─ SSE: RunStarted { id, key, label }
   └─ HTTP returns immediately: { "status": "started", "run_id": "..." }

2. Agent runs (streaming events via broadcast channel)
   └─ Each agent message → JSON → broadcast to /api/run/events SSE subscribers
   └─ Events accumulated in memory for sidecar persistence

3. Agent completes (or errors)
   └─ RunRecord updated: status, completed_at, cost_usd, turns
   └─ Persisted: {id}.json + {id}.events.json
   └─ SSE: RunFinished { id, key, status }
   └─ Active run removed from agent_runs map
   └─ SSE: <completion_event> (if provided, e.g. MilestoneUatCompleted)
```

---

## Plan-Act Pattern (Two-Phase Workflow)

When an action benefits from user review before irreversible writes, use two `spawn_agent_run` calls:

```
POST /api/<domain>/plan  →  Phase 1: agent reads context, streams a plan
POST /api/<domain>/act   →  Phase 2: agent receives the plan, implements it
```

Frontend state machine: `form → planning → adjusting → acting → done`

- Phase 1 uses `AmaAnswerPanel`'s `onDone` callback to capture plan text
- User can adjust in the "adjusting" step
- Phase 2 receives `{ name, plan: planText + adjustments }`

Existing usages: quality gates (reconfigure → fix), tool creation (plan → build).

See `docs/plan-act-pattern.md` for the full recipe.

---

## Common Usage Example

```rust
pub async fn start_run(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    let opts = sdlc_query_options(app.root.clone(), 200);
    let prompt = format!(
        "Drive feature '{}' through the sdlc state machine. \
         Run `sdlc next --for {} --json` to get the next action, \
         execute it, then loop until done.",
        slug, slug
    );
    spawn_agent_run(slug, prompt, opts, &app, "feature", &slug, None).await
}
```

---

## Anti-Patterns

- **Never** use `max_turns: 1` with no tools for a feature that reads files or makes decisions — that is the legacy `suggest.rs` pattern, do not copy it
- **Never** spawn raw `tokio::spawn` with `query()` directly — always use `spawn_agent_run`
- **Never** add Playwright tools to `sdlc_query_options` base — they are UAT-only

---

## Source Files

| File | Purpose |
|---|---|
| `crates/sdlc-server/src/routes/runs.rs` | `spawn_agent_run`, `sdlc_query_options`, all endpoint handlers |
| `crates/sdlc-server/src/state.rs` | `RunRecord`, `SseMessage` variants, persistence helpers |
| `docs/plan-act-pattern.md` | Full Plan-Act pattern recipe with Rust + frontend code |
| `CLAUDE.md` (Server Pattern section) | Canonical guidance for agent endpoints |
