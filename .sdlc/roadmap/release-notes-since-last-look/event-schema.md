# Event Log: Data Model

## File Location
`.sdlc/changelog.yaml` — single append-only file, managed by sdlc-core

## Schema

```yaml
events:
  - id: "20260302-143211-abc"      # unique, sortable (timestamp + random suffix)
    kind: feature_merged            # event kind (see Event Kinds below)
    timestamp: "2026-03-02T14:32:11Z"
    label: "Feature 'My Feature' merged"  # human-readable summary
    slug: my-feature                # optional: feature slug if relevant
    meta: {}                        # optional kind-specific metadata
```

## Event Kinds

### Always emit (highest signal)
| Kind | Trigger | Meta |
|---|---|---|
| `feature_merged` | `sdlc merge <slug>` | `{ title }` |
| `run_failed` | run completes with `status: failed` | `{ run_id, key, cost_usd, turns }` |
| `milestone_wave_completed` | wave run completes successfully | `{ milestone_slug, wave_num, feature_count }` |
| `feature_phase_advanced` | feature moves into IMPLEMENTATION or beyond | `{ from, to, title }` |

### Emit on significant approvals
| Kind | Trigger | Meta |
|---|---|---|
| `review_approved` | `sdlc artifact approve <slug> review` | `{ title }` |
| `audit_approved` | `sdlc artifact approve <slug> audit` | `{ title }` |
| `qa_approved` | `sdlc artifact approve <slug> qa_results` | `{ title }` |

### Explicitly NOT emitted (low signal / noise)
- `artifact draft` — work in progress, not worth notifying
- `artifact reject` — will be retried, not an alert
- Ponder session starts/ends
- Knowledge research runs
- SSE broadcast events (these are ephemeral, not persistent)
- Advisory runs (informational, not actionable)

## Querying
```bash
# CLI
sdlc changelog                    # last 20 events
sdlc changelog --since 3d         # last 3 days
sdlc changelog --since 2026-03-01 # since specific date

# API
GET /api/changelog                         # all recent events
GET /api/changelog?since=<ISO timestamp>   # events after timestamp
GET /api/changelog?limit=20               # limit count
```

## Implementation Notes
- Appending is safe — one writer at a time (CLI commands are synchronous)
- Server reads on-demand; no caching needed (file is small, < 500KB for any real project)
- ID format: same as run IDs (`YYYYMMDD-HHMMSS-xxx`) for sortability
- mtime watcher on `.sdlc/changelog.yaml` triggers SSE `Update` event → frontend re-fetches
