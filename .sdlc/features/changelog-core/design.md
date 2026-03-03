# Design: changelog-core

## Overview

This document describes the technical design for `changelog-core`: the append-only changelog event log in `sdlc-core`. It covers the data model, module structure, I/O strategy, emission call sites, and SSE integration.

## Module: `crates/sdlc-core/src/event_log.rs`

### Data model

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    FeatureMerged,
    RunFailed,
    MilestoneWaveCompleted,
    FeaturePhaseAdvanced,
    ReviewApproved,
    AuditApproved,
    QaApproved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub kind: EventKind,
    pub slug: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_yaml::Value>,
}
```

All fields are required except `details`. The `kind` and `slug` fields are always populated by the call site.

### File path

Constant added to `crates/sdlc-core/src/paths.rs`:

```rust
pub const CHANGELOG_FILE: &str = ".sdlc/changelog.yaml";

pub fn changelog_path(root: &Path) -> PathBuf {
    root.join(CHANGELOG_FILE)
}
```

### Public API

```rust
/// Append a single event to .sdlc/changelog.yaml.
/// Creates the file if absent. Never rewrites existing content.
/// Non-fatal: returns Ok(()) even if the file write fails only at call-sites
/// that explicitly ignore the error. Library code returns Result.
pub fn append_event(root: &Path, event: &ChangeEvent) -> Result<()>;

/// Query events from the changelog, newest-first.
/// `since` filters out events older than the given timestamp.
/// `limit` caps the number of returned events.
/// Returns Ok(vec![]) if the file is absent or empty.
pub fn query_events(
    root: &Path,
    since: Option<DateTime<Utc>>,
    limit: Option<usize>,
) -> Result<Vec<ChangeEvent>>;
```

### Append strategy

`append_event` serializes the event to a YAML block then appends it using `io::append_text`. The format is a YAML list — the file starts as `- kind: ...\n  slug: ...\n` entries. On each call, the serialized event block is appended without loading prior content. This is O(1) per write regardless of log size.

To keep the file valid YAML while still being appendable, the file stores events as a YAML sequence. Each append writes:

```
- kind: feature_merged
  slug: my-feature
  timestamp: '2026-03-02T23:00:00Z'
```

The first write creates the file with the initial entry. Subsequent writes append additional YAML list items. When reading, the full file is parsed as a YAML sequence. The tail-read optimization applies: for `query_events` with a `limit`, we read from the end of the file using `BufReader` with `seek` to avoid loading the full log into memory for long-lived projects.

### Query strategy

For `query_events`:
1. If the file does not exist or is empty, return `Ok(vec![])`.
2. Read the full file and parse as `Vec<ChangeEvent>`.
3. Reverse (newest-first).
4. Filter by `since` if provided.
5. Truncate to `limit` if provided.

The tail-read optimization (seeking from end) is left as a future improvement to keep the initial implementation simple and correct. The file is expected to stay under 10 MB for 2+ years of typical usage (each entry ~100 bytes; 100,000 events = ~10 MB), which is well within acceptable bounds for full-file reads.

## Emission call sites

### 1. `crates/sdlc-cli/src/cmd/merge.rs` — `feature_merged`

After `state.save(root)` succeeds, emit:

```rust
let event = ChangeEvent {
    kind: EventKind::FeatureMerged,
    slug: slug.to_string(),
    timestamp: Utc::now(),
    details: None,
};
if let Err(e) = sdlc_core::event_log::append_event(root, &event) {
    eprintln!("warn: changelog write failed: {e}");
}
```

### 2. `crates/sdlc-cli/src/cmd/artifact.rs` — `review_approved`, `audit_approved`, `qa_approved`

In the `approve()` function, after `feature.save(root)` succeeds, check the artifact type:

```rust
let changelog_kind = match artifact_type {
    ArtifactType::Review => Some(EventKind::ReviewApproved),
    ArtifactType::Audit => Some(EventKind::AuditApproved),
    ArtifactType::QaResults => Some(EventKind::QaApproved),
    _ => None,
};
if let Some(kind) = changelog_kind {
    let event = ChangeEvent { kind, slug: slug.to_string(), timestamp: Utc::now(), details: None };
    if let Err(e) = sdlc_core::event_log::append_event(root, &event) {
        eprintln!("warn: changelog write failed: {e}");
    }
}
```

### 3. `crates/sdlc-core/src/feature.rs` — `feature_phase_advanced`

The `feature.rs` module is in `sdlc-core` and does not have access to the root path at `transition()` time (only the feature struct is mutated in memory; save happens at the call site). The emission is therefore done at the CLI call site after `feature.save(root)`.

In `crates/sdlc-cli/src/cmd/artifact.rs`, after `try_auto_transition(root, slug)` returns a phase, if the phase is `implementation` or later, emit `feature_phase_advanced`.

Alternatively: in `crates/sdlc-cli/src/cmd/feature.rs`'s `transition` subcommand after save.

The simplest correct approach: emit from `try_auto_transition` wrapper in `classifier.rs`, which already has `root`. Add an optional changelog call there.

Actually, looking at the call sites: `try_auto_transition` in `classifier.rs` already receives `root` and knows the slug and resulting phase. This is the cleanest single place.

In `crates/sdlc-core/src/classifier.rs`:

```rust
// After successful transition, emit phase_advanced for implementation+
if matches!(new_phase, Phase::Implementation | Phase::Review | Phase::Audit | Phase::Qa | Phase::Merge | Phase::Released) {
    let event = ChangeEvent {
        kind: EventKind::FeaturePhaseAdvanced,
        slug: slug.to_string(),
        timestamp: Utc::now(),
        details: Some(serde_yaml::to_value(&serde_json::json!({ "phase": new_phase.to_string() })).unwrap_or_default()),
    };
    let _ = event_log::append_event(root, &event); // non-fatal
}
```

### 4. `crates/sdlc-server/src/routes/runs.rs` — `run_failed`, `milestone_wave_completed`

After the `RunFinished` SSE is emitted, and `status == "failed"`, parse the run key to extract slug and emit `RunFailed`:

```rust
if status == "failed" {
    let slug = extract_slug_from_key(&key_clone);
    let event = ChangeEvent {
        kind: EventKind::RunFailed,
        slug,
        timestamp: Utc::now(),
        details: None,
    };
    let root2 = root.clone();
    tokio::task::spawn_blocking(move || {
        let _ = sdlc_core::event_log::append_event(&root2, &event);
    }).await.ok();
}
```

For `milestone_wave_completed`, check if `key_clone.starts_with("milestone-run-wave:")` and `status == "completed"`:

```rust
if key_clone.starts_with("milestone-run-wave:") && status == "completed" {
    let slug = extract_slug_from_key(&key_clone);
    // emit MilestoneWaveCompleted
}
```

The slug extraction helper:

```rust
fn extract_slug_from_key(key: &str) -> String {
    // "sdlc-run:my-feature" → "my-feature"
    // "milestone-run-wave:v15" → "v15"
    // anything else → "unknown"
    key.split_once(':')
        .map(|(_, s)| s.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
```

## SSE: `ChangelogUpdated`

Add to `SseMessage` enum in `crates/sdlc-server/src/state.rs`:

```rust
/// The changelog event log was updated — clients can re-fetch via the API.
ChangelogUpdated,
```

Add an mtime watcher for `.sdlc/changelog.yaml` in the server startup watchers block (same pattern as `.sdlc/advisory.yaml`, `.sdlc/escalations.yaml`, etc.):

```rust
// Watcher for changelog.yaml
let tx_changelog = event_tx.clone();
let changelog_path = root.join(".sdlc/changelog.yaml");
spawn_mtime_watcher(changelog_path, move || {
    let _ = tx_changelog.send(SseMessage::ChangelogUpdated);
});
```

## Unit tests

In `crates/sdlc-core/src/event_log.rs` `#[cfg(test)]` module:

| Test | Verifies |
|---|---|
| `append_query_roundtrip` | Append 3 events, query all, count = 3, fields match |
| `query_since_filter` | Append events at t0 and t1, query since t0+1ms, get only t1 |
| `query_limit` | Append 10 events, query limit=3, get newest 3 |
| `empty_file_ok` | Create empty file, query returns Ok(vec![]) |
| `missing_file_ok` | No file exists, query returns Ok(vec![]) |
| `newest_first_order` | Append t0 then t1, query returns t1 first |

## Non-goals in this feature

- REST API endpoint for querying changelog (changelog-api feature)
- Dashboard UI (changelog-dashboard-banner feature)
- CLI `sdlc changelog tail` command (changelog-cli feature)
- Backfill of historical events

## Dependency additions

No new Cargo dependencies needed. `serde_yaml` is already in `sdlc-core`. `chrono` is already present.

## File changes summary

| File | Change |
|---|---|
| `crates/sdlc-core/src/event_log.rs` | New module: `ChangeEvent`, `EventKind`, `append_event`, `query_events` |
| `crates/sdlc-core/src/lib.rs` | Add `pub mod event_log;` |
| `crates/sdlc-core/src/paths.rs` | Add `CHANGELOG_FILE` const and `changelog_path()` helper |
| `crates/sdlc-cli/src/cmd/merge.rs` | Emit `feature_merged` after successful merge |
| `crates/sdlc-cli/src/cmd/artifact.rs` | Emit `review_approved`, `audit_approved`, `qa_approved` on approve |
| `crates/sdlc-core/src/classifier.rs` | Emit `feature_phase_advanced` from `try_auto_transition` |
| `crates/sdlc-server/src/routes/runs.rs` | Emit `run_failed` and `milestone_wave_completed` |
| `crates/sdlc-server/src/state.rs` | Add `SseMessage::ChangelogUpdated` variant |
| `crates/sdlc-server/src/main.rs` | Add mtime watcher for `changelog.yaml` |
