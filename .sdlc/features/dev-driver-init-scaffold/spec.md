# Spec: dev-driver scaffolded by sdlc init and sdlc update

## Problem

The `dev-driver` tool was built as a custom TypeScript tool under `.sdlc/tools/dev-driver/` and
works correctly in this project. However, it is not a _stock_ tool — it does not ship with new
projects created via `sdlc init` or refreshed via `sdlc update`. Every project that wants
autonomous development advancement must hand-scaffold the tool from scratch.

The `ama` and `quality-check` tools are already part of the core tool suite written by
`write_core_tools()` in `crates/sdlc-cli/src/cmd/init/mod.rs`. The dev-driver tool should join
them so that any project can get autonomous step-by-step advancement simply by running
`sdlc init` or `sdlc update`.

## Goal

Make `dev-driver` a stock sdlc tool that is automatically scaffolded into `.sdlc/tools/dev-driver/`
whenever `sdlc init` or `sdlc update` is run, just like `ama` and `quality-check`.

## Requirements

### R1 — Rust const for tool.ts

The contents of the current `.sdlc/tools/dev-driver/tool.ts` must be embedded as a Rust string
constant `TOOL_DEV_DRIVER_TS` in `crates/sdlc-cli/src/cmd/init/templates.rs`.

Write policy: **always overwrite** — `tool.ts` is managed content that ships with the binary.
Users should not edit it; they extend it by contributing upstream.

### R2 — Rust const for README.md

The contents of the current `.sdlc/tools/dev-driver/README.md` must be embedded as a Rust string
constant `TOOL_DEV_DRIVER_README_MD` in `crates/sdlc-cli/src/cmd/init/templates.rs`.

Write policy: **write-if-missing** — the README describes the default action recipe and users may
want to annotate it with project-specific notes. Matches the behaviour of `ama/README.md`.

### R3 — Registration in write_core_tools()

`write_core_tools()` in `crates/sdlc-cli/src/cmd/init/mod.rs` must:
1. Import `TOOL_DEV_DRIVER_TS` and `TOOL_DEV_DRIVER_README_MD` from `templates`.
2. Call `io::ensure_dir` for `.sdlc/tools/dev-driver/`.
3. Always overwrite `dev-driver/tool.ts` from `TOOL_DEV_DRIVER_TS`.
4. Call `io::write_if_missing` for `dev-driver/README.md` from `TOOL_DEV_DRIVER_README_MD`.
5. Print `created:` / `updated:` / `exists:` messages consistent with existing tool writes.

### R4 — Static tools.md updated

`TOOL_STATIC_TOOLS_MD` in `templates.rs` must include an entry for `dev-driver` describing:
- Purpose: finds and dispatches the next development action
- Run command: `sdlc tool run dev-driver`
- Setup: not required

This entry appears between `quality-check` and the "Adding a Custom Tool" footer section.

### R5 — Idempotent on re-run

Running `sdlc init` or `sdlc update` multiple times must not corrupt or double-write the
dev-driver files. Specifically:
- `tool.ts` updates silently on re-run (overwrite = always).
- `README.md` does not overwrite user edits on re-run (write-if-missing).

## Out of Scope

- No new CLI subcommands are required.
- No changes to the orchestrator configuration or action recipes.
- No changes to the dev-driver tool logic itself (tool.ts content is frozen at current state).
- No changes to how `sdlc tool sync` works — it regenerates `tools.md` from live metadata; this
  feature only populates the static bootstrap manifest.

## Acceptance Criteria

1. `sdlc init` in a fresh directory produces `.sdlc/tools/dev-driver/tool.ts` and
   `.sdlc/tools/dev-driver/README.md` matching the canonical content.
2. `sdlc update` in an existing project updates `tool.ts` and leaves an already-present `README.md`
   unchanged.
3. `sdlc tool list` shows `dev-driver` in the output after init.
4. The static `tools.md` includes a `dev-driver` entry.
5. `SDLC_NO_NPM=1 cargo test --all` passes without new failures.
