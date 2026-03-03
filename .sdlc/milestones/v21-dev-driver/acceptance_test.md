# Acceptance Test: v21-dev-driver

## Scenario: The self-advancing project

### Setup

A developer has `sdlc ui --run-actions` running. They have created one scheduled action:
- Label: `dev-driver`
- Tool: `dev-driver`
- Input: `{}`
- Recurrence: every 4 hours

Their project has 2 features in IMPLEMENTATION with pending directives.

---

### Test 1: One step, not full run

1. Trigger the dev-driver action manually (or wait for tick)
2. The action completes
3. **Verify:** Exactly ONE directive was executed across all features (not two, not a full feature run)
4. **Verify:** The action output shows: `{ action: "feature_advanced", slug: "...", phase: "...", directive: "..." }`
5. **Verify:** The directive executed was `/sdlc-next <slug>`, not `/sdlc-run <slug>`

### Test 2: Flight lock prevents double-dispatch

1. Trigger dev-driver while a previous dev-driver dispatch is still in flight (`.sdlc/.dev-driver.lock` exists and is < 2h old)
2. **Verify:** Action completes immediately with `{ action: "waiting", lock_age_mins: N }`
3. **Verify:** No new agent runs were spawned
4. **Verify:** Lock file was not overwritten

### Test 3: Quality check blocks advancement

1. Introduce a failing quality check (e.g., break a test)
2. Trigger dev-driver
3. **Verify:** Action returns `{ action: "quality_failing", failed_checks: ["..."] }`
4. **Verify:** No feature directive was dispatched
5. **Verify:** The check runs BEFORE looking at features

### Test 4: Idle when nothing to do

1. All features are in DRAFT (no active directives) and no wave is ready
2. Trigger dev-driver
3. **Verify:** Action returns `{ action: "idle", reason: "no actionable work found" }`
4. **Verify:** No agent runs spawned

### Test 5: Wave advancement when features are ready

1. All features in a milestone are in PLANNED or READY phase
2. No features with active directives
3. Trigger dev-driver
4. **Verify:** Action returns `{ action: "wave_started", milestone: "..." }`
5. **Verify:** An agent run is spawned for `/sdlc-run-wave <milestone>`

### Test 6: Actions page shows what happened

1. Run dev-driver (any outcome)
2. Open the Actions page
3. **Verify:** The completed run shows the action output content (not just pass/fail)
4. **Verify:** You can tell which feature was picked, which directive ran, why

### Test 7: sdlc init scaffolds dev-driver

1. Run `sdlc init` in a fresh directory (or `sdlc update` in an existing project)
2. **Verify:** `.sdlc/tools/dev-driver/tool.ts` exists
3. **Verify:** `.sdlc/tools/dev-driver/README.md` exists
4. **Verify:** `tools.md` documents the default action recipe

### Test 8: --run-actions is required to execute actions

1. Run `sdlc ui` (without `--run-actions`)
2. Create a scheduled action with next_tick_at in the past
3. Wait 2 minutes
4. **Verify:** The action was NOT executed (status remains `pending`)
5. Restart with `sdlc ui --run-actions`
6. Wait 2 minutes
7. **Verify:** The action executes

---

## User observable outcome

A developer can:
1. Run `sdlc ui --run-actions`
2. Create the dev-driver action in the UI
3. Walk away
4. Return and see exactly what the tool did: which feature advanced, which directive ran, what the output was
5. Trust that it advanced exactly ONE step per tick, not the entire feature lifecycle
