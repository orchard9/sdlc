# Spec: Orchestrator Integration Test

## Problem

The orchestrator tick loop (`sdlc orchestrate`) and its startup recovery logic exist in production code but have no integration-level test coverage. The unit tests in `db.rs` verify `ActionDb` in isolation, but no test exercises the full dispatch path: insert actions → run tick → tool executes → status transitions to `Completed`.

## Goal

Add integration tests in `crates/sdlc-cli/tests/integration.rs` that verify:

1. **Happy path**: Two scheduled actions fire and complete in one tick pass.
2. **Startup recovery**: A `Running` action with a stale `updated_at` is recovered to `Failed`.

## Acceptance Criteria

- Test 1 `orchestrator_two_actions_complete_in_one_tick`:
  - Creates a `TempDir` as the project root.
  - Writes a minimal `tool.ts` stub under `.sdlc/tools/stub-tool/tool.ts` that outputs `{"ok":true}` and exits 0 when invoked with `--run`.
  - Inserts two `Pending` actions scheduled at `now+100ms` and `now+200ms`.
  - Spawns the dispatch logic in a background thread (using `run_daemon` from `orchestrate.rs` made accessible, or by calling `range_due` + dispatch directly via a test-accessible wrapper).
  - Polls the DB for up to 600ms; asserts both actions reach `Completed` status.
  - Thread is dropped (daemon is finite via a single-tick helper) or joined with a timeout.

- Test 2 `orchestrator_startup_recovery_marks_stale_running_as_failed`:
  - Creates a `TempDir`.
  - Opens an `ActionDb` at the standard path.
  - Inserts a `Running` action with `updated_at` backdated to `now - 10min` (via direct DB manipulation matching the pattern in `db.rs` unit tests).
  - Calls `db.startup_recovery(Duration::from_secs(120))`.
  - Asserts the return value is 1.
  - Asserts the action status is `Failed { reason }` where `reason` contains "recovered".

- Gate: `SDLC_NO_NPM=1 cargo test --all` passes with no new failures.

## Design Notes

### Tool stub

The `dispatch` function (in `orchestrate.rs`) calls `sdlc_core::tool_runner::run_tool()`, which requires a TypeScript runtime (bun, deno, or node+tsx). The test stub at `.sdlc/tools/stub-tool/tool.ts` must be valid TypeScript that prints `{"ok":true}` when run with `--run`. A single-line stub works:

```ts
console.log(JSON.stringify({ok:true}));
```

If no TypeScript runtime is available in the test environment, Test 1 can be marked `#[ignore]` with a note, or skipped via `detect_runtime()`. Test 2 (startup recovery) has no tool execution and must always pass.

### Tick loop access

`run_daemon` loops forever and cannot be called directly from a test without a thread + timeout. The cleaner approach is to extract a `run_one_tick(root, db)` function from the daemon loop body in `orchestrate.rs` and make it `pub(crate)` or `pub`. Tests can then call `run_one_tick` once and check results without managing a blocking thread.

If extracting is undesirable, an alternative is: run the daemon in a background thread, poll the DB until both actions are Completed or 600ms elapses, then kill the thread by letting it be dropped (the OS will reap it).

The spec does not mandate the implementation approach — the implementing agent picks the cleanest option that keeps the public API stable.

### Crate structure

Integration tests live in `crates/sdlc-cli/tests/integration.rs`. They have access to the `sdlc` binary via `assert_cmd` and to `sdlc_core` as a dev-dependency. The `orchestrate.rs` dispatch logic lives in `sdlc-cli`, not `sdlc-core`, so tests that need to call dispatch functions should either invoke the `sdlc orchestrate` binary or use a helper exposed from `sdlc-cli`.

A simpler approach: test entirely at the `sdlc-core` level (using `ActionDb` directly) for startup recovery, and invoke the binary (`sdlc orchestrate add` + `sdlc orchestrate` with a timeout) for the tick test. The binary invocation approach is the safest integration test pattern.

## Out of Scope

- Testing webhook-triggered actions.
- Testing recurring action rescheduling.
- Persistent daemon lifecycle management.
