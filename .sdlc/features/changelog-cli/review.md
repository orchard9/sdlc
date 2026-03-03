# Review: changelog-cli

## Summary

The `sdlc changelog` command is implemented in `crates/sdlc-cli/src/cmd/changelog.rs` and registered in `main.rs` and `cmd/mod.rs`. All four tasks are complete. Tests pass, clippy is clean.

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-cli/src/cmd/changelog.rs` | New: 280-line module implementing all changelog logic |
| `crates/sdlc-cli/src/cmd/mod.rs` | Added `pub mod changelog;` |
| `crates/sdlc-cli/src/main.rs` | Added `Changelog { since, limit }` variant and dispatch arm |
| `crates/sdlc-cli/src/cmd/artifact.rs` | Fixed pre-existing unused-import clippy warning |

## Correctness

### Flag parsing
- `--since 7d` / `--since 1w` → relative duration — correct
- `--since 2026-03-01` → ISO midnight UTC — correct
- `--since last-merge` → finds newest merge run; falls back to 7d with stderr warning — correct
- Invalid input → `anyhow::bail!` with descriptive message — correct

### Classification
All six categories implemented and tested:
- `RunFailed` — `status == "failed"` or `status == "stopped" && error.is_some()`
- `FeatureMerged` — `run_type == "merge"` or `key.contains("merge")`
- `Approval` — `run_type.contains("approve")` or `key.contains("approve")`
- `PhaseAdvanced` — `run_type.contains("transition")` or `key.contains("transition")`
- `RunStopped` — `status == "stopped"` without error
- `AgentRun` — everything else

### Output
- Pretty-print: label padded to 45 chars, relative time on the right — verified working via smoke test
- JSON: matches spec schema with `since`, `limit`, `total`, `events` — correct
- Empty state: prints `No activity in the selected window.` — correct

## Code Quality

- No `unwrap()` in production code paths — all error-prone operations use `?` or `.ok()?` / `.unwrap_or`
- Skips unreadable/unparseable run files silently (consistent with `load_run_history` in sdlc-server)
- No new crate dependencies needed
- Tests cover 12 unit cases including all classification paths, all SinceSpec variants, and all relative time formats

## Build Verification

- `SDLC_NO_NPM=1 cargo test --all` → 114 sdlc-cli tests, 358 sdlc-core tests, 130 sdlc-server tests — all pass
- `cargo clippy --all -- -D warnings` → clean

## Smoke Test

```
$ sdlc changelog --since 7d
▶  run-wave: v22-project-changelog    10 min ago
▶  run-wave: v20-feedback-threads     10 min ago
⏹  run-wave: v15-agent-observability  1 hr ago
🚀  ...
```

Output is correct, formatted, and readable.

## Findings

No blocking issues. One minor observation:

- The label truncation relies on terminal whitespace padding — very long labels (>45 chars) will push the time column. This is acceptable for a developer tool where labels are generally short; no fix needed now. Tracked as low-priority polish if needed later.

## Verdict

APPROVED — implementation complete, correct, and all quality gates pass.
