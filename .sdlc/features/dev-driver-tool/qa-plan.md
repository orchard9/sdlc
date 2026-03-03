# QA Plan: dev-driver tool

## QA approach

Manual execution of dev-driver in a real sdlc project against each priority level.
All tests use `node .sdlc/tools/dev-driver/tool.ts --run <<< '{}'` directly.

---

## Test 1: --meta mode returns valid schema

```bash
node .sdlc/tools/dev-driver/tool.ts --meta | jq .
```

Pass: JSON matches ToolMeta shape with `name: "dev-driver"`, `input_schema`, `output_schema`

---

## Test 2: Idle when nothing actionable

Setup: All features in DRAFT/RELEASED, no lock file.

```bash
node .sdlc/tools/dev-driver/tool.ts --run <<< '{}'
```

Pass: `{ "ok": true, "data": { "action": "idle", "reason": "no actionable work found" } }`

---

## Test 3: Flight lock blocks dispatch

Setup: Create `.sdlc/.dev-driver.lock` with `started_at` 30 minutes ago.

Pass: `{ "ok": true, "data": { "action": "waiting", "lock_age_mins": 30 } }`

---

## Test 4: Stale lock is ignored

Setup: Create `.sdlc/.dev-driver.lock` with `started_at` 3 hours ago.

Pass: Tool proceeds past Level 1 (no `waiting` from lock)

---

## Test 5: Quality failing blocks advancement

Setup: Break a check in quality-check config (e.g. `script: exit 1`).

Pass: `{ "ok": true, "data": { "action": "quality_failing", "failed_checks": ["<name>"] } }`

---

## Test 6: Feature with active directive is advanced (Level 3)

Setup: Feature in IMPLEMENTATION phase with pending task.

Pass:
- `{ "ok": true, "data": { "action": "feature_advanced", "slug": "...", "phase": "implementation", "directive": "/sdlc-next ..." } }`
- `.sdlc/.dev-driver.lock` file is written
- Claude process is spawned (check with `ps aux | grep claude`)

---

## Test 7: skip:autonomous excludes feature

Setup: Feature in IMPLEMENTATION. Add task with title containing `skip:autonomous`.

Pass: Feature is NOT selected; tool returns `idle` (if no other actionable features)

---

## Test 8: Active sdlc run blocks dispatch

Setup: Simulate active run (if `sdlc run list --status running` returns a result).

Pass: `{ "ok": true, "data": { "action": "waiting", "reason": "agent run in progress" } }`

---

## Test 9: /sdlc-next is dispatched, not /sdlc-run

Inspect the spawn call in source code: confirm the command string is `/sdlc-next <slug>`
and there is no `/sdlc-run` call anywhere in Level 3.

Pass: Code review shows `--print /sdlc-next` with a comment explaining this is intentional

---

## Test 10: Output is visible in Actions page

Setup: Create an orchestrator action using dev-driver tool. Trigger it. Open Actions page.

Pass: The completed action shows the full `data` object (which feature, which directive, which phase)

---

## Test 11: README is complete

Check README covers: what it does, skip mechanism, one-step model, lock file, default action recipe, output reference.

Pass: All 7 sections present

---

## Pass criteria

All 11 tests pass. No TypeScript compilation errors. `--meta` output is valid JSON.
