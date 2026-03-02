# Audit: sdlc knowledge librarian run — maintenance pass + harvest hooks wired

## Scope

Security and correctness audit of all code changes introduced by this feature:
- `crates/sdlc-cli/src/cmd/knowledge.rs` — new `Run` and `Harvest` CLI variants, `run_harvest()` helper.
- Pre-existing code that this feature depends on: `librarian_harvest_workspace` in `sdlc-core`, investigate/ponder hooks, server endpoints.

---

## Finding 1: Subprocess injection via `--slug` argument (ACCEPTED — low risk, by design)

**Location**: `crates/sdlc-cli/src/cmd/investigate.rs` line 455, `crates/sdlc-cli/src/cmd/ponder.rs` line 487.

**Observation**: The auto-harvest hook passes the workspace `slug` directly as a CLI argument to a subprocess call:
```rust
std::process::Command::new("sdlc")
    .args(["knowledge", "librarian", "harvest", "--type", "investigation", "--slug", &slug])
    .status()
```
Slugs are passed as discrete `args()` elements (not shell-interpolated), so there is no shell injection risk. Each element is passed as a separate `argv` entry to the OS. This is the safe pattern.

**Verdict**: No issue. The `args([...])` API passes arguments as separate tokens, not through a shell.

---

## Finding 2: `--mode harvest` with missing `--type`/`--slug` returns a clear error (PASS)

**Location**: `run_librarian` in `knowledge.rs`, `Run { mode, r#type, slug }` arm.

**Observation**: When `mode == "harvest"` and either `--type` or `--slug` is absent, the handler returns `Err(anyhow!("--type is required..."))` / `Err(anyhow!("--slug is required..."))`. The process exits non-zero with a descriptive message. No panic, no unwrap.

**Verdict**: Correct error handling.

---

## Finding 3: `workspace_type` validation in `librarian_harvest_workspace` (PASS)

**Location**: `crates/sdlc-core/src/knowledge.rs`, `librarian_harvest_workspace`.

**Observation**: The function pattern-matches on `workspace_type` and returns `Err(SdlcError::KnowledgeNotFound(...))` for any value other than `"investigation"` or `"ponder"`. This prevents creation of knowledge entries for unsupported workspace types.

The server endpoint (`harvest_knowledge_workspace`) also independently validates `body.type` before calling the agent. Defense-in-depth is correct.

**Verdict**: No issue.

---

## Finding 4: `run_harvest` output shape matches spec (PASS)

**Location**: `run_harvest()` in `knowledge.rs`.

**Observation**: JSON output is `{ "type": workspace_type, "slug": workspace_slug, "created": bool, "entry_slug": result.slug }`. This matches the spec exactly (`{ "type", "slug", "created", "entry_slug" }`).

**Verdict**: No issue.

---

## Finding 5: No `unwrap()` in new code (PASS)

All error paths in `run_harvest()` and the `run_librarian` match arms use `?`, `ok_or_else`, and `anyhow::anyhow!`. No `unwrap()` or `expect()` calls introduced.

**Verdict**: Compliant with CLAUDE.md convention.

---

## Finding 6: Maintenance prompt content is instruction text, not executable code (PASS)

The `Run { mode: "maintain", .. }` arm prints a static string describing the six maintenance checks. This text is consumed by a human or agent that reads stdout — it is not evaluated or executed by Rust. No injection surface.

**Verdict**: No issue.

---

## Finding 7: Best-effort harvest hook does not leak errors to users (PASS)

Both the investigate and ponder hooks silently discard subprocess errors (`let _ = cmd.status()`). A failed harvest does not cause the `--status complete` update to fail. This is the correct behavior per spec — the harvest is best-effort.

**Verdict**: No issue.

---

## Summary

| Finding | Severity | Action |
|---|---|---|
| Subprocess arg safety | Informational | No action — `args([])` is safe |
| Missing flag error handling | Pass | Correct |
| workspace_type validation | Pass | Defense-in-depth present |
| JSON output shape | Pass | Matches spec |
| No unwrap | Pass | Compliant |
| Prompt is static text | Pass | No injection surface |
| Best-effort hook | Pass | Correct behavior |

No blocking findings. No deferred items. Feature is safe to merge.
