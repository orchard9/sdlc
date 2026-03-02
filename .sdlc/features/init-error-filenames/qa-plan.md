# QA Plan: Add File Paths to Init Error Messages

## Objective

Verify that all four error-context additions in `crates/sdlc-cli/src/cmd/init/mod.rs` produce human-readable path information in error messages, and that no regressions are introduced.

## Test Cases

### TC1: Build passes

```bash
SDLC_NO_NPM=1 cargo build --all
```

Expected: zero compilation errors.

### TC2: Clippy clean

```bash
cargo clippy --all -- -D warnings
```

Expected: zero warnings.

### TC3: All tests pass

```bash
SDLC_NO_NPM=1 cargo test --all
```

Expected: all tests green.

### TC4: Code review — context strings present

Manually inspect `crates/sdlc-cli/src/cmd/init/mod.rs` and confirm:

- `io::ensure_dir(&p)` call has `.with_context(|| format!("failed to create {}", p.display()))`
- `io::write_if_missing(&index_path, ...)` call has `.with_context(|| format!("failed to write {}", index_path.display()))`
- `cfg.save(root)` call has `.with_context(|| format!("failed to write {}", config_path.display()))`
- `state.save(root)` call has `.with_context(|| format!("failed to write {}", state_path.display()))`

### TC5: No bare `?` on filesystem calls in init path

Grep confirms no remaining bare `?` on `io::` calls in `init/mod.rs`:

```bash
grep -n 'io::\(ensure_dir\|write_if_missing\|atomic_write\)[^;]*?;' crates/sdlc-cli/src/cmd/init/mod.rs
```

Expected: no matches.

## Pass Criteria

All five test cases pass with no failures or regressions.
