# Tasks: cargo-dist Cargo.toml Configuration

## T1: Add [workspace.metadata.dist] to Cargo.toml

Add the `[workspace.metadata.dist]` section to the root `Cargo.toml` with:
- `targets` listing all five release targets (matching the existing release.yml matrix)
- `install-path = ["~/.local/bin"]`

**File:** `Cargo.toml`

**Done when:** The section is present and the file passes `cargo build --all` without error.
