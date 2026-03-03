# UAT Run — Project Changelog — what changed since you last looked
**Date:** 2026-03-03T03:32:00Z
**Agent:** claude-sonnet-4-6
**Verdict:** FAILED

---

## Hard Blocker

`localhost:7777` was unreachable at time of UAT run. The sdlc server was not running.
Per UAT rules, the server was NOT restarted. Scenarios 1–4 and Scenario 6 could not be exercised.

---

## Scenario 1: First Visit (New Developer)

- [ ] ~~Open the dashboard in a fresh browser (no `sdlc_last_visit_at` in localStorage)~~ _(✗ BLOCKED — server not running)_
- [ ] ~~EXPECT: A "Recent project activity" section is visible above main dashboard~~ _(✗ BLOCKED)_
- [ ] ~~EXPECT: Header reads "Recent project activity"~~ _(✗ BLOCKED)_
- [ ] ~~EXPECT: Events from last 7 days listed~~ _(✗ BLOCKED)_
- [ ] ~~EXPECT: Failed run events appear first~~ _(✗ BLOCKED)_
- [ ] ~~EXPECT: Clicking failed run event navigates to run detail~~ _(✗ BLOCKED)_

## Scenario 2: Returning User With Changes

- [ ] ~~Visit dashboard to set last_visit_at~~ _(✗ BLOCKED — server not running)_
- [ ] ~~Dismiss banner~~ _(✗ BLOCKED)_
- [ ] ~~Trigger a project event~~ _(✗ BLOCKED)_
- [ ] ~~EXPECT: Banner reappears with count~~ _(✗ BLOCKED)_
- [ ] ~~EXPECT: Merge event shown with 🚀~~ _(✗ BLOCKED)_

## Scenario 3: Dismiss Persists Across SPA Navigation

- [ ] ~~Open dashboard with unread events~~ _(✗ BLOCKED — server not running)_
- [ ] ~~Navigate to Runs page and back~~ _(✗ BLOCKED)_
- [ ] ~~EXPECT: Banner still showing — SPA navigation did NOT reset~~ _(✗ BLOCKED)_
- [ ] ~~Click Dismiss~~ _(✗ BLOCKED)_
- [ ] ~~EXPECT: Banner disappears and doesn't reappear after navigate~~ _(✗ BLOCKED)_

## Scenario 4: Failed Run Link Works

- [ ] ~~Expand banner with failed run event~~ _(✗ BLOCKED — server not running)_
- [ ] ~~Click "→" link on failed run event~~ _(✗ BLOCKED)_
- [ ] ~~EXPECT: Navigates to run detail showing failed run~~ _(✗ BLOCKED)_

## Scenario 5: CLI Changelog

- [x] `sdlc changelog` produces output _(2026-03-03T03:30:00Z)_
- [x] Output is formatted with icon + label + relative timestamp _(2026-03-03T03:30:00Z)_
- [x] Icon categories defined in code: ⚠️ RunFailed, 🚀 FeatureMerged, ✅ Approval, 🔄 PhaseAdvanced _(code-verified)_
- [ ] ~~`sdlc changelog` shows lifecycle events (merges, approvals) from changelog.yaml~~ _(✗ task changelog-cli#T5 — CLI reads .runs/ not changelog.yaml; direct merges/approvals are invisible)_
- [x] `sdlc changelog --since 3d --json` returns JSON output _(2026-03-03T03:30:00Z)_
- [ ] ~~JSON events have fields `id`, `kind`, `timestamp`, `label`, `slug`, `meta`~~ _(✗ task changelog-cli#T6 — actual fields: `id`, `category`, `icon`, `label`, `run_type`, `status`, `started_at`; `kind`→`category`, `timestamp`→`started_at`, missing `slug` and `meta`)_

## Scenario 6: API Endpoint

- [ ] ~~`curl http://localhost:PORT/api/changelog` returns `{ events: [...], total: N }`~~ _(✗ BLOCKED — server not running)_
- [ ] ~~`curl .../api/changelog?since=...&limit=5` filters correctly~~ _(✗ BLOCKED)_

## Scenario 7: Event Log Integrity

- [x] `changelog.yaml` exists and contains events (83 events, ev-0001 to ev-0083) _(2026-03-03T03:31:00Z)_
- [x] `feature_merged` events present with correct `id`, `kind`, `slug`, `timestamp` structure (10 events) _(2026-03-03T03:31:00Z)_
- [x] `sdlc merge` code path confirmed: calls `event_log::append_event(EventKind::FeatureMerged)` _(code-verified)_
- [x] `review_approved` events present with correct structure (7 events) _(2026-03-03T03:31:00Z)_
- [x] `sdlc artifact approve <slug> review` code path confirmed: calls `event_log::append_event(EventKind::ReviewApproved)` _(code-verified)_
- [ ] ~~`feature_merged` event includes `label` field~~ _(✗ task changelog-core#T13 — `ChangeEvent` struct has no `label` field; spec expects `slug`, `label`, `timestamp`)_

---

**Tasks created:** changelog-cli#T5, changelog-cli#T6, changelog-core#T13
**2/13 steps passed** (hard blocker prevents 10 steps; 3 steps are genuine failures)

## Failure Analysis

### Hard Blocker (environment)
`localhost:7777` unreachable — the sdlc server must be running for the full UAT.
**Fix:** Start the sdlc server, then re-run `/sdlc-milestone-uat v22-project-changelog`.

### Code Bugs (3 tasks created)
1. **changelog-cli#T5** — `sdlc changelog` reads `.sdlc/.runs/*.json` (agent runs) instead of `.sdlc/changelog.yaml`. Direct `sdlc merge` and `sdlc artifact approve` operations write to changelog.yaml but are invisible in CLI output.
2. **changelog-cli#T6** — `sdlc changelog --json` field names don't match spec: `category` vs `kind`, `started_at` vs `timestamp`, and `slug`/`meta` fields are absent.
3. **changelog-core#T13** — `ChangeEvent` struct is missing a `label` field; spec expects a human-readable label on `feature_merged` events.
