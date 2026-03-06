# Code Review: Spike CLI — list, show, promote subcommands

## Summary

Reviewed `crates/sdlc-cli/src/cmd/spike.rs` and the registration changes in `mod.rs`
and `main.rs`. Implementation is complete and correct.

## Files Changed

- `crates/sdlc-cli/src/cmd/spike.rs` — new file (149 lines)
- `crates/sdlc-cli/src/cmd/mod.rs` — added `pub mod spike;`
- `crates/sdlc-cli/src/main.rs` — added import, Commands variant, and dispatch arm

## Findings

### APPROVED — Correctness

- `list`: calls `spikes::list(root)`, outputs table with SLUG | VERDICT | DATE | TITLE.
  Handles empty case, handles JSON mode. No issues.
- `show`: calls `spikes::load(root, slug)`, prints full findings.md. Verdict-specific hints
  are accurate per spec. Handles absent findings.md (empty string from core → "No findings.").
  JSON includes `findings_content`. No issues.
- `promote`: calls `spikes::promote_to_ponder(root, slug, as_slug_override)`, prints slug
  and next-step hint. JSON output shape correct. No issues.

### APPROVED — Patterns

- Uses `anyhow::Context` consistently for error messages — matches investigate.rs pattern.
- Uses `print_table` and `print_json` from `crate::output` — consistent.
- `SpikeSubcommand` enum with clap `Subcommand` derive — matches all other commands exactly.
- `pub fn run(root, subcmd, json)` dispatch pattern — matches investigate.rs exactly.
- No `unwrap()` in any code path — all errors propagated via `?`.

### APPROVED — Build quality

- `SDLC_NO_NPM=1 cargo build --all` — clean
- `cargo clippy --all -- -D warnings` — no warnings
- `SDLC_NO_NPM=1 cargo test --all` — 875 tests pass, 0 failures

### APPROVED — Edge cases

- Empty spikes dir: returns "No spikes." or `[]` — handled
- Missing findings.md: core returns empty string; `show` prints "No findings." — handled
- REJECT auto-filing: triggered by `spikes::list` in core — no CLI action needed
- `--as` slug override on promote — wired correctly via `as_slug.as_deref()`

## Verdict

APPROVED. No changes required. Ready to advance to audit.
