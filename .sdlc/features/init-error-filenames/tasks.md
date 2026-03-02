# Tasks: Add File Paths to Init Error Messages

## T1: Add `.with_context()` to `.ai/` directory creation loop

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Replace bare `io::ensure_dir(&p)?;` with:
```rust
io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;
```

## T2: Add `.with_context()` to AI index file write

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Replace bare `io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())?;` with:
```rust
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())
    .with_context(|| format!("failed to write {}", index_path.display()))?;
```

## T3: Improve context string on `Config::save()` call

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Replace:
```rust
cfg.save(root).context("failed to write config.yaml")?;
```
With (using the already-computed `config_path` variable for a full path):
```rust
cfg.save(root).with_context(|| format!("failed to write {}", config_path.display()))?;
```

## T4: Improve context string on `State::save()` call

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Replace:
```rust
state.save(root).context("failed to write state.yaml")?;
```
With (using the already-computed `state_path` variable for a full path):
```rust
state.save(root).with_context(|| format!("failed to write {}", state_path.display()))?;
```

## T5: Verify build and tests pass

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```
