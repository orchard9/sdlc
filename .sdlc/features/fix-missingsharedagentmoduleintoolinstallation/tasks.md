# Tasks: Fix Missing `_shared/agent.ts` in Tool Installation

## T1: Add `TOOL_SHARED_AGENT_TS` constant to `templates.rs`

In `crates/sdlc-cli/src/cmd/init/templates.rs`, add:

```rust
pub const TOOL_SHARED_AGENT_TS: &str = include_str!("../../../../.sdlc/tools/_shared/agent.ts");
```

Place it alongside the other `TOOL_SHARED_*` constants.

## T2: Register `agent.ts` in `write_core_tools()`

In `crates/sdlc-cli/src/cmd/init/mod.rs`, inside `write_core_tools()`, add:

```rust
write_always(
    &tools_dir.join("_shared").join("agent.ts"),
    TOOL_SHARED_AGENT_TS,
)?;
```

And import the constant at the top of the relevant block or module.

## T3: Verify build passes

Run `SDLC_NO_NPM=1 cargo build --all` — should compile cleanly with the new `include_str!` path resolving to the existing file.

## T4: Run tests

Run `SDLC_NO_NPM=1 cargo test --all` — existing init tests should still pass; the new constant is additive.
