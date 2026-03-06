# Design: Capture session_id and stop_reason in RunRecord and telemetry events

## Overview

This is a backend-only data-capture change. The design covers:
1. Struct changes to `RunRecord`
2. Capture logic in `spawn_agent_run`
3. SSE payload extension for `RunFinished`
4. `ResultMessage` helper method additions
5. Frontend TypeScript interface update

---

## 1. `RunRecord` struct changes (`crates/sdlc-server/src/state.rs`)

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
    // NEW FIELDS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}
```

Both fields use `skip_serializing_if = "Option::is_none"` so existing persisted JSON (without these keys) deserializes correctly via `#[serde(default)]` semantics — `Option` fields default to `None` when the key is absent.

---

## 2. `ResultMessage` helper methods (`crates/claude-agent/src/types.rs`)

`ResultSuccess` and `ResultError` both have `stop_reason: Option<String>` and `session_id: String`. Add a `stop_reason()` accessor to `ResultMessage` (mirroring the existing `session_id()`, `total_cost_usd()`, `num_turns()` accessors):

```rust
impl ResultMessage {
    pub fn stop_reason(&self) -> Option<&str> {
        match self {
            ResultMessage::Success(r) => r.stop_reason.as_deref(),
            ResultMessage::ErrorDuringExecution(r)
            | ResultMessage::ErrorMaxTurns(r)
            | ResultMessage::ErrorMaxBudgetUsd(r)
            | ResultMessage::ErrorMaxStructuredOutputRetries(r) => r.stop_reason.as_deref(),
        }
    }
}
```

---

## 3. Capture logic in `spawn_agent_run` (`crates/sdlc-server/src/routes/runs.rs`)

### New local variables (alongside `final_cost`, `final_turns`)

```rust
let mut final_session_id: Option<String> = None;
let mut final_stop_reason: Option<String> = None;
```

### Capture on `Message::Result`

Inside the `if let Message::Result(ref r) = message` block, after extracting cost and turns:

```rust
final_session_id = Some(r.session_id().to_string());
final_stop_reason = r.stop_reason().map(|s| s.to_string());
```

### Write to `RunRecord` at completion

In the lock block where `status`, `completed_at`, `cost_usd`, `turns`, and `error` are written:

```rust
rec.session_id = final_session_id.clone();
rec.stop_reason = final_stop_reason.clone();
```

Also add them to the fallback `RunRecord` constructed when the run is not found in history.

---

## 4. `RunRecord` initialization in `spawn_agent_run`

When the initial `RunRecord` is constructed (status `"running"`), set:

```rust
session_id: None,
stop_reason: None,
```

---

## 5. `RunFinished` SSE event (`crates/sdlc-server/src/state.rs`)

The `SseMessage::RunFinished` variant currently carries `id`, `key`, and `status`. Extend to include the new fields:

```rust
RunFinished {
    id: String,
    key: String,
    status: String,
    session_id: Option<String>,   // NEW
    stop_reason: Option<String>,  // NEW
}
```

Update the emit site in `spawn_agent_run`:

```rust
let _ = event_tx.send(SseMessage::RunFinished {
    id: run_id_clone,
    key: key_clone.clone(),
    status: status.to_string(),
    session_id: final_session_id,
    stop_reason: final_stop_reason,
});
```

---

## 6. Frontend `RunRecord` interface (`frontend/src/lib/types.ts`)

```typescript
export interface RunRecord {
  id: string
  key: string
  run_type: RunType
  target: string
  label: string
  status: RunStatus
  started_at: string
  completed_at?: string
  cost_usd?: number
  turns?: number
  error?: string
  prompt?: string | null
  session_id?: string    // NEW
  stop_reason?: string   // NEW
}
```

---

## Affected files

| File | Change |
|---|---|
| `crates/sdlc-server/src/state.rs` | Add `session_id`, `stop_reason` to `RunRecord`; extend `RunFinished` variant |
| `crates/claude-agent/src/types.rs` | Add `stop_reason()` accessor to `ResultMessage` |
| `crates/sdlc-server/src/routes/runs.rs` | Capture fields from `Message::Result`; write to `RunRecord`; emit in `RunFinished` |
| `frontend/src/lib/types.ts` | Add optional fields to `RunRecord` interface |

---

## Test coverage

- Unit test in `runs.rs`: construct a `ResultSuccess` with `stop_reason = Some("end_turn")` and `session_id = "s1"`, verify both fields are present on the resulting `RunRecord`
- Unit test in `types.rs`: verify `stop_reason()` accessor returns the correct value for `Success` and each `Error*` variant
- Existing tests must continue to pass — no logic changes, only additive fields
