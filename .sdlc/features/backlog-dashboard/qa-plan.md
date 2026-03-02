# QA Plan: Backlog section in Dashboard frontend

## Scope

This plan covers backend API correctness (Rust unit tests), frontend integration (manual browser verification), and acceptance criteria validation for all 9 tasks.

---

## 1. Build verification

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
cd frontend && npm ci && npm run build
```

All must pass with zero errors and zero warnings.

---

## 2. Rust unit tests (routes/backlog.rs)

These run automatically as part of `cargo test`.

| Test | Assertion |
|------|-----------|
| `list_empty_when_no_backlog_file` | GET `/api/backlog` returns `[]` with no backlog.yaml |
| `list_returns_open_items` | Seed via `BacklogStore::add`; GET returns all items with correct fields |
| `list_status_filter_open` | Seed open+parked items; `?status=open` returns only open |
| `list_source_feature_filter` | Seed items with different source features; filter returns correct subset |
| `park_item_updates_status` | POST `/api/backlog/B1/park` with reason → item.status = "parked", park_reason set |
| `park_empty_reason_returns_422` | POST with `{ park_reason: "" }` → 422 response |
| `park_whitespace_reason_returns_422` | POST with `{ park_reason: "   " }` → 422 response |
| `park_promoted_item_returns_422` | POST park on promoted item → 422 response |
| `promote_item_creates_feature` | POST promote → feature dir exists at `.sdlc/features/<slug>/` |
| `promote_item_marks_promoted` | POST promote → item.status = "promoted", item.promoted_to = slug |
| `promote_duplicate_slug_returns_409` | POST promote with existing feature slug → 409 response |
| `get_missing_item_returns_404` | POST park on non-existent id "B99" → 404 |

---

## 3. Manual browser verification

Prerequisites: `sdlc ui` running, seed some backlog items via `sdlc backlog add`.

### 3a. Empty state

1. Ensure `.sdlc/backlog.yaml` does not exist or has no open items
2. Load Dashboard (`/`)
3. **Verify**: "Backlog" section header visible
4. **Verify**: Empty state copy rendered verbatim:
   > "When an agent finishes a run, it captures concerns that were out of scope for that feature. Items appear here. Promote to create a tracked feature, or Park to set aside. Nothing in the backlog blocks your current work."
5. **Verify**: No item cards rendered, no filter dropdown visible

### 3b. Items display

Seed: `sdlc backlog add --title "auth.rs: token race" --kind concern --source-feature my-feature`
         `sdlc backlog add --title "DB compaction not configured" --kind debt`

1. Reload Dashboard
2. **Verify**: "Backlog" header shows "2 open" badge
3. **Verify**: First item shows red-tinted "concern" badge, title, "→ my-feature" link, timestamp
4. **Verify**: Second item shows amber-tinted "debt" badge, title, timestamp (no source feature link)
5. **Verify**: Filter dropdown absent (only 1 source feature among items)

### 3c. Source feature filter

Seed a third item: `sdlc backlog add --title "idea: improve logging" --kind idea --source-feature other-feature`

1. Reload Dashboard
2. **Verify**: Filter dropdown appears (2+ distinct source features)
3. Select "my-feature" → only auth.rs item visible
4. Select "other-feature" → only logging idea visible
5. Select "All sources" → all items visible

### 3d. Staleness indicator

Manually modify `.sdlc/backlog.yaml`: set `created_at` on one item to a date >60 days ago.

1. Reload Dashboard
2. **Verify**: That item shows amber Clock icon + "stale" label before the timestamp
3. **Verify**: Other items (within 60 days) show no stale indicator

### 3e. Park flow

1. Click "Park" on an item
2. **Verify**: Textarea expands, "Confirm" button disabled
3. Leave textarea empty, click Confirm — **Verify**: button stays disabled (no submit)
4. Type a reason ("revisit after v2"), click Confirm
5. **Verify**: Item disappears from open list
6. **Verify**: "Show parked (1)" toggle appears (or increments)
7. Click toggle — **Verify**: Parked item shown with park_reason, no Park/Promote buttons

### 3f. Promote flow

1. Click "Promote" on an item
2. **Verify**: Inline form appears with slug input (auto-derived from title) and title input pre-populated
3. **Verify**: "Promote" button has tooltip "Creates a draft feature. No agent work begins automatically."
4. Submit form
5. **Verify**: Item disappears from open list
6. **Verify**: Toast appears at bottom-right: "Feature created → [slug]" with clickable link
7. Click toast link — **Verify**: navigates to `/features/<slug>`
8. **Verify**: Feature exists in `.sdlc/features/<slug>/manifest.yaml`

### 3g. Promote duplicate slug error

1. Note an existing feature slug (e.g., "backlog-dashboard")
2. Click "Promote" on any item, set slug to "backlog-dashboard"
3. Submit
4. **Verify**: Inline error "A feature with that slug already exists. Choose a different name."
5. **Verify**: Item still in open list

### 3h. Park reason server-side validation

1. Bypass UI by posting directly: `curl -X POST /api/backlog/B1/park -d '{"park_reason":""}'`
2. **Verify**: 422 response with error message

---

## 4. Acceptance criteria checklist

| # | Criterion | Method |
|---|-----------|--------|
| 1 | GET `/api/backlog` returns `[]` when no backlog.yaml | Rust test |
| 2 | GET `/api/backlog` returns items with correct fields | Rust test |
| 3 | POST park with valid reason → status = parked | Rust test |
| 4 | POST promote → feature dir created, item promoted | Rust test |
| 5 | Dashboard shows Backlog section with open count badge | Browser 3b |
| 6 | Empty state copy is shown verbatim when no open items | Browser 3a |
| 7 | Item cards show title, kind badge, source_feature link, timestamp | Browser 3b |
| 8 | Staleness indicator shown for items > 60 days old | Browser 3d |
| 9 | Park CTA requires non-empty reason before confirm | Browser 3e |
| 10 | Promote CTA tooltip exact wording | Browser 3f |
| 11 | Promote success: toast with link to new feature | Browser 3f |
| 12 | Promote 409: inline error shown | Browser 3g |
| 13 | Source feature filter dropdown appears with 2+ sources | Browser 3c |
| 14 | Parked items hidden by default, toggle reveals them | Browser 3e |

---

## 5. Pass/fail definition

**Pass**: All 12 Rust tests pass, all 14 acceptance criteria verified, build clean.

**Fail**: Any Rust test failure, any acceptance criterion not met, clippy warning, or build error.

Failed items must be fixed before `approve_merge`.
