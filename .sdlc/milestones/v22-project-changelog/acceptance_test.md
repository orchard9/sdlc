# Acceptance Test: v22-project-changelog

## Setup
- sdlc server running with at least one active project
- At least one past run failure in the run history
- At least one recently merged feature
- Browser with no prior visit to this sdlc instance (clear localStorage or use incognito)

## Scenario 1: First Visit (New Developer)

1. Open the dashboard in a fresh browser (no `sdlc_last_visit_at` in localStorage)
2. **EXPECT**: A "Recent project activity" section is visible above the main dashboard content
3. **EXPECT**: The header reads "Recent project activity" (not "since your last visit")
4. **EXPECT**: Events from the last 7 days are listed — failed runs, merges, approvals visible
5. **EXPECT**: Failed run events appear first in the list
6. **EXPECT**: Clicking a failed run event navigates to that run's detail

## Scenario 2: Returning User With Changes

1. Visit the dashboard (to set `last_visit_at`)
2. Dismiss the banner
3. Trigger a project event: run `sdlc merge <any-mergeable-feature>` (or simulate by directly appending to changelog.yaml)
4. Navigate to a different page (Runs, Features, etc.) and back to the dashboard
5. **EXPECT**: Banner reappears with count "1 change since you were last here"
6. Expand the banner
7. **EXPECT**: The merge event is shown: "🚀 Feature 'X' merged — just now"

## Scenario 3: Dismiss Persists Across SPA Navigation

1. Open dashboard with unread events showing in banner
2. Navigate to the Runs page (SPA navigation, no page reload)
3. Navigate back to the Dashboard
4. **EXPECT**: Banner is still showing the same events — SPA navigation did NOT reset it
5. Click Dismiss
6. **EXPECT**: Banner disappears
7. Navigate away and back
8. **EXPECT**: Banner does NOT reappear (last_visit_at was updated on dismiss)

## Scenario 4: Failed Run Link Works

1. Expand the banner (returning user with at least one failed run event)
2. Click the "→" link on a failed run event
3. **EXPECT**: Navigates to the run detail page showing the failed run
4. **EXPECT**: The failed run's error information is visible

## Scenario 5: CLI Changelog

```bash
sdlc changelog
```
**EXPECT**: Output shows recent events from last 7 days with icons:
- `⚠️` for failed runs
- `🚀` for merges
- `✅` for review/audit/qa approvals
- `🔄` for phase transitions

```bash
sdlc changelog --since 3d --json
```
**EXPECT**: JSON array of events with `id`, `kind`, `timestamp`, `label`, `slug`, `meta` fields

## Scenario 6: API Endpoint

```bash
curl http://localhost:PORT/api/changelog
```
**EXPECT**: JSON `{ events: [...], total: N }` with recent events

```bash
curl "http://localhost:PORT/api/changelog?since=2026-03-01T00:00:00Z&limit=5"
```
**EXPECT**: Only events after the given timestamp, max 5 results

## Scenario 7: Event Log Integrity

```bash
sdlc merge <feature-slug>
cat .sdlc/changelog.yaml
```
**EXPECT**: A `feature_merged` event is appended with correct `slug`, `label`, and `timestamp`

```bash
sdlc artifact approve <feature-slug> review
cat .sdlc/changelog.yaml
```
**EXPECT**: A `review_approved` event is appended

## Pass Criteria
All 7 scenarios pass without manual intervention. The "first visit" scenario shows real recent content, not a blank page. The failed run link navigates correctly. The CLI produces readable output. The API returns structured JSON. The event log reflects real state changes.
