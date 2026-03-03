# Spec: changelog-core

## Summary

Add an append-only changelog event log to sdlc-core that records 7 high-signal lifecycle events to `.sdlc/changelog.yaml`. Events are emitted from CLI commands and the server when key feature and run milestones occur. The event log is tail-read (memory-safe for long-lived projects) and triggers an SSE `ChangelogUpdated` notification when the file changes.

## Problem

There is no persistent, structured record of significant project events (merges, phase advancements, run failures, wave completions). Without it, agents and dashboards cannot answer "what has happened recently?" or show a meaningful activity feed. Ad-hoc log scraping is fragile; a first-class event source is needed.

## Solution

### Event log file

A new file `.sdlc/changelog.yaml` stores a list of `ChangeEvent` entries. Each entry has:

- `kind`: one of the 7 `EventKind` variants (see below)
- `slug`: feature or milestone slug providing context (required, extracted from call-site context or run key)
- `timestamp`: UTC RFC 3339 timestamp of when the event occurred
- `details`: optional free-form YAML map for kind-specific fields (e.g. `phase`, `wave`, `session_id`)

The file grows via append. It is never rewritten in full during normal operation.

### EventKind variants (7 total)

| Variant | Emitted when |
|---|---|
| `feature_merged` | `sdlc merge <slug>` succeeds |
| `run_failed` | a spawned agent run completes with `failed` status |
| `milestone_wave_completed` | a wave execution run completes successfully |
| `feature_phase_advanced` | a feature transitions to `implementation` phase or later |
| `review_approved` | `sdlc artifact approve <slug> review` succeeds |
| `audit_approved` | `sdlc artifact approve <slug> audit` succeeds |
| `qa_approved` | `sdlc artifact approve <slug> qa_results` succeeds |

### Core API (sdlc-core)

New module `crates/sdlc-core/src/event_log.rs`:

```rust
pub struct ChangeEvent {
    pub kind: EventKind,
    pub slug: String,
    pub timestamp: DateTime<Utc>,
    pub details: Option<serde_yaml::Value>,
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

// Append a single event (atomic per-write, not a full rewrite)
pub fn append_event(root: &Path, event: &ChangeEvent) -> Result<()>;

// Query the tail of the changelog, newest-first
pub fn query_events(root: &Path, since: Option<DateTime<Utc>>, limit: Option<usize>) -> Result<Vec<ChangeEvent>>;
```

`append_event` uses `io::append_text` to append a single YAML document separator (`---`) plus the event entry rather than loading and rewriting the file. This keeps writes O(1) regardless of log size.

`query_events` tail-reads the file using a bounded buffer or by reading from the end with `BufReader` seeking to avoid loading the full file in memory. Events are filtered by `since` timestamp and truncated to `limit` count, returned newest-first.

### Slug extraction for run events

For `run_failed` and `milestone_wave_completed` events, the slug is parsed from the run key:
- `sdlc-run:my-feature` → slug `my-feature`
- `milestone-run-wave:v15` → slug `v15`
- Any other key → slug `unknown`

This parsing is done at the call site in `runs.rs`, not in `event_log.rs`.

### SSE notification

Add `SseMessage::ChangelogUpdated` variant to `state.rs`. Add an mtime watcher for `.sdlc/changelog.yaml` in the server startup (same pattern as existing watchers). When the file changes, broadcast `SseMessage::ChangelogUpdated`.

### Emission call sites

| File | What triggers it |
|---|---|
| `crates/sdlc-cli/src/cmd/merge.rs` | After successful merge |
| `crates/sdlc-cli/src/cmd/artifact.rs` | After successful approve for `review`, `audit`, `qa_results` |
| `crates/sdlc-core/src/feature.rs` | When transitioning to `implementation` phase or beyond |
| `crates/sdlc-server/src/routes/runs.rs` | On `run_failed`; on `milestone_wave_completed` |

Emission failures are non-fatal — log a warning and continue. The changelog is observability data, not critical-path state.

## Constraints

- No `unwrap()` in library code — use `?` and `SdlcError`
- All file writes go through `io::append_text` (for changelog) or `io::atomic_write` (for tests)
- `query_events` must be safe for files with thousands of entries — read from the tail
- Emission is fire-and-forget — never block the main operation on changelog write errors
- The `ChangelogUpdated` SSE variant does not carry payload; clients re-fetch via the REST API
- No new REST API in this feature — query_events is library-level only; the API route is a separate feature

## Out of Scope

- REST endpoint to query events (separate `changelog-api` feature)
- Dashboard UI panel (separate `changelog-dashboard-banner` feature)
- CLI subcommand to tail the log (separate `changelog-cli` feature)
- Backfilling historical events from existing state

## Acceptance Criteria

1. `.sdlc/changelog.yaml` is created and appended to on each emission call site trigger
2. `query_events` returns events correctly filtered by `since` and `limit`, newest-first
3. `query_events` handles an empty or missing file without error
4. All 7 `EventKind` variants round-trip through YAML serde without data loss
5. `append_event` is called correctly in all 5 call sites (merge, artifact approve x3, feature phase advance, run failed, wave completed)
6. `SseMessage::ChangelogUpdated` is emitted when the file's mtime changes
7. A failing `append_event` call does not abort the parent operation
8. Unit tests cover: append/query round-trip, since filter, limit, empty file, missing file
