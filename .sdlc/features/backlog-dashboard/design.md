# Design: Backlog section in Dashboard frontend

## Overview

This feature adds a Backlog section to the Dashboard page by wiring together three layers:
1. A new `routes/backlog.rs` module in `sdlc-server` exposes the backlog data and actions
2. API client methods in `frontend/src/api/client.ts` consume those endpoints
3. A `BacklogSection` component rendered in `Dashboard.tsx` provides the UI

The backlog core (`sdlc-core/src/backlog.rs`) already exists and needs no changes.

---

## Backend: `crates/sdlc-server/src/routes/backlog.rs`

Follow the same pattern as `routes/escalations.rs`.

### Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/backlog` | `list_backlog` | Returns all items, supports `?status=` and `?source_feature=` filters |
| POST | `/api/backlog/:id/park` | `park_backlog_item` | Parks item with required `park_reason` |
| POST | `/api/backlog/:id/promote` | `promote_backlog_item` | Creates a draft feature, marks item promoted |

### Handler: `list_backlog`

```rust
#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub source_feature: Option<String>,
}

pub async fn list_backlog(
    State(app): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    // parse status string → BacklogStatus if present
    // call BacklogStore::list(root, status_filter, source_feature.as_deref())
    // serialize items via backlog_item_to_json helper
}
```

### Handler: `park_backlog_item`

```rust
#[derive(Deserialize)]
pub struct ParkBody { pub park_reason: String }

pub async fn park_backlog_item(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ParkBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    // call BacklogStore::park(root, &id, body.park_reason)
    // return updated item JSON
}
```

### Handler: `promote_backlog_item`

```rust
#[derive(Deserialize)]
pub struct PromoteBody {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
}

pub async fn promote_backlog_item(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<PromoteBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. sdlc_core::feature::Feature::create(root, slug, title, description)
    // 2. BacklogStore::mark_promoted(root, &id, &slug)
    // 3. return { promoted_to: slug }
}
```

### JSON shape (`backlog_item_to_json`)

```json
{
  "id": "B1",
  "title": "...",
  "kind": "concern",
  "status": "open",
  "description": null,
  "evidence": null,
  "source_feature": "auth-race-fix",
  "park_reason": null,
  "promoted_to": null,
  "created_at": "2026-03-01T12:00:00Z",
  "updated_at": "2026-03-01T12:00:00Z"
}
```

### Error mapping

- `SdlcError::BacklogItemNotFound` → 404
- `SdlcError::InvalidTransition { .. }` → 422 with `{ error: reason }`
- Feature already exists (slug collision) → 409

### Registration in `lib.rs`

```rust
.route("/api/backlog", get(routes::backlog::list_backlog))
.route("/api/backlog/{id}/park", post(routes::backlog::park_backlog_item))
.route("/api/backlog/{id}/promote", post(routes::backlog::promote_backlog_item))
```

And add `pub mod backlog;` to `routes/mod.rs`.

---

## Frontend types: `frontend/src/lib/types.ts`

```typescript
export type BacklogKind = 'concern' | 'idea' | 'debt'
export type BacklogStatus = 'open' | 'parked' | 'promoted'

export interface BacklogItem {
  id: string
  title: string
  kind: BacklogKind
  status: BacklogStatus
  description: string | null
  evidence: string | null
  source_feature: string | null
  park_reason: string | null
  promoted_to: string | null
  created_at: string
  updated_at: string
}
```

---

## Frontend API: `frontend/src/api/client.ts`

```typescript
getBacklog: (status?: BacklogStatus, source_feature?: string) =>
  request<BacklogItem[]>(
    `/api/backlog${buildQuery({ status, source_feature })}`
  ),
parkBacklogItem: (id: string, park_reason: string) =>
  request<BacklogItem>(`/api/backlog/${encodeURIComponent(id)}/park`, {
    method: 'POST', body: JSON.stringify({ park_reason }),
  }),
promoteBacklogItem: (id: string, body: { slug: string; title: string; description?: string }) =>
  request<{ promoted_to: string }>(`/api/backlog/${encodeURIComponent(id)}/promote`, {
    method: 'POST', body: JSON.stringify(body),
  }),
```

---

## Frontend UI: Dashboard.tsx

### Data fetching

Add to the existing `useEffect` setup block (or a separate `useEffect` since backlog is independent of config/vision/agents):

```typescript
const [backlog, setBacklog] = useState<BacklogItem[]>([])
const [backlogLoading, setBacklogLoading] = useState(true)

useEffect(() => {
  api.getBacklog('open').then(items => {
    setBacklog(items)
    setBacklogLoading(false)
  }).catch(() => setBacklogLoading(false))
}, [])
```

### Section placement

Insert the Backlog section **between** the Escalations section and the PreparePanel / HITL section (above the wave plan).

### BacklogSection component (inline or extracted)

```
┌──────────────────────────────────────────────────────────┐
│  📋 Backlog                          [3 open]            │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │ [concern] AuthMiddleware: token race under …        │  │
│  │ → auth-race-fix  ·  2 days ago                     │  │
│  │                  [Park]  [Promote]                  │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────┐  │
│  │ ⚠ [debt] redb.rs: compaction not configured       │  │ ← stale (>60 days)
│  │ 63 days ago                                        │  │
│  │                  [Park]  [Promote]                  │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  [source filter: All sources ▾]                         │
│  [Show parked (1)]                                      │
└──────────────────────────────────────────────────────────┘
```

Empty state:
```
When an agent finishes a run, it captures concerns that were out of scope
for that feature. Items appear here. Promote to create a tracked feature,
or Park to set aside. Nothing in the backlog blocks your current work.
```

### Park inline flow

1. Click "Park" → card expands with textarea ("Why park this item?")
2. Confirm button disabled until textarea is non-empty
3. On submit → `api.parkBacklogItem(id, reason)` → re-fetch open items
4. On error: inline error message below textarea

### Promote inline flow

1. Click "Promote" → card expands with:
   - Feature slug input (auto-generated from title: lowercase, spaces→hyphens)
   - Feature title input (pre-populated from backlog item title, editable)
   - Optional description textarea
   - Submit button with tooltip "Creates a draft feature. No agent work begins automatically."
2. On submit → `api.promoteBacklogItem(id, { slug, title, description })` 
3. On success: item removed from list + toast notification: "Feature created → [slug]" (links to `/features/<slug>`)
4. On 409 error: "A feature with that slug already exists. Choose a different name."

### Kind badge colors

| Kind | Style |
|------|-------|
| concern | `bg-red-500/10 text-red-400 border border-red-500/20` |
| idea | `bg-blue-500/10 text-blue-400 border border-blue-500/20` |
| debt | `bg-amber-500/10 text-amber-400 border border-amber-500/20` |

### Staleness indicator

If `(Date.now() - new Date(item.created_at).getTime()) > 60 * 24 * 60 * 60 * 1000`:
Show `<Clock className="w-3 h-3 text-amber-400/60" />` + "stale" text in muted amber before timestamp.

### Source feature filter

Derived client-side from items. Show dropdown only when 2+ distinct `source_feature` values exist among open items. Default: "All sources". Filter is applied to the displayed list; parked items section always shows all parked items regardless.

### Show parked toggle

Below the open list (even if empty), show: "Show parked (N)" — a clickable text link that toggles a collapsed list of parked items. Parked items are read-only (no Park/Promote buttons, just title + kind badge + park_reason).

---

## Toast notification

Use a simple local state toast (no external library). After promote succeeds:

```typescript
const [toast, setToast] = useState<{ message: string; slug: string } | null>(null)
// show for 4 seconds then auto-dismiss
```

Rendered as a fixed bottom-right notification with a link to the new feature.

---

## File changes summary

| File | Change |
|------|--------|
| `crates/sdlc-server/src/routes/backlog.rs` | New file — 3 handlers + JSON helper + tests |
| `crates/sdlc-server/src/routes/mod.rs` | Add `pub mod backlog;` |
| `crates/sdlc-server/src/lib.rs` | Register 3 routes |
| `frontend/src/lib/types.ts` | Add `BacklogItem`, `BacklogKind`, `BacklogStatus` types |
| `frontend/src/api/client.ts` | Add 3 API methods |
| `frontend/src/pages/Dashboard.tsx` | Add backlog state + BacklogSection rendering |

---

## Tests

### Rust unit tests (`routes/backlog.rs`)
- `list_empty_when_no_backlog_file` — returns empty array
- `create_and_list_open` — add via BacklogStore::add, list returns it
- `park_item` — POST park updates status
- `park_empty_reason_returns_422` — validation error
- `promote_item_creates_feature` — feature dir appears, item promoted

### Frontend (manual / E2E)
- Covered in QA plan
