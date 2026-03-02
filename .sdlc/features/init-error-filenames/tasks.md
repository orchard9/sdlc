# Tasks: Add File Paths to Init Error Messages

## T1: Add `.with_context()` to `.ai/` directory creation loop

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Change the bare `?` on `io::ensure_dir(&p)` to include the path in the error message:

```rust
// Before
io::ensure_dir(&p)?;

// After
io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;
```

## T2: Add `.with_context()` to AI index file write

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Change the bare `?` on `io::write_if_missing(&index_path, ...)` to include the path:

```rust
// Before
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())?;

// After
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())
    .with_context(|| format!("failed to write {}", index_path.display()))?;
```

## T3: Improve Config save context string to include full path

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Replace static string context with dynamic path-based context:

```rust
// Before
cfg.save(root).context("failed to write config.yaml")?;

// After
cfg.save(root).with_context(|| format!("failed to write {}", config_path.display()))?;
```

## T4: Improve State save context string to include full path

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs`

Replace static string context with dynamic path-based context:

```rust
// Before
state.save(root).context("failed to write state.yaml")?;

// After
state.save(root).with_context(|| format!("failed to write {}", state_path.display()))?;
```

## T5: Verify `config_path` and `state_path` variables are in scope

Before T3 and T4, confirm that `config_path` and `state_path` are already computed variables in scope at the point of the save calls, or compute them if needed. If not in scope, derive them:

```rust
let config_path = root.join("config.yaml"); // already exists or add
let state_path = root.join("state.yaml");   // already exists or add
```
