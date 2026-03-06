# QA Plan: Capture session_id and stop_reason in RunRecord and telemetry events

## Test Strategy

This is a data-capture feature with no user-visible UI changes. QA focuses on:
1. Unit tests for the new `stop_reason()` accessor
2. Unit tests verifying field capture in `spawn_agent_run`
3. Backward compatibility for existing run JSON files
4. Compilation and clippy clean

---

## Test Cases

### TC1: `ResultMessage::stop_reason()` accessor

**Location:** `crates/claude-agent/src/types.rs`

| Scenario | Input | Expected |
|---|---|---|
| Success with stop reason | `ResultSuccess { stop_reason: Some("end_turn"), ... }` | Returns `Some("end_turn")` |
| Success without stop reason | `ResultSuccess { stop_reason: None, ... }` | Returns `None` |
| ErrorMaxTurns with stop reason | `ResultError { stop_reason: Some("max_turns"), ... }` | Returns `Some("max_turns")` |

---

### TC2: `RunRecord` field extraction in `spawn_agent_run`

**Location:** `crates/sdlc-server/src/routes/runs.rs`

| Scenario | Expected |
|---|---|
| Successful run with `stop_reason = "end_turn"` and `session_id = "s-abc"` | `RunRecord.session_id == Some("s-abc")`, `RunRecord.stop_reason == Some("end_turn")` |
| Error run (e.g. max turns) | `RunRecord.stop_reason` captures the error stop reason from the `ResultError` message |
| Stream ends without `Result` message | `session_id == None`, `stop_reason == None` |

---

### TC3: Backward compatibility — deserializing old `RunRecord` JSON

**Scenario:** A `.sdlc/.runs/*.json` file created before this feature (no `session_id` or `stop_reason` keys) must deserialize to `RunRecord` with both fields as `None`.

**Verification:** Parse a JSON string containing only the original `RunRecord` fields and assert both new fields are `None`.

---

### TC4: `RunFinished` SSE serialization

**Scenario:** The `RunFinished` variant serializes with `session_id` and `stop_reason` present when populated, and omits them (or null) when `None`.

---

### TC5: Compile and clippy

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Both must succeed with no errors or new warnings.

---

## Pass Criteria

- All unit tests (new + existing) pass
- `cargo clippy` clean
- No regressions in run history loading or SSE event emission
