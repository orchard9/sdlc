# Dev Driver — Priority Waterfall

The dev-driver tool applies these rules in order. First match wins. One action per invocation.

## Level 1: Quality Failing

**Condition:** `quality-check` returns `failed > 0`  
**Action:** Write quality check report to `.sdlc/dev-driver-last-quality.json`, exit with `{ action: "quality_failing", failed_checks: [...] }`  
**Rationale:** Broken quality gates block everything. Report before advancing.

## Level 2: Features in Flight — Wait

**Condition:** `.sdlc/.dev-driver.lock` exists and is < 2 hours old  
**Action:** Exit with `{ action: "waiting", reason: "previous run in flight", lock_age_mins: N }`  
**Rationale:** Prevent concurrent agent runs on the same codebase.

## Level 3: Features with Active Directives

**Condition:** `sdlc state` shows features in IMPLEMENTATION / REVIEW / AUDIT / QA with pending directives  
**Action:** Pick highest-priority feature (IMPLEMENTATION > REVIEW > AUDIT, then by last-active), run `claude --print "/sdlc-next <slug>"`  
**Output:** `{ action: "feature_advanced", slug, directive_type, ... }`

## Level 4: Wave Ready to Start

**Condition:** `sdlc project prepare` shows a milestone with all features in PLANNED/READY state, no wave in flight  
**Action:** Run `claude --print "/sdlc-run-wave <milestone>"`  
**Output:** `{ action: "wave_started", milestone }`

## Level 5: Idle

**Condition:** None of the above match  
**Action:** `{ action: "idle", reason: "no actionable work found" }`  
**Note:** This is a signal worth surfacing in the UI — consistent idle might mean stalled project or missing features.

---

## Exclusions (v1)

The following are explicitly out of scope for v1:

- Feature suggestion / ideation (this is product work, not dev automation)
- Full codebase audit (too broad, too expensive per tick)
- Dependency unblocking (requires human judgment)
- Phase transitions without existing directives

These can be added as higher/lower priority levels in future versions.
