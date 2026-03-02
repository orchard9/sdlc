# Code Review: Add File Paths to Init Error Messages

## Summary

All four targeted edits were applied to `crates/sdlc-cli/src/cmd/init/mod.rs`. The changes are purely additive error-context improvements — no logic, no new dependencies, no behavioural changes.

## Changes Reviewed

### 1. `.sdlc/` directory creation loop (line 53)

**Already fixed in a prior pass** — `io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;` was present before this feature started. Confirmed intact.

### 2. `.ai/` directory creation loop (line 96)

Applied correctly:
```rust
io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;
```
The variable `p` is computed from `root.join(dir)` in the loop, so `p.display()` gives the full absolute path.

### 3. AI index write (lines 100–101)

Applied correctly:
```rust
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())
    .with_context(|| format!("failed to write {}", index_path.display()))?;
```
`index_path` is `root.join(paths::AI_LOOKUP_INDEX)` — full path.

### 4. Config::save() context (lines 60–61)

Applied correctly:
```rust
cfg.save(root)
    .with_context(|| format!("failed to write {}", config_path.display()))?;
```
`config_path` is `paths::config_path(root)` — already in scope at line 57.

### 5. State::save() context (lines 71–72)

Applied correctly:
```rust
state.save(root)
    .with_context(|| format!("failed to write {}", state_path.display()))?;
```
`state_path` is `paths::state_path(root)` — already in scope at line 67.

## Verification

- `SDLC_NO_NPM=1 cargo test --all` — all tests pass
- `cargo clippy --all -- -D warnings` — no warnings

## Pattern Consistency

All changes follow the established pattern used throughout the rest of the file (e.g., `write_guidance_md`, `write_user_command_scaffold`, `write_agents_md`). The codebase now has no bare `?` on `io::` filesystem calls in the critical init path.

## Verdict

APPROVED — clean, minimal, consistent. No issues found.
