# Spec: Add File Paths to Init Error Messages

## Problem

`sdlc init` emits `error: Permission denied (os error 13)` with no path context when directory creation or file writes fail. A user running `sdlc init` in a readonly or P4-managed directory gets an opaque error with no actionable information — they cannot tell which path failed or why.

This was identified as a first-run blocker by Xist (enterprise game dev) who spent 20 minutes debugging what turned out to be a permissions issue in a `.ai/` subdirectory.

## Root Cause

Two bare `?` calls in `crates/sdlc-cli/src/cmd/init/mod.rs` have no `.with_context()` to add the path to the error:

1. **Line 94** — `.ai/` directory creation loop:
   ```rust
   io::ensure_dir(&p)?;  // bare ? — no path in error
   ```

2. **Line 98** — AI index file write:
   ```rust
   io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())?;  // bare ?
   ```

Additionally, the existing context strings for `Config::save()` and `State::save()` (lines 60, 70) include only the filename, not the full path:
```rust
cfg.save(root).context("failed to write config.yaml")?;
state.save(root).context("failed to write state.yaml")?;
```

The codebase already has the correct pattern — see `write_guidance_md` which uses:
```rust
io::atomic_write(&path, GUIDANCE_MD_CONTENT.as_bytes())
    .with_context(|| format!("cannot write {}", path.display()))?;
```

## Solution

Apply consistent `.with_context()` to all bare `?` calls in the init path that touch the filesystem.

### Change 1: `.ai/` directory creation loop (line 94)

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Before:
```rust
io::ensure_dir(&p)?;
```

After:
```rust
io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;
```

### Change 2: AI index write (line 98)

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Before:
```rust
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())?;
```

After:
```rust
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())
    .with_context(|| format!("failed to write {}", index_path.display()))?;
```

### Change 3: Config and State save context strings (lines 60, 70)

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Before:
```rust
cfg.save(root).context("failed to write config.yaml")?;
// ...
state.save(root).context("failed to write state.yaml")?;
```

After (use the already-computed paths for full path in error):
```rust
cfg.save(root).with_context(|| format!("failed to write {}", config_path.display()))?;
// ...
state.save(root).with_context(|| format!("failed to write {}", state_path.display()))?;
```

## Expected User-Facing Result

Before:
```
error: Permission denied (os error 13)
```

After:
```
error: failed to create /Users/xist/p4ws/project/.ai/patterns: Permission denied (os error 13)
```

## Scope

- **File:** `crates/sdlc-cli/src/cmd/init/mod.rs` only
- **Changes:** 4 targeted edits — no new logic, no new dependencies, purely additive error context
- **Existing users:** No impact — error messages are additive improvements
- **Tests:** Existing tests still pass; no test changes needed (error paths are not tested at integration level)
