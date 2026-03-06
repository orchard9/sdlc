# RFP: Dev Run v2 — Autonomous Continuous Delivery

**Status:** Draft
**Author:** Jordan Washburn
**Date:** 2026-03-04

---

## 1. Problem Statement

Dev-driver v1 dispatches up to 4 parallel agents into worktrees and reports results.
It does not manage what happens _after_ agents complete: how work merges to main, how
conflicts are handled, how green builds become releases, or how failures re-enter the
system. These gaps create three problems:

1. **No merge-back protocol.** Completed worktrees sit waiting for manual intervention.
2. **No release pipeline.** There is no automated path from "agent finished" to "code is deployable."
3. **Failure recovery is ad-hoc.** Quality failures, UAT failures, and stale slots all
   require human diagnosis.

Dev-run v2 closes these gaps with a single, cyclic loop that chains autonomously —
agents work while you sleep, and every green merge is tagged as a release candidate.

---

## 2. Design Principles

### Code is net, not precious

Writing code is fast. If two agents conflict, discard the loser and retry on new main.
The retry produces better code than any merge — it accounts for the winner's changes.
No merge queues. No three-way reconciliation. No dead code from branch surgery.

### Fail = re-enter

Every failure maps to a re-entry command. Quality fail? `/sdlc-quality-fix`. Merge
conflict? Discard and re-queue. UAT fail? Failure pathway. The dev-run loop never
stops — it routes through the right command and continues.

### Main is the workspace, tags are the product

Agents commit freely to main. Main may be broken at any point — that's fine. Green
builds are tagged as release candidates (`rc-N`). The tag is the deployable artifact.
Release branches are only cut when production needs a hotfix while main has moved on.

### Worktrees provide isolation, not ceremony

Agents execute in worktrees for filesystem isolation during runs. This prevents
concurrent agents from stomping each other's uncommitted files. Worktrees are
disposable — created at dispatch, merged or discarded after completion.

---

## 3. Tiers

Three operational modes, selectable per run. Each tier uses the same phases but
differs in dispatch concurrency and merge-back strategy.

| Tier         | Dispatch       | Merge-back        | Conflicts      | Use case                              |
| ------------ | -------------- | ----------------- | -------------- | ------------------------------------- |
| T1: Observe  | Report only    | None              | N/A            | Dry run, debugging, dashboard preview |
| T2: Cautious | One at a time  | Always clean      | Impossible     | Active debugging, unfamiliar codebase |
| T3: Parallel | All concurrent | First-commit-wins | Discard losers | Default production mode               |

**T1** runs the full pipeline (quality gate, fetch slots) but stops before dispatch.
Returns `{ action: plan, items: [...] }` — what _would_ run. Zero side effects.

**T2** dispatches slot 1 into a worktree, waits for completion, merges to main, then
dispatches slot 2 against the updated main. Conflicts are impossible because work is
serialized. Slow but predictable.

**T3** dispatches all slots concurrently into separate worktrees. After all complete,
each attempts to merge to main. First to merge wins. If a later merge conflicts, the
worktree is discarded and the feature is automatically re-queued — it will be
re-dispatched on the next cycle against current main, which now includes the winner's
changes. The retry produces correct code because it works with full context.

---

## 4. The Loop

```
                          +========================+
                          |    dev-driver --run     |
                          |   tier: T1 | T2 | T3   |
                          +=============+==========-+
                                        |
  --- QUALITY GATE ------------------------+  all tiers
                                        |
                                 quality-check
                                        |
                               pass ----+---- fail/error
                                        |          |
                                        |     /sdlc-quality-fix
                                        |     next run re-checks ---x
                                        |
  --- FETCH --------------------------------+  all tiers
                                        |
                                 GET /api/state
                                 -> parallel_work[]
                                        |
                               1+ ------+---- empty
                                        |       |
                                        |    { idle } ---------------x
                                        |
  --- DISPATCH -----------------------------+  into worktrees
                                        |
                        T1 -------------+-------- report & exit
                                        |              |
                        T2: one at a time              |
                        T3: all concurrent         { action: plan }
                                        |              |
                                        |              x
                                        |
                                  slots complete
                                  (each in own worktree)
                                        |
  --- MERGE TO MAIN ------------------------+
                                        |
                           for each slot:
                                        |
                             +-----------+-----------+
                          clean                  conflict
                             |                       |
                        commit to main          discard worktree
                             |                  feature stays in
                             |                  parallel_work
                             |                  (retries next run
                             |                   against new main)
                             |                       |
                             v                       x
                                        |
  --- TAG GREEN ----------------------------+  after each clean merge
                                        |
                                 quality-check
                                        |
                               +--------+--------+
                             green              red
                               |                  |
                          tag rc-N           next run
                               |             quality-fixes
                            deployable            |
                               |                  x
                               |
  --- RE-ENTRY ---------------------+
                               |
                         { dispatched }
                               |
                         +=========================================+
                         | quality fail  -> /quality-fix           |
                         | UAT fail     -> failure pathway         |
                         | conflict     -> discard + retry         |
                         | max_turns    -> retry with more budget  |
                         | agent error  -> re-dispatch             |
                         | 409          -> skip (in-flight)        |
                         | stale loop   -> escalate                |
                         +=========================================+
                               |
                               +---> next cycle starts from top
```

---

## 5. Git Model

### Why not branches?

The traditional branch-per-feature model creates a merge queue. With multiple concurrent
agents, the queue becomes the bottleneck — agents are fast but merge ceremony is
serial.

When code is cheap to write, discard and rewrite produces cleaner output than merge.

### Main as workspace

All development happens on main. Agents commit directly. Main may be broken at any
given commit — that is acceptable. The quality-check tool identifies green commits.
Green commits are tagged as release candidates.

```
main: --A--B*--C--D*--E*--F--G*--H--I*--J--K*--
           |      |   |      |      |      |
          rc-1  rc-2 rc-3   rc-4   rc-5   rc-6

* = green (quality-check passed after worktree merge)
```

Every `rc-N` is a deployable snapshot. The deploy pipeline picks the latest tag.

### Conflict resolution

When a worktree merge to main conflicts (another slot merged first and touched the
same files), the conflicting worktree attempts to merge. If it is difficult or wasteful
to merge, then it is discarded. The feature's state has not advanced (the merge didn't land),
so `select_parallel_work()` still includes it.
Next cycle, the feature is re-dispatched into a fresh worktree against current main —
which now includes the winner's changes. The agent writes correct code because it
has full context.

Cost of a discard: one wasted agent run (~10 minutes, ~$0.50). Cost of a merge:
dead code, Frankenstein output, or a human debugging three-way diffs. Discard wins.

### When you need a release branch

Release branches are only necessary when production needs a hotfix while main has
moved past the deployed tag. In that case:

```
main: --A--B*--C--D--E--F--G--
                |
release/v42: ---+-- hotfix-1 -- hotfix-2
                |
                (deployed, needs a fix,
                 but main has moved on)
```

Cut a release branch from the deployed tag, apply hotfixes, deploy from the branch.
Forward-port hotfixes to main so they don't regress. This is rare — most of the time,
fixing forward on main and deploying the next green tag is faster.

### state.yaml

`.sdlc/state.yaml` is a derived projection of per-feature state. When agents in
different worktrees advance different features, state.yaml conflicts on every merge.

**Resolution:** Regenerate state.yaml after merge, never merge it. It should be
rebuilt from `.sdlc/features/*/` which are per-directory and never conflict across
features. Consider adding state.yaml to `.gitignore` and generating it on read.

---

## 6. Overnight Scenario

Agents are configured to run dev-driver on a 5-minute cron via the orchestrator.

```
22:00  Cycle 1: 4 slots dispatched (milestone A features)
22:45  Cycle 2: 3 merged, 1 conflict discarded. 3 tagged green.
                Re-dispatch conflict + next slot.
23:30  Cycle 3: Milestone A complete (all features merged).
                UAT slot dispatched.
00:15  Cycle 4: UAT passes. Milestone A tagged rc-12.
                Milestone B slots dispatched.
01:00  Cycle 5: 2 merged, quality-check fails on main.
                /sdlc-quality-fix runs.
01:30  Cycle 6: Quality fixed. Re-dispatch + new slots.
02:15  Cycle 7: Milestone B features merging...
...
06:00  Cycle 15: Milestone D in progress. rc-28 is latest green.

Owner wakes up:
  - 4 milestones completed overnight
  - 28 release candidates tagged
  - 2 features pending (conflict retries resolved themselves)
  - 1 escalation (secret request for milestone D)
  - Latest deployable: rc-28
```

Total agent time: ~8 hours of continuous work across 4 concurrent slots.
Effective wall-clock utilization: ~100% of the 8-hour sleep window.

---

## 7. Re-entry Table

Every failure type maps to exactly one re-entry action. The dev-driver never retries
raw — it routes through the appropriate command, which produces new state, and the
next cycle reads that state.

| Failure            | Re-entry            | Mechanism                                                    |
| ------------------ | ------------------- | ------------------------------------------------------------ |
| Quality gate fails | `/sdlc-quality-fix` | Agent diagnoses and fixes; next cycle re-checks              |
| Merge conflict     | Discard worktree    | Feature stays in `parallel_work`; retries on new main        |
| UAT fails (minor)  | Dispatch fix tasks  | Creates tasks from failures; next cycle works them           |
| UAT fails (major)  | Escalation          | Dashboard surfaces to human; slot skipped until resolved     |
| Agent max_turns    | Retry with +budget  | Dev-driver reads `stop_reason`, bumps maxTurns (40->80->120) |
| Agent error        | Re-dispatch         | Feature still in `parallel_work`; next cycle re-dispatches   |
| 409 conflict       | Skip                | Slot already in-flight; report and move on                   |
| Stale loop         | Escalation          | Feature dispatched N times without phase change; escalate    |

---

## 8. Required Changes

### 8.1 dev-driver tool.ts

| Change             | Description                                                       |
| ------------------ | ----------------------------------------------------------------- |
| Tier parameter     | Add `tier: 'T1' \| 'T2' \| 'T3'` to input schema (default: T3)    |
| Worktree lifecycle | Create worktree before dispatch, merge or discard after           |
| Merge-back step    | After slot completes: `git merge`, if conflict → discard worktree |
| Tag green          | After clean merge: run quality-check, if pass → `git tag rc-N`    |
| Stale detection    | Track dispatch count per feature; escalate after N retries        |

### 8.2 parallel_work.rs (Rust)

| Change                  | Description                                                       |
| ----------------------- | ----------------------------------------------------------------- |
| Conflict retry tracking | Count how many times a feature has been discarded due to conflict |
| Stale loop detection    | Flag features dispatched N+ times without phase advancement       |

### 8.3 state.yaml handling

| Change             | Description                                            |
| ------------------ | ------------------------------------------------------ |
| Regenerate on read | state.yaml rebuilt from per-feature dirs, never merged |
| .gitignore         | Consider excluding state.yaml from version control     |

### 8.4 New: release tagging

| Change                   | Description                                                    |
| ------------------------ | -------------------------------------------------------------- |
| `sdlc release tag`       | Tag current HEAD as rc-N if quality-check passes               |
| `sdlc release latest`    | Print latest rc tag (for deploy pipelines)                     |
| `sdlc release cut <tag>` | Create release branch from a specific rc tag (hotfix workflow) |

### 8.5 Escalation integration

| Change                  | Description                                                  |
| ----------------------- | ------------------------------------------------------------ |
| Skip escalated slots    | Dev-driver checks for open escalations before dispatching    |
| Stale loop → escalation | Auto-create escalation when dispatch count exceeds threshold |

---

## 9. What This Does NOT Cover

- **Deploy pipeline.** How rc tags reach production (canary, traffic shift, rollback)
  is infrastructure-specific. This RFP covers everything up to the deployable tag.
- **Multi-repo.** This design assumes a single repo. Multi-repo orchestration is a
  separate concern.
- **Human review gates.** This design is autonomous by default. Adding optional human
  review checkpoints (e.g., "hold rc tags for approval before deploy") is compatible
  but not specified here.

---

## 10. Options for Team Discussion

The design above is opinionated toward net-code-on-main with tag-based releases.
For teams that need stronger production gates, three alternative options were
considered:

### Option A: Net code on main + deploy gates (this RFP)

Agents commit to main. Green commits tagged as release candidates. Deploy pipeline
(canary, smoke tests, traffic shift) sits between tags and production. Rollback =
deploy previous rc tag + write forward on main.

**Best for:** Fast iteration, solo or small teams, high agent trust.

### Option B: Milestone branches

Each milestone gets one branch. All features commit freely within it (net code style).
Milestone merges to main only after UAT passes. Main is always UAT-verified.

**Best for:** Teams that want main to be always-stable. Adds merge ceremony at
milestone boundaries but keeps net code speed within milestones.

### Option C: Main + automated revert

Agents commit to main. CI runs on every commit. If CI fails, the commit is
automatically reverted. Main stays green by construction.

**Best for:** Teams with fast CI (<5 min). Requires robust test coverage — a bad
commit that passes CI is unrecoverable without manual revert.

### Option D: Branch-per-feature + agent PRs

Traditional model. Agents write code, open PRs, review PRs, merge PRs. Merge queue
handles ordering. Humans intervene on escalation only.

**Best for:** Teams with existing PR-based workflows, compliance requirements, or
multi-team coordination. Slowest option but most familiar.

---

## 11. Key Files

| File                                    | Role                                                   |
| --------------------------------------- | ------------------------------------------------------ |
| `.sdlc/tools/dev-driver/tool.ts`        | Dev-driver tool — quality gate + dispatch + merge-back |
| `crates/sdlc-core/src/parallel_work.rs` | Slot selection (Rust, single source of truth)          |
| `crates/sdlc-server/src/routes/runs.rs` | `spawn_agent_run` — all dispatches                     |
| `crates/sdlc-core/src/escalation.rs`    | Escalation records                                     |
| `docs/dev-run.md`                       | v1 design doc (superseded by this RFP)                 |
