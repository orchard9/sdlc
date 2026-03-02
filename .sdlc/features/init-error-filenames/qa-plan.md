# QA Plan: Add File Paths to Init Error Messages

## Scope

Single file: `crates/sdlc-cli/src/cmd/init/mod.rs`. Four additive error-context changes. No new logic, no new dependencies.

## Test Strategy

### 1. Compile Check (automated)

```bash
SDLC_NO_NPM=1 cargo build -p sdlc-cli
```

- Ensures all `.with_context(|| ...)` closures compile correctly.
- Catches any lifetime/move issues with path captures in closures.

### 2. Full Test Suite (automated)

```bash
SDLC_NO_NPM=1 cargo test --all
```

- Existing integration tests must continue to pass.
- No test changes expected — error paths are not covered at integration level.

### 3. Clippy (automated)

```bash
cargo clippy --all -- -D warnings
```

- No new warnings must be introduced.

### 4. Error Message Content Verification (manual / code review)

For each of the four changes, verify via code review that:

| Change | Expected error text |
|---|---|
| `.ai/` dir creation | `failed to create /path/to/.ai/<dir>: <OS error>` |
| `.ai/` index write | `failed to write /path/to/.ai/index.md: <OS error>` |
| config.yaml save | `failed to write /path/to/.sdlc/config.yaml: <OS error>` |
| state.yaml save | `failed to write /path/to/.sdlc/state.yaml: <OS error>` |

### 5. Variable Scope Verification (code review)

Confirm that `config_path` (computed at line 57) and `state_path` (computed at line 67) are in scope at the points of their respective save calls (lines 60 and 70). Both variables are already computed before the conditionals, so no additional bindings are needed.

## Pass Criteria

- `cargo build` succeeds with no errors
- `cargo test --all` passes
- `cargo clippy` produces no new warnings
- Code review confirms correct path variables are captured in each closure
- No regressions in existing behavior
