# Design: Fix Missing `_shared/agent.ts` in Tool Installation

## Approach

This is a pure Rust constant + registration change. No new abstractions needed.

## Files to Change

### 1. `crates/sdlc-cli/src/cmd/init/templates.rs`

Add one new constant after the existing `TOOL_SHARED_RUNTIME_TS`:

```rust
pub const TOOL_SHARED_AGENT_TS: &str = include_str!("../../../../.sdlc/tools/_shared/agent.ts");
```

This embeds the current `agent.ts` source directly into the binary, matching the same pattern used for all other shared files.

### 2. `crates/sdlc-cli/src/cmd/init/mod.rs`

In `write_core_tools()`, add one `write_always()` call alongside the existing shared file writes:

```rust
write_always(
    &tools_dir.join("_shared").join("agent.ts"),
    TOOL_SHARED_AGENT_TS,
)?;
```

## Current State (before fix)

```
write_core_tools():
  _shared/types.ts    ✓ installed
  _shared/log.ts      ✓ installed
  _shared/config.ts   ✓ installed
  _shared/runtime.ts  ✓ installed
  _shared/agent.ts    ✗ MISSING
```

## After Fix

```
write_core_tools():
  _shared/types.ts    ✓ installed
  _shared/log.ts      ✓ installed
  _shared/config.ts   ✓ installed
  _shared/runtime.ts  ✓ installed
  _shared/agent.ts    ✓ installed  ← NEW
```

## No Migration Needed

On next `sdlc init` or `sdlc update` by a consumer project, `agent.ts` is dropped into `_shared/`. No data format changes, no backward-compat concerns.
