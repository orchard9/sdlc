# Tasks: dev-driver scaffolded by sdlc init and sdlc update

## T1 — Add dev-driver tool.ts content as Rust const in templates.rs

**File:** `crates/sdlc-cli/src/cmd/init/templates.rs`

Add `TOOL_DEV_DRIVER_TS` as the full verbatim content of `.sdlc/tools/dev-driver/tool.ts`,
embedded using `include_str!` macro pointing at the canonical source file.

The constant goes after the `TOOL_QUALITY_CHECK_README_MD` block, before `TOOL_STATIC_TOOLS_MD`.

## T2 — Add dev-driver README.md content as Rust const in templates.rs

**File:** `crates/sdlc-cli/src/cmd/init/templates.rs`

Add `TOOL_DEV_DRIVER_README_MD` as the full verbatim content of `.sdlc/tools/dev-driver/README.md`,
embedded using `include_str!` macro.

Goes immediately after `TOOL_DEV_DRIVER_TS`.

## T3 — Register dev-driver in write_core_tools() (init + update paths)

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

1. Add `TOOL_DEV_DRIVER_TS` and `TOOL_DEV_DRIVER_README_MD` to the `use templates::{...}` import.
2. In `write_core_tools()`, after the quality-check block and before `.gitignore` entries, add:
   - `io::ensure_dir` for `.sdlc/tools/dev-driver/`
   - `io::atomic_write` for `dev-driver/tool.ts` (always overwrite)
   - `io::write_if_missing` for `dev-driver/README.md`
   - `println!` for each file (`created:` / `updated:` / `exists:`)

## T4 — Document default action recipe in TOOL_STATIC_TOOLS_MD

**File:** `crates/sdlc-cli/src/cmd/init/templates.rs`

In `TOOL_STATIC_TOOLS_MD`, insert a `dev-driver` section between `quality-check` and
"Adding a Custom Tool":

```
## dev-driver — Dev Driver

Finds the next development action and dispatches it — advances the project one step per tick.

**Run:** `sdlc tool run dev-driver`
**Setup required:** No
_Configure via orchestrator: Label=dev-driver, Tool=dev-driver, Input={}, Recurrence=14400. See .sdlc/tools/dev-driver/README.md for full docs._
```
