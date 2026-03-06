# Design: Paused (turn limit) UX and Resume action for investigation and ponder runs

## Overview

Three layers of change: Rust data model, REST endpoints, and React UI.

## 1. Rust Data Model

### RunRecord (`crates/sdlc-server/src/state.rs`)

Add two optional fields to `RunRecord`. Both use `skip_serializing_if` for
zero-cost backward compatibility with existing persisted JSON files.

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub key: String,
    pub run_type: String,
    pub target: String,
    pub label: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub cost_usd: Option<f64>,
    pub turns: Option<u64>,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    // NEW:
    /// Claude session ID captured from the terminal ResultMessage.
    /// Required for resume. Only present when the run completed normally
    /// or paused at the turn limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// stop_reason from the terminal ResultMessage (e.g. "end_turn", "max_turns").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}
```

### spawn_agent_run (`crates/sdlc-server/src/routes/runs.rs`)

Extend the run loop to track whether the terminal message was `ErrorMaxTurns`:

```rust
let mut is_max_turns = false;
let mut final_session_id: Option<String> = None;
let mut final_stop_reason: Option<String> = None;

// In the Message::Result arm:
if let Message::Result(ref r) = message {
    is_error = r.is_error();
    is_max_turns = matches!(r, ResultMessage::ErrorMaxTurns(_));
    final_cost = Some(r.total_cost_usd());
    final_turns = Some(r.num_turns() as u64);
    final_session_id = Some(r.session_id().to_string());
    // stop_reason is Option<String> on both ResultSuccess and ResultError
    final_stop_reason = match r {
        ResultMessage::Success(s) => s.stop_reason.clone(),
        ResultMessage::ErrorMaxTurns(e)
        | ResultMessage::ErrorDuringExecution(e)
        | ResultMessage::ErrorMaxBudgetUsd(e)
        | ResultMessage::ErrorMaxStructuredOutputRetries(e) => e.stop_reason.clone(),
    };
    if is_error && !is_max_turns {
        error_msg = r.result_text().map(|s| s.to_string());
    }
    break;
}
```

Status determination:

```rust
let status = if is_max_turns {
    "paused"
} else if is_error {
    "failed"
} else {
    "completed"
};
```

`session_id` and `stop_reason` are stored on the RunRecord alongside the
existing `cost_usd`, `turns`, `error` fields.

### Orphan detection in `load_run_history`

The existing check promotes `running` → `failed` on server restart. The condition
must exclude `paused` (which is already terminal, not orphaned):

```rust
if rec.status == "running" {
    rec.status = "failed".to_string();
    // ...
}
// "paused" is terminal — no change needed
```

The current code already only touches `status == "running"`, so no change is
needed here, but the design documents the invariant explicitly.

## 2. Resume Endpoints

### POST /api/ponder/:slug/chat/resume

```
Body: { "run_id": "<run_id>" }
```

1. Look up `RunRecord` where `id == run_id` in `run_history`.
2. Validate: `status == "paused"` and `session_id.is_some()`. Return 400 otherwise.
3. Extract `session_id` from the record.
4. Build the same prompt as `start_ponder_chat` (no seed message for resume).
5. Set `opts.resume = Some(session_id)`.
6. Call `spawn_agent_run` with `run_key = "ponder:{slug}"`.
7. Return `{ "status": "started" }`.

Route registration in router: `POST /api/ponder/:slug/chat/resume`.

### POST /api/investigation/:slug/chat/resume

Same pattern. Look up paused RunRecord, build investigation chat prompt, set
`opts.resume`, call `spawn_agent_run`.

## 3. Frontend Changes

### types.ts

```ts
export type RunStatus = 'running' | 'completed' | 'failed' | 'stopped' | 'paused'

export interface RunRecord {
  // ... existing fields ...
  session_id?: string | null
  stop_reason?: string | null
}
```

### RunCard.tsx

**Status icon** — add a `'paused'` case:

```tsx
case 'paused':
  return <PauseCircle className="w-4 h-4 text-amber-400 shrink-0" />
```

**Label suffix** — when `run.status === 'paused'`, display label as
`"{run.label} · paused (turn limit)"`.

**Resume button** — shown when `status === 'paused'` and key is `ponder:*` or
`investigation:*`. Positioned next to the expand chevron, amber color:

```tsx
{run.status === 'paused' && isResumable && (
  <button
    onClick={handleResume}
    disabled={resuming}
    className="p-0.5 rounded text-amber-400 hover:bg-amber-400/10 transition-colors shrink-0 mt-0.5"
    aria-label="Resume run"
    title="Resume"
  >
    {resuming
      ? <Loader2 className="w-3.5 h-3.5 animate-spin" />
      : <Play className="w-3.5 h-3.5" />
    }
  </button>
)}
```

`isResumable` is derived from the run key prefix:

```ts
const isResumable = run.key.startsWith('ponder:') || run.key.startsWith('investigation:')
```

`handleResume` calls the appropriate endpoint:

```ts
const handleResume = useCallback(async (e: React.MouseEvent) => {
  e.stopPropagation()
  setResuming(true)
  const prefix = run.key.startsWith('ponder:') ? 'ponder' : 'investigation'
  const target = run.target
  await fetch(`/api/${prefix}/${encodeURIComponent(target)}/chat/resume`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ run_id: run.id }),
  })
  setResuming(false)
}, [run])
```

**getStopDetails** — add a `paused` guard so clicking "stop" on a paused run is
a no-op (paused runs are already terminal):

```ts
// Paused runs have no active agent to stop — but the button isn't shown
// for paused runs, so this is belt-and-suspenders.
```

## 4. API client (client.ts)

Add two new methods:

```ts
resumePonderChat: (slug: string, runId: string) =>
  request<{ status: string }>(`/api/ponder/${slug}/chat/resume`, {
    method: 'POST',
    body: JSON.stringify({ run_id: runId }),
  }),
resumeInvestigationChat: (slug: string, runId: string) =>
  request<{ status: string }>(`/api/investigation/${slug}/chat/resume`, {
    method: 'POST',
    body: JSON.stringify({ run_id: runId }),
  }),
```

## UI Wireframe

```
┌─────────────────────────────────────────────────────────┐
│ ⏸  ponder: my-idea · paused (turn limit)               │
│     14:23 · $0.42 · 50 turns                     ▶ ⌄   │
└─────────────────────────────────────────────────────────┘

Legend:
⏸  = amber PauseCircle icon
▶  = amber Play icon (Resume button)
⌄  = expand chevron
```

Contrast with failed:
```
┌─────────────────────────────────────────────────────────┐
│ ✗  ponder: my-idea                                     │
│     14:23 · max_budget_usd exceeded              ⌄      │
└─────────────────────────────────────────────────────────┘
```

## Sequence

```
Agent hits max_turns
    → claude process emits ResultMessage::ErrorMaxTurns { session_id, stop_reason: "max_turns" }
    → spawn_agent_run sets is_max_turns=true, captures session_id
    → RunRecord persisted with status="paused", session_id=<id>, stop_reason="max_turns"
    → SseMessage::RunFinished { status: "paused" } emitted
    → Frontend RunCard shows amber ⏸ icon + "· paused (turn limit)"
    → User clicks ▶ Resume
    → POST /api/ponder/:slug/chat/resume { run_id }
    → Server reads session_id from RunRecord, sets opts.resume=session_id
    → New spawn_agent_run starts — agent continues from where it left off
    → SseMessage::RunStarted emitted — spinner appears in RunCard panel
```
