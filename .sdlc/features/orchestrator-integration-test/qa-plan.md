# QA Plan: Orchestrator Integration Test

## Gate Command

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Both must exit 0.

## Test Cases

### QA-1: Happy path test compiles and is registered

**Check**: `cargo test --all 2>&1 | grep orchestrator_two_actions`

**Pass**: Test name appears in output (either as "test ... ok" or "test ... ignored").

---

### QA-2: Happy path test passes when a JS runtime is available

**Check**: Run `SDLC_NO_NPM=1 cargo test orchestrator_two_actions_complete_in_one_tick -- --nocapture` in an environment where `bun`, `deno`, or `npx` is in PATH.

**Pass**: Output shows "test ... ok". Both actions have `Completed` status in the DB.

**Acceptable alternative**: Output shows "test ... ignored" if no runtime is detected — the skip guard is working correctly.

---

### QA-3: Startup recovery test passes unconditionally

**Check**: `SDLC_NO_NPM=1 cargo test orchestrator_startup_recovery_marks_stale_running_as_failed -- --nocapture`

**Pass**: Output shows "test ... ok". Return value was 1 and action status is `Failed { reason: "recovered from restart" }`.

**Fail**: Test is skipped, panics, or is missing from output.

---

### QA-4: Existing tests are unaffected

**Check**: `SDLC_NO_NPM=1 cargo test --all 2>&1 | grep -E "FAILED|error"` returns empty.

**Pass**: No existing test is broken by the refactor of `run_daemon` → `run_one_tick`.

---

### QA-5: Clippy clean

**Check**: `cargo clippy --all -- -D warnings` exits 0.

**Pass**: No warnings or errors from the new code.

---

### QA-6: `run_one_tick` is accessible from integration tests

**Check**: The test file imports `run_one_tick` without a compiler error.

**Pass**: `cargo build --all` succeeds.

---

## Risk Notes

- The happy-path test (QA-2) depends on a JS runtime being available. CI environments may not have one. The skip guard handles this gracefully — QA-3 must always pass regardless.
- Timing: the 300ms sleep + tool execution should complete well within any reasonable timeout. If a CI environment is very slow, the sleep can be increased.
