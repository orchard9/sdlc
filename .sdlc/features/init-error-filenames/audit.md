# Security Audit: Add File Paths to Init Error Messages

## Scope

Single file: `crates/sdlc-cli/src/cmd/init/mod.rs`

Four changes, all of the form: add `.with_context(|| format!("...", path.display()))` to error-path `?` operators.

## Security Analysis

### Information Disclosure

Error messages now include absolute filesystem paths (e.g., `/Users/xist/p4ws/project/.ai/patterns`). This is expected and safe for a local CLI tool that:

- Runs on the developer's own machine
- Operates in the current working directory the developer chose
- Outputs errors to stderr for the invoking user
- Has no network surface — no HTTP endpoints, no JSON API, no logs shipped remotely

The developer already knows the path they ran `sdlc init` in. Surfacing the path in an error is strictly more useful than hiding it.

### Path Traversal

No new path construction is introduced. All paths are derived from `root` (the CWD passed to `run()`) combined with compile-time constant path segments from `paths::*`. The `.display()` call is for error message formatting only — it does not affect what path is opened or written.

### Error Message Injection

`p.display()` and `index_path.display()` format a `Path` into a human-readable string. There is no user-supplied data in these paths beyond the CWD, which is already trusted input. No injection risk.

### Supply Chain / Dependencies

No new dependencies. `anyhow::Context` (already in scope via `use anyhow::Context`) is the only library involved.

## Findings

None. This change has no meaningful security surface.

## Verdict

APPROVED — no security concerns.
