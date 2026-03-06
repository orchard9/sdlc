# Review: Fix Missing `_shared/agent.ts` in Tool Installation

## Changes Made

### `crates/sdlc-cli/src/cmd/init/templates.rs`
- Added `TOOL_SHARED_AGENT_TS` constant using `include_str!("../../../../../.sdlc/tools/_shared/agent.ts")`
- Placed after `TOOL_SHARED_RUNTIME_TS`, before `TOOL_AMA_TS`
- Uses `include_str!` (not an inline raw string) to avoid duplicating 442 lines and to keep the source of truth in the actual `.sdlc/tools/_shared/agent.ts` file

### `crates/sdlc-cli/src/cmd/init/mod.rs`
- Added `TOOL_SHARED_AGENT_TS` to the `use templates::{ ... }` import block
- Added `("agent.ts", TOOL_SHARED_AGENT_TS)` entry to the `shared_files` slice in `write_core_tools()`

## Verification

- `SDLC_NO_NPM=1 cargo build --all` → clean, no errors
- `SDLC_NO_NPM=1 cargo test --all` → all tests pass
- `cargo clippy --all -- -D warnings` → no new warnings

## Findings

**No issues found.** The change is minimal, additive, and follows the exact pattern of the four existing shared file entries. The `include_str!` path resolves correctly at compile time (verified by successful build). No behavior changes to the state machine, server, CLI, or any other tool.

The fix directly addresses the reported crash: after `sdlc init` or `sdlc update`, `_shared/agent.ts` will exist in consumer projects and `dev-driver/tool.ts` will load without error.
