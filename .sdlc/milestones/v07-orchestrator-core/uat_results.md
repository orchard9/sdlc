# UAT Run — Tick-rate orchestrator core — scheduled actions fire tools
**Date:** 2026-03-02T04:19:25Z
**Agent:** claude-sonnet-4-6
**Verdict:** PASS

---

## Setup

- [x] `sdlc tool run quality-check` verified — returns `{"ok":true,"data":{"checks":[]}}` _(2026-03-02T04:16:00Z)_

## Scenario 1: Schedule and fire an action

- [x] `sdlc orchestrate add test-action --tool quality-check --input '{}' --at "now+2s"` — action scheduled with id `0a2a83e8`, status `Pending` _(2026-03-02T04:16:28Z)_
- [x] `sdlc orchestrate --tick-rate 5` daemon started and action fired within 8 seconds _(2026-03-02T04:16:35Z)_
- [x] `sdlc orchestrate list` shows `test-action` with status `Completed` _(2026-03-02T04:16:36Z)_

## Scenario 2: Recurring action

- [x] `sdlc orchestrate add recurring-check --tool quality-check --input '{}' --at "now+3s" --every 10` — action scheduled _(2026-03-02T04:16:53Z)_
- [x] Action fired at t+3s — new `Completed` row created _(2026-03-02T04:16:57Z)_
- [x] Action rescheduled in 10s — `"orchestrate: [recurring-check] rescheduled in 10s"` _(2026-03-02T04:16:57Z)_
- [x] Action fired again at t+13s — second `Completed` row created _(2026-03-02T04:17:09Z)_
- [x] Action fired again at t+23s — third `Completed` row with new `Pending` for next run _(2026-03-02T04:17:21Z)_
- [x] `sdlc orchestrate list` shows 3 `Completed` entries and 1 `Pending` _(2026-03-02T04:17:22Z)_

## Scenario 3: Restart recovery

- [x] Integration test `orchestrator_startup_recovery_marks_stale_running_as_failed` — 1 passed _(2026-03-02T04:19:00Z)_
- [x] `startup_recovery()` marks stale `Running` actions as `Failed` with reason containing "recovered" — verified _(2026-03-02T04:19:00Z)_
- [x] `startup_recovery_leaves_recent_running_alone` — recently-started Running actions are not incorrectly recovered _(2026-03-02T04:19:00Z)_

## Scenario 4: Integration test gate

- [x] `SDLC_NO_NPM=1 cargo test --all` — all tests pass _(2026-03-02T04:19:20Z)_
- [x] `orchestrator_two_actions_complete_in_one_tick` — PASS _(2026-03-02T04:19:20Z)_
- [x] `orchestrator_startup_recovery_marks_stale_running_as_failed` — PASS _(2026-03-02T04:19:20Z)_

---

**Tasks created:** none
**10/10 steps passed**
