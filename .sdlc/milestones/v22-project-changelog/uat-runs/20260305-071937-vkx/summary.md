# UAT Run Summary: v22-project-changelog

**Run ID:** 20260305-071937-vkx
**Date:** 2026-03-05T07:19:37Z
**Agent:** claude-opus-4-6
**Verdict:** pass_with_tasks

## Overview

All 7 scenarios exercised with the sdlc server running at localhost:7777. The dashboard changelog banner, CLI, API, and event log all function as specified. Two previously-known CLI issues remain (tasks from prior run).

## Scenario Results

### Scenario 1: First Visit (New Developer) — PASS
- "Recent project activity" section visible above main dashboard content
- Header reads "Recent project activity" (not "since your last visit")
- Events from last 7 days listed (merges, approvals, phase transitions, failed runs)
- Failed run event (`run_failed · ponder-ux-polish`) appears first in list
- Clicking failed run navigates to `/runs` page (not specific run detail — see tasks)

### Scenario 2: Returning User With Changes — PASS
- Visited dashboard, dismissed banner, appended test event to changelog.yaml
- Navigated away and back — banner reappeared with "2 changes since just now"
- Test merge event `feature_merged · uat-test-event` correctly displayed

### Scenario 3: Dismiss Persists Across SPA Navigation — PASS
- Banner visible, navigated to Runs via SPA, returned — banner still showing (SPA did NOT reset)
- Clicked Dismiss — banner disappeared
- Navigated away and back — only genuinely new events appear (dismiss updated last_visit_at correctly)

### Scenario 4: Failed Run Link Works — PASS (partial)
- Failed run event visible in banner with warning icon
- Link navigates to `/runs` (Run History page) — not to specific run detail
- Run History shows failed runs with red indicators
- **Task:** Failed run events should deep-link to specific run detail (pre-existing changelog-cli#T5)

### Scenario 5: CLI Changelog — PASS (partial)
- `sdlc changelog` produces formatted output with icons
- `sdlc changelog --since 3d --json` returns JSON array
- **Task:** CLI reads from `.sdlc/.runs/` not `changelog.yaml` — direct merges/approvals invisible (pre-existing changelog-cli#T5)
- **Task:** JSON field names mismatch spec: `category` vs `kind`, `started_at` vs `timestamp`, missing `slug`/`meta` (pre-existing changelog-cli#T6)

### Scenario 6: API Endpoint — PASS
- `GET /api/changelog` returns `{ events: [...], total: N }` with 100 events
- Events have `id`, `kind`, `timestamp`, `slug`, `metadata` fields
- `GET /api/changelog?since=2026-03-01T00:00:00Z&limit=5` correctly filters and limits

### Scenario 7: Event Log Integrity — PASS
- `changelog.yaml` contains 470+ events with correct structure
- `feature_merged`, `review_approved`, `feature_phase_advanced`, `run_failed` events all present
- Events have `id`, `kind`, `slug`, `timestamp`, `metadata` fields

## Pre-existing Tasks (from prior UAT run)
- **changelog-cli#T5** — CLI reads `.runs/` not `changelog.yaml`; direct lifecycle events invisible
- **changelog-cli#T6** — CLI JSON field names don't match spec
- **changelog-core#T13** — `ChangeEvent` struct missing `label` field
