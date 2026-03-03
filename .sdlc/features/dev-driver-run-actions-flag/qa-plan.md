# QA Plan: sdlc ui --run-actions

## Scope

This QA plan covers the rename and inversion of `--no-orchestrate` → `--run-actions` in the `sdlc ui` command. Coverage is primarily behavioral (CLI flag semantics) and build-level (compilation, clippy, tests).

---

## Test Cases

### TC-1: Build succeeds

**Steps:**
1. Apply all changes (T1–T4).
2. Run `cargo build --all`.

**Pass criteria:** Zero compilation errors. Zero new warnings.

---

### TC-2: All existing tests pass

**Steps:**
1. Run `SDLC_NO_NPM=1 cargo test --all`.

**Pass criteria:** All test cases pass. No regressions.

---

### TC-3: Clippy passes

**Steps:**
1. Run `cargo clippy --all -- -D warnings`.

**Pass criteria:** Zero warnings (warnings-as-errors mode). No dead code, unused variable, or type mismatch warnings related to the rename.

---

### TC-4: `sdlc ui --help` shows `--run-actions`, not `--no-orchestrate`

**Steps:**
1. Build the binary.
2. Run `./target/debug/sdlc ui --help`.
3. Run `./target/debug/sdlc ui start --help`.

**Pass criteria:**
- `--run-actions` appears in both help outputs with description "Start the orchestrator daemon and execute scheduled actions" (or similar).
- `--no-orchestrate` does NOT appear in either help output.

---

### TC-5: `sdlc ui --no-orchestrate` produces a clap error

**Steps:**
1. Run `./target/debug/sdlc ui --no-orchestrate`.

**Pass criteria:**
- Process exits with a non-zero status code.
- Error message indicates unrecognized argument (`--no-orchestrate`).

---

### TC-6: `sdlc ui` (no flags) does not start orchestrator thread

**Steps:**
1. Run `sdlc ui` in a test project.
2. Observe process threads (via `ps -T` or similar).
3. Wait 2+ orchestrator tick intervals.

**Pass criteria:**
- No thread named `sdlc-orchestrator` is visible in the process.
- No scheduled actions are executed during the wait.

Note: This can be verified structurally by code review (the `if run_actions` guard) since the orchestrator side-effects require a running server; CI verifies via TC-1 and TC-2.

---

### TC-7: `sdlc ui --run-actions` starts orchestrator thread

**Steps:**
1. Run `sdlc ui --run-actions` in a test project.
2. Observe logs for orchestrator daemon startup message.

**Pass criteria:**
- The orchestrator daemon starts (log line emitted or thread spawned).
- No error returned from the spawn call.

Note: This is structurally verified by the `if run_actions` guard and build verification. Manual smoke test validates in dev.

---

### TC-8: `DEVELOPER.md` has no mention of `--no-orchestrate`

**Steps:**
1. Run `grep -n "no-orchestrate" DEVELOPER.md`.

**Pass criteria:** Zero matches.

---

## Pass/Fail Summary Table

| TC | Description | Method |
|---|---|---|
| TC-1 | Build succeeds | `cargo build --all` |
| TC-2 | Tests pass | `SDLC_NO_NPM=1 cargo test --all` |
| TC-3 | Clippy passes | `cargo clippy --all -- -D warnings` |
| TC-4 | Help shows `--run-actions` | CLI smoke test |
| TC-5 | `--no-orchestrate` is rejected | CLI smoke test |
| TC-6 | No orchestrator without flag | Code review / structural |
| TC-7 | Orchestrator starts with flag | Code review / structural + smoke |
| TC-8 | DEVELOPER.md updated | grep |

---

## Exit Criteria

All 8 test cases must pass for QA to be approved. Any failure blocks merge.
