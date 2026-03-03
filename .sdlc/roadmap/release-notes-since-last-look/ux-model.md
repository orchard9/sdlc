# UX Model: What Changed Since You Last Looked

## Web UI: Dashboard Banner (V1)

### Mechanism
- On page unload → `localStorage.setItem('last_visit_at', new Date().toISOString())`
- On page load → read `last_visit_at`, query `GET /api/changelog?since=<last_visit_at>`
- If count > 0 → show banner

### Banner design (Ben Hartley's model)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│  📋  7 changes since you were last here (2 days ago)  [Expand ▼]  [Dismiss] │
└─────────────────────────────────────────────────────────────────────────────┘
```

On expand:
```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Recent Activity — Since March 1 at 2:14 PM                       [Collapse] │
├─────────────────────────────────────────────────────────────────────────────┤
│  ⚠️  Agent run failed for 'my-other-feature'        2 days ago     [Retry →] │
│  🚀  Feature 'quota-visibility-panel' merged         2 days ago              │
│  ✅  QA approved for 'concurrency-heatmap'           1 day ago               │
│  ✅  Review approved for 'run-activity-ui'           3 hours ago             │
│  🔄  'dev-driver-tool' → IMPLEMENTATION              1 hour ago              │
│  ✅  Milestone wave v15 completed (4 features)       45 min ago              │
│  ⚠️  Agent run failed for 'dev-driver-init'          12 min ago    [Retry →] │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Event categorization in UI
- `⚠️` Failed runs — shown first, most actionable (link to run detail)
- `🚀` Merges — high celebration, positive signal
- `✅` Significant approvals (review, audit, qa)
- `🔄` Phase transitions
- No `📝` artifact drafts or ponder sessions — those are noise

### Dismiss behavior
- "Dismiss" → updates `last_visit_at` to now without expanding
- After dismiss, banner shows "catch up" count in a subtle badge on nav (not a banner)

## CLI: `sdlc changelog` (V2)

```bash
$ sdlc changelog
Recent project activity (last 7 days):

  ⚠️  2026-03-02 23:28  Agent run FAILED for 'dev-driver-init'  (12 turns, $0.18)
  ✅  2026-03-02 23:00  QA approved for 'concurrency-heatmap'
  🔄  2026-03-02 22:45  'dev-driver-tool' transitioned to IMPLEMENTATION
  ✅  2026-03-02 20:15  Review approved for 'run-activity-ui'
  🚀  2026-03-01 18:30  Feature 'quota-visibility-panel' merged

$ sdlc changelog --since 2d --json
# JSON output for scripting / notification consumers
```

## Notification Consumer API (V3)

```
GET /api/changelog?since=2026-03-01T00:00:00Z&limit=50
→ { events: [...], has_more: false, oldest_returned: "..." }
```

WhatsApp/Telegram bot stores its last-seen timestamp, polls this endpoint, formats events into a message.

## What NOT to build
- A dedicated "Release Notes" page — the dashboard banner is the right pattern
- Per-user server-side tracking — localStorage is enough for the web UI case
- Agent changelog context — agents have the oracle; this would be solving a non-problem
