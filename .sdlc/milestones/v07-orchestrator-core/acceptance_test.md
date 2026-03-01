# Acceptance Test: v07-orchestrator-core

## Setup

```bash
# Ensure a tool exists to test with
sdlc tool run quality-check  # verify it runs
```

## Scenario 1: Schedule and fire an action

```bash
# Add a scheduled action 10 seconds from now
sdlc orchestrate add test-action \
  --tool quality-check \
  --input '{}' \
  --at "now+10s"

# Start the orchestrator with a 15-second tick
sdlc orchestrate --tick-rate 15
```

**Expected:** Within 15 seconds, quality-check fires. The action appears in
`sdlc orchestrate list` with status `Completed`.

## Scenario 2: Recurring action

```bash
sdlc orchestrate add recurring-check \
  --tool quality-check \
  --input '{}' \
  --at "now+5s" \
  --every 30
```

**Expected:** Action fires at t+5s, then again at t+35s, t+65s. Each run
shows a new `Completed` entry with a fresh timestamp.

## Scenario 3: Restart recovery

```bash
# Kill the daemon mid-tick (simulate crash)
# Check status
sdlc orchestrate list
```

**Expected:** No action is `Running` after restart. Any action that was
`Running` when the daemon died appears as `Failed` with reason containing
"recovered". No action fires twice.

## Scenario 4: Integration test gate

```bash
SDLC_NO_NPM=1 cargo test --all
```

**Expected:** All tests pass. The orchestrator integration test explicitly
verifies two actions fire within one tick window.
