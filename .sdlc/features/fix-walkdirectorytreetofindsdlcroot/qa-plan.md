# QA Plan: Fix resolve_root to Walk Directory Tree

## Automated Tests (unit)

All tests are in `crates/sdlc-cli/src/root.rs`.

| Test | Scenario | Expected |
|------|----------|----------|
| `explicit_root_wins` | `--root` flag provided | Returns the explicit path unchanged |
| `sdlc_dir_in_current_dir` | CWD contains `.sdlc/` | Returns CWD |
| `sdlc_dir_in_grandparent` | CWD is two levels below a dir with `.sdlc/` | Returns grandparent path |
| `no_sdlc_dir_falls_back_to_cwd` | No `.sdlc/` anywhere in ancestor tree | Returns CWD |

## Build Verification

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Both must pass with zero errors or warnings.

## Manual Smoke Test

```bash
cd /path/to/sdlc-project/crates/sdlc-cli
sdlc next --for some-feature --json
```

Should return a valid directive (not "project not initialized").
