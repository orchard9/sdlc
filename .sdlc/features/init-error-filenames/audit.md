# Security Audit: Add File Paths to Init Error Messages

## Scope

Four additive `.with_context()` calls in `crates/sdlc-cli/src/cmd/init/mod.rs`. No new logic, no new I/O operations, no new dependencies.

## Security Analysis

### Information Disclosure via Error Messages

**Finding:** Error messages now include filesystem paths (e.g., `/Users/xist/p4ws/project/.ai/patterns`).

**Assessment:** Acceptable. This is a CLI tool running under the user's own account. The paths exposed are:
- The `.ai/` subdirectory paths the user explicitly asked `sdlc init` to create
- The `.sdlc/config.yaml` and `.sdlc/state.yaml` paths in the project root

The user already knows these paths — they are in the directory they invoked the command in. There is no cross-user boundary, no server context, and no log forwarding. The path information is printed to stderr only when an error occurs (the error propagates up through `anyhow` and is printed by the CLI's error handler), which is exactly the user who ran the command.

**Action:** Accept — no change needed.

### Path Traversal or Injection

**Finding:** Paths are constructed from `root` (the current working directory or explicitly supplied root) plus known static string suffixes (`.ai/patterns`, etc.) or via `paths::config_path(root)`.

**Assessment:** No user-controlled input enters the path construction for the four changed call sites. The `root` value is derived from the current directory or a CLI flag already present before this feature. No new attack surface introduced.

**Action:** Accept.

### Closure Capture Safety

**Finding:** The `.with_context(|| ...)` closures capture `PathBuf` values by reference-equivalent (the closures capture owned `PathBuf` or references in scope).

**Assessment:** All captured values (`p`, `index_path`, `config_path`, `state_path`) are owned `PathBuf` values in the enclosing scope that outlive the closure. No use-after-free or lifetime unsoundness possible. Rust's borrow checker enforces this at compile time (confirmed — `cargo build` passes).

**Action:** Accept.

## Findings Summary

| # | Finding | Severity | Action |
|---|---|---|---|
| 1 | Paths in error messages | INFO | Accept — CLI-only, same-user context |
| 2 | Path traversal surface | INFO | Accept — no new user input in path construction |
| 3 | Closure capture safety | INFO | Accept — verified by compiler |

## Verdict

No security concerns. The change is purely additive error context for a local CLI tool. All findings accepted with rationale documented above.
