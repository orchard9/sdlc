# Code Review: orchestrator-tick-cli

## Summary

The implementation is complete and correct. All five file-level tasks from the spec were delivered:
`orchestrate.rs`, `paths.orchestrator_db_path`, `cmd/mod.rs` registration, `main.rs` wiring, and `.gitignore` entry. The daemon loop, `add`, and `list` subcommands all match the spec precisely.

## Files Reviewed

| File | Status |
|------|--------|
| `crates/sdlc-cli/src/cmd/orchestrate.rs` | ✅ New — full implementation |
| `crates/sdlc-core/src/orchestrator/action.rs` | ✅ Existing — data model, no changes needed |
| `crates/sdlc-core/src/orchestrator/db.rs` | ✅ Existing — redb store with thorough tests |
| `crates/sdlc-core/src/paths.rs` | ✅ `orchestrator_db_path` added (line 254–256) |
| `crates/sdlc-cli/src/cmd/mod.rs` | ✅ `pub mod orchestrate` registered |
| `crates/sdlc-cli/src/main.rs` | ✅ `Orchestrate` variant + dispatch wired |
| `.gitignore` | ✅ `.sdlc/orchestrator.db` added |

## Spec Compliance

| Requirement | Status | Notes |
|-------------|--------|-------|
| Daemon tick loop with `startup_recovery(2*tick_rate)` | ✅ | `run_daemon` lines 83–113 |
| `range_due(now)` → Running → run_tool → Completed\|Failed | ✅ | `dispatch` function |
| Reschedule on Completed when `recurrence` set | ✅ | lines 157–169 |
| `sleep(max(0, tick_rate - elapsed))` | ✅ | lines 108–111 |
| `add` subcommand with `--at`, `--every` | ✅ | `run_add` + `parse_at` |
| `--at` accepts `now`, `now+Ns` (s/m/h), RFC3339 | ✅ | `parse_at` lines 219–244 |
| `--at` defaults to `now` | ✅ | clap `default_value = "now"` |
| `list` with optional `--status` filter | ✅ | `run_list` with status_tag filter |
| Table: ID \| LABEL \| TOOL \| STATUS \| UPDATED | ✅ | headers at line 268 |
| Tool not found → `Failed` (no daemon crash) | ✅ | lines 121–127 |
| DB error → propagate to main | ✅ | `?` propagation throughout |

## Observations

**Minor: daemon startup log uses default DB path, ignoring `--db` override**

In `run_daemon` (line 95–96), the log prints:
```rust
sdlc_core::paths::orchestrator_db_path(root).display()
```
This always computes the default `.sdlc/orchestrator.db` path. If the user passes `--db /custom/path.db`, the startup message misleads. The resolved `db_path` is only available in `run()`, not passed down to `run_daemon`. This is cosmetic-only — behavior is correct since the `ActionDb` was opened from the correct path. Low priority; can be addressed in a follow-on task.

**Acceptable: `unwrap_or(serde_json::Value::Null)` on tool stdout parse**

Line 141: tool stdout that isn't valid JSON silently becomes `Null`. The spec says `Completed { result: parse_json(stdout) }` without specifying failure handling. This is reasonable — a malformed-stdout result is better than an unexpected `Failed`.

**`rescheduled` fallback differs from spec**

Spec shows `unwrap_or_default()` (= 0-second `Duration`). Implementation uses `unwrap_or(chrono::Duration::seconds(60))`. The `chrono::Duration::from_std` conversion only fails for subnormal values that can't arise from `Duration::from_secs`, so this path is unreachable in practice. The fallback choice doesn't matter.

## Test Coverage

- `crates/sdlc-core/src/orchestrator/db.rs` has 6 unit tests covering: range_due correctness, non-Pending exclusion, key ordering, startup_recovery (stale + recent), and empty-DB edge cases.
- CLI-layer `orchestrate.rs` has no unit tests — consistent with the rest of the CLI cmd layer which relies on integration-level smoke tests.

## Verdict

**APPROVED.** Implementation matches the spec. The minor DB-path log issue is not a blocker. No regressions introduced.
