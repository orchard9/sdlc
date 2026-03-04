# Code Review: cargo-dist Cargo.toml Configuration

## Summary

Single-file config-only change: added `[workspace.metadata.dist]` to `Cargo.toml`.

## Diff

```toml
# cargo-dist configuration
+[workspace.metadata.dist]
+targets = [
+    "aarch64-apple-darwin",
+    "x86_64-apple-darwin",
+    "x86_64-unknown-linux-musl",
+    "aarch64-unknown-linux-musl",
+    "x86_64-pc-windows-msvc",
+]
+install-path = ["~/.local/bin"]
+
 # Release profile for distributed binaries
 [profile.dist]
```

## Findings

### Correctness

- Targets exactly match the matrix in `.github/workflows/release.yml` — no drift.
- `install-path = ["~/.local/bin"]` uses correct TOML array syntax for cargo-dist.
- `~/.local/bin` is the correct XDG user binary path for Linux installs without root.
- `[workspace.metadata.dist]` is the correct section name for cargo-dist workspace config (not `[package.metadata.dist]`).

### Build Verification

- `cargo metadata` parses the section correctly — confirmed `dist` key in workspace metadata.
- `SDLC_NO_NPM=1 cargo build --all` succeeds.
- `SDLC_NO_NPM=1 cargo test --all --lib` passes (143 tests, 0 failures).

### No Regressions

- No Rust source files changed.
- No CI workflow files changed.
- The existing `[profile.dist]` is untouched.
- Windows and macOS targets are included unchanged.

## Verdict

APPROVED. The change is minimal, correct, and well-scoped. No issues.
