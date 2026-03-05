# Tasks: Fix resolve_root to Walk Directory Tree

## Implementation

- [ ] **TASK-1**: Update `resolve_root` in `crates/sdlc-cli/src/root.rs` to walk ancestor directories looking for `.sdlc/`, falling back to CWD if none found.
- [ ] **TASK-2**: Update/replace the `falls_back_to_cwd` test and add new tests: `.sdlc/` in current dir, `.sdlc/` in grandparent dir, no `.sdlc/` found.
- [ ] **TASK-3**: Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings` to verify no regressions.
