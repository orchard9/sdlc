# Tasks: Capture session_id and stop_reason in RunRecord and telemetry events

## T1: Add `stop_reason()` accessor to `ResultMessage` in `claude-agent`

**File:** `crates/claude-agent/src/types.rs`

Add a `stop_reason()` method to `ResultMessage` that mirrors the existing `session_id()`, `total_cost_usd()`, and `num_turns()` accessors:

```rust
pub fn stop_reason(&self) -> Option<&str> {
    match self {
        ResultMessage::Success(r) => r.stop_reason.as_deref(),
        ResultMessage::ErrorDuringExecution(r)
        | ResultMessage::ErrorMaxTurns(r)
        | ResultMessage::ErrorMaxBudgetUsd(r)
        | ResultMessage::ErrorMaxStructuredOutputRetries(r) => r.stop_reason.as_deref(),
    }
}
```

Add a unit test verifying the accessor for `Success` and one `Error*` variant.

---

## T2: Add `session_id` and `stop_reason` fields to `RunRecord`

**File:** `crates/sdlc-server/src/state.rs`

Add two optional fields to `RunRecord`:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub session_id: Option<String>,
#[serde(skip_serializing_if = "Option::is_none")]
pub stop_reason: Option<String>,
```

Verify existing `load_run_history` tests still pass — `Option` fields default to `None` when absent in JSON.

---

## T3: Extend `RunFinished` SSE variant with new fields

**File:** `crates/sdlc-server/src/state.rs`

Update `SseMessage::RunFinished` to include the two new fields:

```rust
RunFinished {
    id: String,
    key: String,
    status: String,
    session_id: Option<String>,
    stop_reason: Option<String>,
}
```

---

## T4: Capture `session_id` and `stop_reason` in `spawn_agent_run`

**File:** `crates/sdlc-server/src/routes/runs.rs`

1. Declare capture variables alongside `final_cost` and `final_turns`:
   ```rust
   let mut final_session_id: Option<String> = None;
   let mut final_stop_reason: Option<String> = None;
   ```

2. Inside `if let Message::Result(ref r) = message`, after extracting cost and turns:
   ```rust
   final_session_id = Some(r.session_id().to_string());
   final_stop_reason = r.stop_reason().map(|s| s.to_string());
   ```

3. Initialize the initial `RunRecord` (status `"running"`) with:
   ```rust
   session_id: None,
   stop_reason: None,
   ```

4. In the completion update block (where `status`, `completed_at`, `cost_usd`, `turns`, `error` are written):
   ```rust
   rec.session_id = final_session_id.clone();
   rec.stop_reason = final_stop_reason.clone();
   ```

5. In the fallback `RunRecord` (when the run is not found in history), add:
   ```rust
   session_id: final_session_id.clone(),
   stop_reason: final_stop_reason.clone(),
   ```

6. Update the `RunFinished` emit:
   ```rust
   let _ = event_tx.send(SseMessage::RunFinished {
       id: run_id_clone,
       key: key_clone.clone(),
       status: status.to_string(),
       session_id: final_session_id,
       stop_reason: final_stop_reason,
   });
   ```

Add a unit test that constructs a `ResultSuccess` with known `session_id` and `stop_reason`, runs the extraction logic, and asserts the captured values.

---

## T5: Update frontend `RunRecord` TypeScript interface

**File:** `frontend/src/lib/types.ts`

Add two optional fields to the `RunRecord` interface:

```typescript
session_id?: string
stop_reason?: string
```

---

## T6: Run tests and verify

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

All tests must pass. No new clippy warnings.
