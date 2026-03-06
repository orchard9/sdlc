## Design Decisions

### Entry point
**Inspect** button on route rows where store_only=true. Dispatch routes don't get this button — they already have the event log in the existing Webhook Events section below.

### Panel vs. page
Inline panel that replaces the route section body (not a separate page, not a modal). Keeps context — the user stays in Actions. The panel slides in / expands in-place when Inspect is clicked on a store_only route.

### Two-pane layout
Payload list (left, ~280px), detail viewer (right, flex). Standard mail-client pattern. Works for sequential inspection without losing list position. Mobile: collapse to list-first, detail on tap.

### Time filter
Quick-select chips: 1h / 6h / 24h / 7d / custom. Default: 6h. Calls GET /api/webhooks/{route}/data?since=&until=&limit=1000. Updates list immediately on chip tap.

### Payload list items
Each row: time (HH:MM:SS), content-type, size in bytes, first 60 chars of body preview. Newest first. Selected row highlighted with blue left border.

### JSON viewer
Syntax-highlighted pre block. Copy button in header. Not editable — read-only insight.

### Replay mechanics
'Replay through tool' button → POST /api/webhooks/{route}/replay/{id}. Server re-dispatches the stored raw_body through the tool registered on the route (same as dispatch path, bypasses public webhook ingress). Returns a run ID. Replay result appears as a footer strip below the JSON viewer.

### What replay does NOT do
- Does not re-ingest through POST /webhooks/{route} (would create duplicate storage + secret verification confusion)
- Does not delete the original payload
- Does not advance any offset or cursor

### Open: replay target
V1: always dispatches to the tool registered on the route. V2: could allow picking a different tool for testing.

### Open: retention
store_only payloads accumulate indefinitely. V1 shows a warning if count > 10,000. V2: configurable TTL per route.