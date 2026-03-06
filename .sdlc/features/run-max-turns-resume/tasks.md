# Tasks: Paused (turn limit) UX and Resume action for investigation and ponder runs

## T1 — Add session_id and stop_reason fields to RunRecord

**File:** `crates/sdlc-server/src/state.rs`

Add `session_id: Option<String>` and `stop_reason: Option<String>` to
`RunRecord` with `#[serde(skip_serializing_if = "Option::is_none")]` and
`#[serde(default)]` for forward/backward compat.

## T2 — Capture session_id, stop_reason, and is_max_turns in spawn_agent_run

**File:** `crates/sdlc-server/src/routes/runs.rs`

In the `Message::Result` arm of the run loop:
- Set `is_max_turns = matches!(r, ResultMessage::ErrorMaxTurns(_))`
- Capture `final_session_id = Some(r.session_id().to_string())`
- Capture `final_stop_reason` from the `stop_reason` field of either
  `ResultSuccess` or `ResultError`
- Do NOT set `error_msg` for `ErrorMaxTurns` (it's not a failure)

Update status determination:
```rust
let status = if is_max_turns { "paused" } else if is_error { "failed" } else { "completed" };
```

Write `session_id` and `stop_reason` onto the RunRecord in the history update block and in the fallback minimal record.

## T3 — Add resume endpoint for ponder

**File:** `crates/sdlc-server/src/routes/runs.rs`

`POST /api/ponder/:slug/chat/resume` handler:
- Deserializes `{ "run_id": String }`
- Looks up RunRecord in `run_history`, validates `status == "paused"` and `session_id.is_some()`
- Returns 400 if not paused or no session_id
- Builds ponder prompt (same as `start_ponder_chat`, no seed message)
- Sets `opts.resume = run_record.session_id`
- Calls `spawn_agent_run` with `run_key = "ponder:{slug}"`
- Returns `{ "status": "started" }`

Register in the router alongside existing ponder routes.

## T4 — Add resume endpoint for investigation

**File:** `crates/sdlc-server/src/routes/runs.rs` and/or `investigations.rs`

`POST /api/investigation/:slug/chat/resume` handler — same pattern as T3 but
uses `run_key = "investigation:{slug}"` and the investigation chat prompt.

Register in the router alongside existing investigation routes.

## T5 — Update RunStatus type and RunRecord interface in frontend

**File:** `frontend/src/lib/types.ts`

- Add `'paused'` to `RunStatus` union
- Add `session_id?: string | null` and `stop_reason?: string | null` to `RunRecord` interface

## T6 — Update RunCard to show paused state and Resume button

**File:** `frontend/src/components/layout/RunCard.tsx`

- Import `PauseCircle` and `Play` from lucide-react
- Add `'paused'` case to `StatusIcon` returning `<PauseCircle className="... text-amber-400 ..." />`
- When `run.status === 'paused'`, append `" · paused (turn limit)"` to the label text
- Add `resuming` state (`useState(false)`)
- Derive `isResumable = run.key.startsWith('ponder:') || run.key.startsWith('investigation:')`
- Add Resume button shown when `status === 'paused' && isResumable`
- `handleResume` calls `POST /api/ponder/:slug/chat/resume` or
  `POST /api/investigation/:slug/chat/resume` with `{ run_id: run.id }`

## T7 — Add resumePonderChat and resumeInvestigationChat to API client

**File:** `frontend/src/api/client.ts`

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

Update `handleResume` in RunCard to use these typed client methods.

## T8 — Write tests

**Files:**
- `crates/sdlc-server/src/routes/runs.rs` (unit tests in existing `mod tests`)
- Integration: verify `status == "paused"` for simulated `ErrorMaxTurns` result

Test cases:
1. Simulate `ResultMessage::ErrorMaxTurns` in `spawn_agent_run` flow — verify
   RunRecord has `status = "paused"`, `session_id` set, `stop_reason = "max_turns"`,
   and `error = None`.
2. Verify `ResultMessage::Success` still produces `status = "completed"` with
   `session_id` populated.
3. Verify `ResultMessage::ErrorDuringExecution` produces `status = "failed"`.
4. Verify backward compat: deserialize a RunRecord JSON without
   `session_id`/`stop_reason` fields succeeds.
