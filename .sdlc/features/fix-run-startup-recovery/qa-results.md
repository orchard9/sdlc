# QA Results: Server Startup Marks Orphaned Runs Failed

## TC-1: Orphaned run gets `status = "failed"` on startup

**Result:** PASS

```
test state::tests::orphaned_runs_marked_failed_on_startup ... ok
```

The returned `Vec<RunRecord>` for the orphaned record has:
- `status == "failed"` ✓
- `error == Some("server restarted")` ✓
- `completed_at == Some(<ISO8601 timestamp>)` ✓

---

## TC-2: Orphaned run persisted to disk with `status = "failed"`

**Result:** PASS

The test asserts that the `.json` file on disk is updated to reflect
`status = "failed"` and `error = "server restarted"` — both verified by the
`orphaned_runs_marked_failed_on_startup` test.

---

## TC-3: Non-running records are unchanged

**Result:** PASS

The `"completed"` run in the test is verified to have:
- `status == "completed"` (unchanged) ✓
- `error == None` (unchanged) ✓

---

## TC-4: `cargo test --all` passes with `SDLC_NO_NPM=1`

**Result:** PASS (all test suites, 0 failures)

```
sdlc-core:        428 passed; 0 failed
sdlc-server lib:  148 passed; 0 failed (includes orphaned_runs_marked_failed_on_startup)
sdlc-server int:   45 passed; 0 failed
sdlc-cli:          52 passed; 0 failed
```

All unit and integration tests pass. The migration test
`migrate_v2_to_v3_backfills_missing_artifacts` was added as part of the
artifact backfill migration (schema version 2→3) that enables this feature
to complete its lifecycle.

---

## TC-5: `cargo clippy --all -- -D warnings` passes

**Result:** PASS

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 11s
```

Zero warnings, zero errors.

---

## Summary

| TC | Result |
|---|---|
| TC-1: Orphan status = "failed" in memory | PASS |
| TC-2: Orphan persisted to disk as "failed" | PASS |
| TC-3: Non-running records unchanged | PASS |
| TC-4: `cargo test --all` | PASS |
| TC-5: `cargo clippy` | PASS |

**QA Verdict: PASS** — all five test cases pass. Feature is ready for merge.
