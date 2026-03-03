# V1 Implementation Sketch

## What We're Building in V1
Dashboard "What Changed" banner that shows events since the user's last web UI visit.

## Rust: Event Log Infrastructure

### New file: `crates/sdlc-core/src/event_log.rs`
```rust
pub struct ChangeEvent {
    pub id: String,        // "20260302-143211-abc"
    pub kind: EventKind,
    pub timestamp: DateTime<Utc>,
    pub label: String,
    pub slug: Option<String>,
    pub meta: serde_json::Value,
}

pub enum EventKind {
    FeatureMerged,
    RunFailed,
    MilestoneWaveCompleted,
    FeaturePhaseAdvanced,
    ReviewApproved,
    AuditApproved,
    QaApproved,
}

pub fn append_event(root: &Path, event: ChangeEvent) -> Result<()> { ... }
pub fn query_events(root: &Path, since: Option<DateTime<Utc>>, limit: Option<usize>) -> Result<Vec<ChangeEvent>> { ... }
```

### File: `.sdlc/changelog.yaml`
Append-only. `append_event` reads, appends, atomic-writes.

### Where to emit events (in sdlc-cli)
- `crates/sdlc-cli/src/cmd/merge.rs` → emit `FeatureMerged` after successful merge
- `crates/sdlc-cli/src/cmd/artifact.rs` → emit `ReviewApproved`, `AuditApproved`, `QaApproved` on approve
- `crates/sdlc-server/src/routes/runs.rs` → emit `RunFailed` when run finishes with error
- `crates/sdlc-server/src/routes/runs.rs` → emit `MilestoneWaveCompleted` on wave run completion
- `crates/sdlc-core/src/feature.rs` → emit `FeaturePhaseAdvanced` on phase transitions to IMPLEMENTATION+

## Server: New Endpoint

### `crates/sdlc-server/src/routes/changelog.rs`
```rust
// GET /api/changelog?since=<ISO>&limit=<N>
async fn get_changelog(
    Query(params): Query<ChangelogParams>,
    State(app): State<AppState>,
) -> impl IntoResponse { ... }
```

Register in router alongside other GET routes.

## Frontend: Dashboard Banner

### New component: `frontend/src/components/layout/WhatChangedBanner.tsx`
- On mount: read `localStorage.getItem('sdlc_last_visit_at')` 
- Fetch `GET /api/changelog?since=<timestamp>`
- If events.length > 0: render banner with count
- On expand: show event feed
- On dismiss/unmount: `localStorage.setItem('sdlc_last_visit_at', new Date().toISOString())`

### Integration point
Add `<WhatChangedBanner />` at the top of the dashboard page (above the main content).

## Event Feed Item Component
```tsx
<EventFeedItem event={event} />
// icon: ⚠️ for run_failed, 🚀 for feature_merged, ✅ for approvals, 🔄 for transitions
// label from event.label
// relative time ("2 days ago")
// optional action link for failed runs
```

## SSE Integration
mtime watcher on `.sdlc/changelog.yaml` → broadcasts `SseMessage::Update`
Frontend re-fetches changelog count on Update → badge updates without page reload

## Estimated Effort
- Event log Rust layer: 1 day
- Emit events in right CLI/server locations: 0.5 day
- Server endpoint: 0.5 day
- Frontend banner + feed: 1 day
**Total: ~3 days**

## Commit Signal
When event schema is finalized and we know which events to emit. That's done.
