# Spec: Fix Missing `_shared/agent.ts` in Tool Installation

## Problem

`agent.ts` was added to the `_shared/` library and `dev-driver/tool.ts` was updated to import from it, but `agent.ts` was never added to the `write_core_tools()` installation function in `crates/sdlc-cli/src/cmd/init/mod.rs`.

As a result, consumer projects that run `sdlc init` or `sdlc update` receive the updated `dev-driver/tool.ts` (which imports `../_shared/agent.ts`) but do **not** receive `_shared/agent.ts`. This causes a hard crash on every `GET /api/tools` request:

```
Error: Cannot find module '../_shared/agent.ts'
Require stack:
- /Users/xist/.../buildbot/.sdlc/tools/dev-driver/tool.ts
```

The crash makes the entire tools list endpoint (`GET /api/tools`) completely broken for affected users.

## Root Cause

In `crates/sdlc-cli/src/cmd/init/mod.rs`, `write_core_tools()` installs these `_shared/` files:
- `types.ts` (via `TOOL_SHARED_TYPES_TS`)
- `log.ts` (via `TOOL_SHARED_LOG_TS`)
- `config.ts` (via `TOOL_SHARED_CONFIG_TS`)
- `runtime.ts` (via `TOOL_SHARED_RUNTIME_TS`)

It does **not** install `agent.ts`, which was added to the `_shared/` directory later without updating the installation manifests. Also similarly, `sdlc.ts` is not installed by init either (though this may be intentional if no tool depends on it yet).

## Fix

1. Add a `TOOL_SHARED_AGENT_TS` constant to `crates/sdlc-cli/src/cmd/init/templates.rs` containing the full content of `.sdlc/tools/_shared/agent.ts`.
2. Register it in `write_core_tools()` alongside the other shared files — always overwritten on every `init`/`update` call.

## Acceptance Criteria

- After `sdlc init` or `sdlc update`, `.sdlc/tools/_shared/agent.ts` exists in consumer projects.
- `GET /api/tools` no longer crashes with `Cannot find module '../_shared/agent.ts'`.
- The `dev-driver` tool loads without error.
- Existing shared files (`types.ts`, `log.ts`, `config.ts`, `runtime.ts`) continue to be installed as before.

## Scope

- **Files changed**: `crates/sdlc-cli/src/cmd/init/templates.rs`, `crates/sdlc-cli/src/cmd/init/mod.rs`
- **No behavior change** to the state machine, server, or CLI commands — pure installation artifact fix.
- No migration needed — next `sdlc init`/`sdlc update` run by the consumer will drop the file.
