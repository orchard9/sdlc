# Plan: Project Changelog — What Changed Since You Last Looked

## Milestone: v22-project-changelog
**Title:** Project Changelog — what changed since you last looked
**Vision:** A developer coming back to the dashboard after days away can immediately see what happened — failed runs, merged features, phase transitions — without hunting through the activity feed. The same event log powers CLI queries and the Telegram notification bot.

## Why one milestone, not multiple
The event log is the backbone. Everything else (API, CLI, banner) is a thin consumer of it. They ship together as one deliverable — partial delivery (e.g., just the event log without the UI) has no user-observable value.

## Wave 1 (foundation)

### Feature: changelog-core
**Title:** Changelog event log core — append-only persistent event source
**Why first:** All other features depend on this. No parallelism possible.

Tasks:
1. Create `crates/sdlc-core/src/event_log.rs` — `ChangeEvent` struct, `EventKind` enum, `append_event()` + `query_events()` functions; atomic write via `io.rs`
2. Add changelog file path constant to project root helpers (`.sdlc/changelog.yaml`)
3. Emit `feature_merged` event in `crates/sdlc-cli/src/cmd/merge.rs` after successful merge
4. Emit `review_approved`, `audit_approved`, `qa_approved` events in `crates/sdlc-cli/src/cmd/artifact.rs` on approve for those artifact types
5. Emit `run_failed` event in `crates/sdlc-server/src/routes/runs.rs` when run completes with failed status
6. Emit `milestone_wave_completed` event in `crates/sdlc-server/src/routes/runs.rs` on successful wave completion
7. Emit `feature_phase_advanced` event in `crates/sdlc-core/src/feature.rs` when feature transitions to IMPLEMENTATION or beyond
8. Add `SseMessage::ChangelogUpdated` variant to `state.rs`; add mtime watcher for `.sdlc/changelog.yaml` → broadcasts it
9. Unit tests: append → query round-trip, since-filter, limit, empty file handling

## Wave 2 (parallel — both depend on Wave 1)

### Feature: changelog-api
**Title:** Changelog REST endpoint — query events by timestamp
**Depends on:** changelog-core

Tasks:
1. Create `crates/sdlc-server/src/routes/changelog.rs` — `GET /api/changelog` with `since` (ISO timestamp) and `limit` (usize, default 50) query params; returns `{ events: Vec<ChangeEvent>, total: usize }`
2. Register route in `crates/sdlc-server/src/routes/mod.rs` router
3. Frontend hook: `useChangelog(since?: string)` — fetches `/api/changelog?since=<ts>`, re-fetches on `ChangelogUpdated` SSE event

### Feature: changelog-cli
**Title:** `sdlc changelog` CLI command — terminal digest of recent project activity
**Depends on:** changelog-core

Tasks:
1. Create `crates/sdlc-cli/src/cmd/changelog.rs` — `sdlc changelog [--since <date|Nd>] [--limit <N>] [--json]`
2. Default: last 7 days, limit 20 — pretty-printed with icons (⚠️ run_failed, 🚀 feature_merged, ✅ approvals, 🔄 phase_advanced)
3. `--since` flag accepts: ISO date (`2026-03-01`), relative (`3d`, `7d`, `1w`), or `last-merge`
4. Register in `crates/sdlc-cli/src/main.rs` command dispatcher

## Wave 3 (depends on Wave 2)

### Feature: changelog-dashboard-banner
**Title:** Dashboard "What Changed" banner — since-your-last-visit event feed
**Depends on:** changelog-api (useChangelog hook)

Tasks:
1. Create `frontend/src/components/layout/WhatChangedBanner.tsx`
   - On mount: read `localStorage.getItem('sdlc_last_visit_at')`; fetch changelog since that timestamp
   - If events.length > 0: render banner with count + relative time since last visit
   - Expand: show up to 7 events sorted (⚠️ run_failed first, then by timestamp desc); show "See X more" if truncated
   - Dismiss: set `last_visit_at = now`, collapse banner; on next visit count resets
   - Page unload / navigation: save `last_visit_at = now` via `beforeunload`
2. Wire `⚠️ run_failed` items to link to the run detail in the Runs page
3. Wire `🚀 feature_merged` items to link to the feature in the feature list
4. Add `<WhatChangedBanner />` to the Dashboard page above the main content zones
5. First-ever visit: no banner (no `last_visit_at` in localStorage) — correct behavior, no special case needed

## Connections to existing milestones
- `telegram-digest-bot` (active) — the `/api/changelog?since=<ts>` endpoint this builds is the data source the Telegram bot digest should read from. That milestone's digest cron should be updated to query this endpoint once it ships.
- `dashboard-rethink` (active) — the banner integrates into the dashboard page but does not change the zone layout. No coordination needed; both can progress independently.

## Definition of Done
A developer opens the dashboard after 2+ days away and sees a banner: "7 changes since you were last here (2 days ago)" with an expandable feed showing failed runs first, then merges and approvals. Failed runs have a link to the run detail. The same data is queryable via `sdlc changelog --since 2d` in the terminal.
