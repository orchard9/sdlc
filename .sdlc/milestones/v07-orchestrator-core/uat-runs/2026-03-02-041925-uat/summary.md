# UAT Run — Tick-rate orchestrator core — scheduled actions fire tools
**Date:** 2026-03-02T04:19:25Z
**Verdict:** Pass
**Tests:** 4/4 scenarios
**Tasks created:** none

## Results

Suite: v07-orchestrator-core acceptance test
Mode: CLI-based (no browser UI — orchestrator is a daemon/CLI feature)
Duration: ~60s total (includes live daemon run time)
Passed: 4 | Failed: 0 | Skipped: 0

## Scenario Detail

| Scenario | Description | Result |
|---|---|---|
| 1 | Schedule action fires within tick window | PASS |
| 2 | Recurring action reschedules after each completion | PASS |
| 3 | Restart recovery marks stale Running actions as Failed | PASS |
| 4 | Full integration test suite (`cargo test --all`) | PASS |

## Notes

- Scenario 1: `sdlc orchestrate add test-action --at now+2s` with `--tick-rate 5` daemon — action appeared `Completed` in `sdlc orchestrate list` within 8s.
- Scenario 2: `--every 10` recurring action fired 3 times in 28s with 3s ticks — each firing created a new `Completed` row and a new `Pending` row for the next run.
- Scenario 3: Verified via integration test `orchestrator_startup_recovery_marks_stale_running_as_failed` (sdlc-cli tests, 1 passed). The `startup_recovery()` method marks any action that has been in `Running` state longer than `2 × tick_rate` as `Failed` with reason containing "recovered".
- Scenario 4: `SDLC_NO_NPM=1 cargo test --all` — all tests pass across sdlc-core, sdlc-cli, sdlc-server.
