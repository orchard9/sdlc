# Review: Standard Agents Scaffolding

## Files Changed

1. **`crates/sdlc-cli/src/cmd/init/mod.rs`** — Added `STANDARD_AGENT_KNOWLEDGE_LIBRARIAN` and `STANDARD_AGENT_CTO_CPO_LENS` constants, `write_standard_agents()` function, and integrated call in `run()` between step 6 and step 7.
2. **`crates/sdlc-cli/src/cmd/update.rs`** — Imported `write_standard_agents`, added call after `write_core_tools` and before `write_agents_md`.
3. **`crates/sdlc-cli/src/cmd/init/commands/sdlc_specialize.rs`** — Added standard agents note in all three template variants (Claude command, playbook, skill).

## Findings

### F1: Template content quality — PASS
- Knowledge-librarian template is a clean generic version without project-specific `{CATALOG_YAML}` or `{PROJECT_NAME}` substitution. Includes all core commands and protocol.
- CTO/CPO lens template matches existing agent content with added `Your Protocol` section for evaluating direction.

### F2: `write_if_missing` semantics — PASS
- Both agents use `io::write_if_missing` which returns `Ok(true)` only if the file was created, `Ok(false)` if it already exists. User edits are never overwritten.

### F3: Error handling — PASS
- All operations use `?` with `.with_context()`. No `unwrap()` in library code.

### F4: Init/update integration — PASS
- `run()` calls `write_standard_agents(root)?` at step 6.5 (between AGENTS.md and user scaffolding).
- `update.rs` imports and calls the same function.
- Both print section headers for user visibility.

### F5: Specialize template updated — PASS
- All three variants (Claude command, Gemini/OpenCode playbook, Agents skill) include the standard agents note.

### F6: Build/clippy — PASS
- `cargo build --all` succeeds.
- `cargo clippy --all -- -D warnings` clean (no new warnings).

### F7: Pre-existing test failure — NOTED
- Integration tests fail with `NotFoundError { path: ".../target/debug/sdlc" }` — binary is named `ponder`, not `sdlc`. This is a pre-existing issue unrelated to this feature.

## Verdict

All changes are correct, minimal, and follow existing patterns. Approved.
