# Scenarios: What Changed Since You Last Looked

## The 5 Scenarios

### 1. Human opens the web UI (primary — highest pain)
**Who**: Developer who worked on the project 2 days ago, now coming back.
**Need**: Re-orientation. "What happened while I was away? Did anything fail?"
**'You' identity**: Browser session → localStorage `last_visit_at` timestamp
**Solution**: Dashboard banner with count + expandable event feed

### 2. Agent invoked (`/sdlc-run`, `/sdlc-next`)
**Who**: Autonomous agent picking up work on a feature or milestone
**Need**: Context to avoid redundant work, know if something unexpected changed
**'You' identity**: Run start timestamp (`run.started_at`) — "new since this run began"
**Solution**: Agents already have `sdlc next --json` and run history. Not a priority.
**Dana's verdict**: V3 or never. The oracle (`sdlc next`) handles this.

### 3. CLI user (`sdlc changelog`)
**Who**: Developer in terminal wanting a digest of recent activity
**Need**: Structured log of what happened — searchable, filterable
**'You' identity**: User provides `--since` flag (relative or absolute date)
**Solution**: `sdlc changelog [--since 3d|7d|<ISO date>] [--limit 20]`

### 4. Notification consumer (WhatsApp/Telegram bots)
**Who**: External service polling for updates to push notifications
**Need**: Events newer than the consumer's last-seen cursor
**'You' identity**: Consumer owns its cursor, stores it externally
**Solution**: `GET /api/changelog?since=<timestamp>` — pagination via timestamp cursor

### 5. New team member
**Who**: Person joining mid-project wanting historical context
**Need**: Full recent history, not just "since last visit"
**Solution**: Same event feed, just no localStorage suppression — show all recent N events

## Priority Order
1. **Web UI banner** — V1 (highest pain, clearest solution)
2. **`sdlc changelog` CLI** — V2
3. **Notification consumer API** — V3 (already partially served by existing `/api/runs`)
4. **Agent context** — Not needed (oracle exists)
