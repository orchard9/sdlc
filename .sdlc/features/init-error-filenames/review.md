# Code Review: Add File Paths to Init Error Messages

## Summary

Four additive changes to `crates/sdlc-cli/src/cmd/init/mod.rs`. No new logic, no new dependencies, no behavioral changes on the happy path. All changes add path context to existing error propagation points.

## Changes Reviewed

### T1: `.ai/` directory creation loop (line 94)

```rust
// Before
io::ensure_dir(&p)?;

// After
io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;
```

- `p` is a local `PathBuf` computed from `root.join(dir)` — correctly captured by the closure.
- Consistent with the existing pattern in the `.sdlc/` directory creation loop at lines 51-54 (which already had `.with_context()`).
- No lifetime or borrow issues — `p` outlives the closure.

### T2: AI index file write (line 98)

```rust
// Before
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())?;

// After
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())
    .with_context(|| format!("failed to write {}", index_path.display()))?;
```

- `index_path` is a `PathBuf` in scope — captured correctly.
- Consistent with the pattern used in `write_guidance_md()` and `write_core_tools()`.

### T3: Config save context string (line 60)

```rust
// Before
cfg.save(root).context("failed to write config.yaml")?;

// After
cfg.save(root).with_context(|| format!("failed to write {}", config_path.display()))?;
```

- `config_path` is computed at line 57 via `paths::config_path(root)` and is in scope.
- The error now includes the full absolute path, making the message actionable.

### T4: State save context string (line 70)

```rust
// Before
state.save(root).context("failed to write state.yaml")?;

// After
state.save(root).with_context(|| format!("failed to write {}", state_path.display()))?;
```

- `state_path` is computed at line 67 via `paths::state_path(root)` and is in scope.
- Same pattern as T3.

## Quality Checks

- `cargo build -p sdlc-cli`: PASS
- `cargo test --all`: PASS (606 tests, 0 failures)
- `cargo clippy --all -- -D warnings`: PASS (0 warnings)

## Findings

No issues found. The changes are minimal, correct, and consistent with existing patterns in the codebase. The lazy `.with_context(|| ...)` closure form is appropriate — it avoids constructing the format string on the non-error path.

## Verdict

APPROVED — ready for audit.
