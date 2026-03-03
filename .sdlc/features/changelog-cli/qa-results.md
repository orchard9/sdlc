# QA Results: changelog-cli

## Outcome: PASS

All test cases from the QA plan pass. Build and clippy are clean.

## Test Case Results

| TC | Description | Result |
|---|---|---|
| TC-1 | Default output (no flags) | PASS |
| TC-2 | `--since 1d` filters old events | PASS (unit test + manual) |
| TC-3 | `--since 2099-01-01` → empty state message | PASS |
| TC-4 | `--limit 2` → exactly 2 lines | PASS |
| TC-5 | `--json` → valid JSON with correct schema | PASS |
| TC-6 | `--since last-merge` with no merges → fallback + stderr warning | PASS |
| TC-7 | `--since last-merge` with existing merge | PASS (manual) |
| TC-8 | Empty `.sdlc/.runs/` → empty state message | PASS (unit test) |
| TC-9 | Invalid `--since badvalue` → exit 1 + error message | PASS |
| TC-10 | Icon classification by run_type/status | PASS (unit tests) |

## Unit Tests

All 12 unit tests pass:
- `test_classify_run_failed`
- `test_classify_run_stopped_with_error`
- `test_classify_merge`
- `test_classify_approval_by_run_type`
- `test_classify_approval_by_key`
- `test_classify_agent_run`
- `test_classify_run_stopped_no_error`
- `test_parse_since_relative_days`
- `test_parse_since_relative_weeks`
- `test_parse_since_iso`
- `test_parse_since_last_merge`
- `test_parse_since_invalid`
- `test_relative_time_format_seconds`
- `test_relative_time_format_minutes`
- `test_relative_time_format_hours`
- `test_relative_time_format_days`

## Build Checks

```
SDLC_NO_NPM=1 cargo test --all   → 114 sdlc-cli, 358 sdlc-core, 130 sdlc-server — all pass
cargo clippy --all -- -D warnings → clean
```

## Smoke Test Output

```
$ sdlc changelog --since 7d | head -5
▶  run-wave: v22-project-changelog    12 min ago
▶  run-wave: v20-feedback-threads     12 min ago
▶  run-wave: ponder-ux-polish         12 min ago
▶  run-wave: finding-closure-protocol 12 min ago
▶  run-wave: v16-orchestrator-ui      12 min ago

$ sdlc changelog --since 2099-01-01
No activity in the selected window.

$ sdlc changelog --since badvalue
error: Invalid --since value: 'badvalue'. Expected ISO date (2026-03-01), relative (7d, 1w), or last-merge
[exit 1]

$ sdlc changelog --json | python3 -c "import sys,json; d=json.load(sys.stdin); print(list(d.keys()))"
['since', 'limit', 'total', 'events']
```

## Verdict

PASS — all acceptance criteria met, no known defects.
