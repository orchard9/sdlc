# QA Plan: cargo-dist Cargo.toml Configuration

## Scope

Verify that the `[workspace.metadata.dist]` section added to `Cargo.toml` is syntactically correct and does not break the build.

## Checks

### 1. Cargo parse check
```bash
cargo metadata --no-deps --manifest-path Cargo.toml
```
Must succeed without error. Verifies the TOML is valid and cargo can read the workspace metadata.

### 2. Build check
```bash
SDLC_NO_NPM=1 cargo build --all
```
Must complete successfully. Confirms the new metadata section does not interfere with the build.

### 3. Test suite
```bash
SDLC_NO_NPM=1 cargo test --all
```
All tests must pass.

### 4. Clippy
```bash
cargo clippy --all -- -D warnings
```
No warnings.

### 5. Visual inspection
Confirm `Cargo.toml` contains:
- `[workspace.metadata.dist]` section
- `targets` list with five entries matching the release workflow matrix
- `install-path = ["~/.local/bin"]`
