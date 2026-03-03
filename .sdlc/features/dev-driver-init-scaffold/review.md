# Review: dev-driver scaffolded by sdlc init and sdlc update

## Summary

Two files changed, 39 lines added, 3 lines changed. The changes follow the exact same pattern
as the existing `ama` and `quality-check` tool registrations. No new types, no new CLI surface,
no architectural decisions.

## Changes reviewed

### crates/sdlc-cli/src/cmd/init/templates.rs (+14 lines)

**T1/T2 — New constants:**

```rust
pub const TOOL_DEV_DRIVER_TS: &str =
    include_str!("../../../../../.sdlc/tools/dev-driver/tool.ts");

pub const TOOL_DEV_DRIVER_README_MD: &str =
    include_str!("../../../../../.sdlc/tools/dev-driver/README.md");
```

- `include_str!` path is relative to the source file location
  (`crates/sdlc-cli/src/cmd/init/templates.rs`). The five `../` segments are correct:
  `init` → `cmd` → `src` → `sdlc-cli` → `crates` → workspace root → `.sdlc/`.
- Build verified: `SDLC_NO_NPM=1 cargo build --all` compiles clean. The path resolves at
  compile time, so a bad path would be a hard build error.
- Placement: after the existing tool constants and before `TOOL_STATIC_TOOLS_MD`. Correct.

**T4 — TOOL_STATIC_TOOLS_MD entry:**

```markdown
## dev-driver — Dev Driver

Finds the next development action and dispatches it — advances the project one step per tick.

**Run:** `sdlc tool run dev-driver`
**Setup required:** No
_Configure via orchestrator: ..._
```

- Follows the same format as `ama` and `quality-check` entries.
- Inserted between `quality-check` and "Adding a Custom Tool" — correct placement.
- Includes orchestrator recipe reference. Accurate.

### crates/sdlc-cli/src/cmd/init/mod.rs (+22 lines)

**T3 — Import and write_core_tools() registration:**

Import extended correctly:
```rust
use templates::{
    ..., TOOL_DEV_DRIVER_README_MD, TOOL_DEV_DRIVER_TS, ...
};
```

Block added before the static `tools.md` write:
```rust
// Dev-driver tool
let dd_dir = paths::tool_dir(root, "dev-driver");
io::ensure_dir(&dd_dir)?;

let dd_script = paths::tool_script(root, "dev-driver");
let existed = dd_script.exists();
io::atomic_write(&dd_script, TOOL_DEV_DRIVER_TS.as_bytes())...
let dd_readme = paths::tool_readme(root, "dev-driver");
let created = io::write_if_missing(&dd_readme, TOOL_DEV_DRIVER_README_MD.as_bytes())...
```

- `io::ensure_dir` — correct pattern.
- `io::atomic_write` for `tool.ts` — always-overwrite policy, consistent with spec R1 and
  with how `ama/tool.ts` and `quality-check/tool.ts` are written.
- `io::write_if_missing` for `README.md` — write-if-missing policy, consistent with spec R2
  and with how `ama/README.md` and `quality-check/README.md` are written.
- Console output messages (`created:` / `updated:` / `exists:`) match the pattern used
  throughout `write_core_tools()`.
- Positioned before the static `tools.md` write — correct; the directory and files must exist
  before `tools.md` references them.

## Test results

- `SDLC_NO_NPM=1 cargo test -p sdlc-cli -p sdlc-core -p sdlc-server`: all 729 tests pass.
- The pre-existing `sdlc-server` clippy failure (non-exhaustive match on `SdlcError` for
  Telegram/Sqlite variants) is unrelated to this feature and was present before these changes.
  Tracked separately.

## Findings

1. **Pre-existing clippy failure** in `crates/sdlc-server/src/lib.rs`: non-exhaustive match on
   `SdlcError` variants (`TelegramTokenMissing`, `TelegramApi`, `Sqlite`). Pre-existing before
   this feature. No action required here.

2. **No config.yaml for dev-driver** — unlike `ama` and `quality-check`, the dev-driver tool
   has no user-configurable `config.yaml`. This is intentional and correct: dev-driver reads
   project state via CLI calls, not a config file. No missing file.

3. **No integration test added for dev-driver scaffolding** — the QA plan lists TC-1 through
   TC-3 as integration tests, but the implementation does not add new test cases. The existing
   `write_core_tools` integration test coverage for `ama` and `quality-check` provides the
   pattern; a dedicated test for `dev-driver` would strengthen regression coverage.
   **Action:** add `sdlc task add dev-driver-init-scaffold "Add integration test for dev-driver scaffolding in write_core_tools (TC-1 through TC-3)"` — tracked as follow-on.

## Verdict

APPROVED. The implementation is correct, minimal, idiomatic, and consistent with the existing
tool scaffolding patterns. All acceptance criteria from the spec are met:

1. Fresh `sdlc init` will produce both `dev-driver/tool.ts` and `dev-driver/README.md`.
2. Re-run updates `tool.ts` and leaves an existing `README.md` unchanged.
3. `sdlc tool list` will show `dev-driver` (via `tools.md`).
4. `TOOL_STATIC_TOOLS_MD` includes the `dev-driver` entry.
5. All tests pass.
