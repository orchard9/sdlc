# Code Review: Orchestrator Integration Test

## Summary

This feature adds two integration tests for the orchestrator subsystem and extracts a `run_one_tick` helper to make the tick loop directly testable. All changes are minimal and well-scoped.

## Files Changed

- `crates/sdlc-cli/src/cmd/orchestrate.rs` — extracted `run_one_tick` from `run_daemon`
- `crates/sdlc-cli/src/lib.rs` — new file: thin lib target for integration test access
- `crates/sdlc-cli/Cargo.toml` — added `[lib]` target
- `crates/sdlc-cli/tests/integration.rs` — two new test functions

## Correctness

**`run_one_tick` extraction**: The extraction is clean. `run_daemon` now delegates the inner tick body to `run_one_tick`, preserving existing behavior. The only difference is that the timestamp snapshot (`let now = Utc::now()`) moved into `run_one_tick`, which is correct — each tick evaluates "now" at the time it runs.

**Happy-path test**: The test correctly:
- Creates the tool stub at the path `dispatch` will resolve (`.sdlc/tools/stub-tool/tool.ts`)
- Inserts actions with `now+100ms` and `now+200ms` scheduled times
- Sleeps 300ms to ensure both are past-due
- Calls `run_one_tick` once, which dispatches both actions synchronously
- Asserts `ActionStatus::Completed` for both

The tool stub `console.log(JSON.stringify({ok:true}));\n` is valid TypeScript that outputs `{"ok":true}` — matching what `dispatch` expects for a successful run.

**Startup recovery test**: The test correctly:
- Constructs an `Action` struct directly with `status=Running` and `updated_at=now-10min`
- Inserts it into the DB (bypassing `set_status` to preserve the backdated timestamp)
- Calls `startup_recovery(120s)` which finds actions with `updated_at < now - 120s`
- Asserts the return count is 1 and the action is `Failed { reason: "recovered..." }`

**Runtime guard**: The skip guard for missing JS runtime is correct — it prints a message and returns early rather than marking the test as `#[ignore]`, which is appropriate since the test infrastructure cannot know at compile time whether a runtime is available.

## Code Quality

- No `unwrap()` in library code (the new `run_one_tick` uses `?` throughout)
- Tests use `unwrap()` appropriately — panics on failure is the right behavior in tests
- `lib.rs` is minimal and clearly documented
- No new dependencies added
- Clippy clean

## Potential Issues

**None blocking.** Minor observations:

1. The `lib.rs` exposes the entire `cmd`, `output`, `root`, and `tools` modules. Future maintainers adding to these modules will have them automatically accessible from integration tests. This is intentional and low-risk.

2. The happy-path test is time-sensitive (300ms sleep). On extremely slow CI, this could be flaky. The 300ms margin is conservative enough for normal environments. If flakiness is ever observed, the sleep can be increased.

3. The tool stub is a single line without a shebang or TS module declaration. It works because the runtimes accept standalone JS/TS expressions. If this causes issues with strict Deno policies, a `// @ts-nocheck` comment can be added.

## Verdict

**APPROVED.** The implementation is correct, minimal, and well-tested. Gate passes: `SDLC_NO_NPM=1 cargo test --all` exits 0 with no failures. Clippy exits 0 with no warnings.
