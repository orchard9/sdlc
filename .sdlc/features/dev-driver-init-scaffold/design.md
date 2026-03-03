# Design: dev-driver scaffolded by sdlc init and sdlc update

## Overview

This is a pure Rust scaffolding change — no new types, no new CLI surface, no architectural
decisions beyond write-policy choices. The design mirrors the existing pattern used for `ama`
and `quality-check` verbatim.

## File write policies

| File | Policy | Rationale |
|---|---|---|
| `dev-driver/tool.ts` | Always overwrite | Managed content embedded in the binary. Same as `ama/tool.ts` and `quality-check/tool.ts`. |
| `dev-driver/README.md` | Write-if-missing | User-annotatable. Same policy as `ama/README.md` and `quality-check/README.md`. |

## Embedding strategy

### templates.rs

Two new string constants are added at the end of `templates.rs`, after the existing tool
constants:

```rust
pub const TOOL_DEV_DRIVER_TS: &str = include_str!("../../../../.sdlc/tools/dev-driver/tool.ts");
// ... or verbatim inline string
pub const TOOL_DEV_DRIVER_README_MD: &str = include_str!("...");
```

Using `include_str!` is preferred over a raw string literal because it avoids escaping issues and
automatically stays in sync if someone edits the source file. The canonical source remains at
`.sdlc/tools/dev-driver/`.

The import in `init/mod.rs` is extended:

```rust
use templates::{
    TOOL_AMA_CONFIG_YAML, TOOL_AMA_README_MD, TOOL_AMA_TS, TOOL_QUALITY_CHECK_CONFIG_YAML,
    TOOL_QUALITY_CHECK_README_MD, TOOL_QUALITY_CHECK_TS, TOOL_SHARED_CONFIG_TS, TOOL_SHARED_LOG_TS,
    TOOL_SHARED_RUNTIME_TS, TOOL_SHARED_TYPES_TS, TOOL_STATIC_TOOLS_MD,
    TOOL_DEV_DRIVER_TS, TOOL_DEV_DRIVER_README_MD,
};
```

### write_core_tools() additions

Appended after the `quality-check` block, before the `.gitignore` entries:

```rust
// Dev-driver tool
let dd_dir = paths::tool_dir(root, "dev-driver");
io::ensure_dir(&dd_dir)?;

// dev-driver/tool.ts — always overwrite (managed content)
let dd_script = paths::tool_script(root, "dev-driver");
let existed = dd_script.exists();
io::atomic_write(&dd_script, TOOL_DEV_DRIVER_TS.as_bytes())
    .context("failed to write dev-driver/tool.ts")?;
println!(
    "  {}: .sdlc/tools/dev-driver/tool.ts",
    if existed { "updated" } else { "created" }
);

// dev-driver/README.md — write-if-missing (user may annotate)
let dd_readme = paths::tool_readme(root, "dev-driver");
let created = io::write_if_missing(&dd_readme, TOOL_DEV_DRIVER_README_MD.as_bytes())
    .context("failed to write dev-driver/README.md")?;
println!(
    "  {}: .sdlc/tools/dev-driver/README.md",
    if created { "created" } else { "exists " }
);
```

### TOOL_STATIC_TOOLS_MD addition

The `dev-driver` entry is inserted between `quality-check` and "Adding a Custom Tool":

```markdown
## dev-driver — Dev Driver

Finds the next development action and dispatches it — advances the project one step per tick.

**Run:** `sdlc tool run dev-driver`
**Setup required:** No
_Edit recurrence in the sdlc UI or CLI. Default: every 4 hours. See README for lock/skip behaviour._
```

## Sequence diagram

```
sdlc init / sdlc update
    │
    └─► write_core_tools(root)
             │
             ├─► ensure_dir(".sdlc/tools/dev-driver/")
             ├─► atomic_write("dev-driver/tool.ts")    ← always overwrite
             └─► write_if_missing("dev-driver/README.md")
```

## No new paths or types needed

`paths::tool_dir`, `paths::tool_script`, and `paths::tool_readme` are already generic helpers that
accept a tool name string. No changes to `paths.rs` required.

## Testing approach

- Integration test: run `write_core_tools` against a `TempDir`, assert both files exist with
  correct content.
- Re-run test: pre-populate README with custom content, call `write_core_tools` again, assert
  README unchanged and `tool.ts` updated.
- Build test: `SDLC_NO_NPM=1 cargo test --all` must pass (the `include_str!` paths must resolve
  from the crate root at compile time).
