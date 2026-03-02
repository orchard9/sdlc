# Security Audit: Orchestrator Integration Test

## Scope

This feature adds test code only:
- A `pub fn run_one_tick` extracted from the existing daemon loop (no new behavior)
- A `[lib]` target in `sdlc-cli` (exposes existing `pub` items to integration tests)
- Two integration test functions in `crates/sdlc-cli/tests/integration.rs`

## Security Surface

**Minimal.** This is a test-only change with no production path additions, no new network access, no new file operations in production paths, and no changes to authentication or authorization logic.

## Findings

### New `pub` exposure via `lib.rs`

The `[lib]` target makes `sdlc_cli::cmd::orchestrate::run_one_tick` (and all other modules) accessible as a library. This is only relevant for crate consumers that add `sdlc-cli` as a dependency, which is not the intended usage pattern. The `sdlc-cli` crate is not published as a library for external consumption — it is an internal workspace member.

**Risk**: Negligible. The exposed function calls `ActionDb::range_due` and `dispatch`, which operate on local file paths only. No network, no secrets.

### Tool stub execution in tests

The happy-path test writes a TypeScript file to a `TempDir` and executes it via the JS runtime. The file content is hardcoded as `console.log(JSON.stringify({ok:true}));\n` — no user input, no shell injection vectors, no environment variable leakage.

**Risk**: None. The path is derived from a `TempDir`, which is OS-provided and not accessible to other processes in normal operation.

### Timing assumptions

The 300ms sleep relies on system clock accuracy. No security implications.

## Verdict

**PASS.** No security concerns. This is a pure test addition that does not alter any production code path, security boundary, or data handling logic.
