# Security Audit: changelog-core

## Surface area

The changelog event log is a local file write system with no network exposure in this feature. The audit focuses on: file write safety, data integrity, information disclosure risks, and denial-of-service potential.

## Findings

### F1 — File path is fixed, no injection risk

The changelog path is always `{root}/.sdlc/changelog.yaml` where `root` is validated at process startup. No user-supplied path components. No path traversal risk.

**Verdict**: Pass.

### F2 — Atomic write via `io::atomic_write` prevents partial writes

`append_event` performs a read-then-write using `io::atomic_write` (tempfile + rename). The file is never partially written or left in an inconsistent state if the process crashes mid-write. This is the same pattern used by all other state files.

**Verdict**: Pass.

### F3 — Unbounded file growth

The changelog grows indefinitely. An adversarial or runaway process that generates frequent events (e.g. a broken agent that repeatedly fails) could cause `changelog.yaml` to grow without bound. At ~200 bytes per event and 1 event/second, the file would reach 1 GB in ~58 days.

**Mitigation**: The `query_events` function correctly handles large files (reads are bounded by `limit`). The file itself is append-only and not read during `append_event`. This is an operational concern, not a security vulnerability in the current threat model (local single-user tool). A rotation or max-size cap would be a future improvement.

**Action**: Add a task to track file rotation as a future improvement.

**Verdict**: Accept — log as a known limitation.

### F4 — No validation of `slug` content in event records

The `slug` field in emitted events comes from feature slugs (already validated by `paths::validate_slug`) or from run keys parsed by `extract_slug_from_key`. The run key parsing returns arbitrary strings that may not be valid slugs (e.g. `"unknown"` for unrecognized keys). These strings are serialized to YAML but never executed or used in path operations.

**Verdict**: Pass — YAML serialization is not a code execution path.

### F5 — `metadata` / `details` field accepts arbitrary JSON

The `serde_json::Value` metadata field can contain arbitrary JSON. In the current implementation all callers use `serde_json::json!({})` (empty) or small known-key objects. A future caller could accidentally serialize sensitive data (tokens, secrets) into this field.

**Verdict**: Accept for now — callers are all internal and controlled. Document in API as "do not include sensitive data in metadata."

### F6 — SSE `ChangelogUpdated` broadcasts without authentication gate

The SSE `ChangelogUpdated` message is broadcast to all connected SSE clients. The sdlc-server already uses tunnel auth middleware (`crates/sdlc-server/src/auth.rs`) to gate all connections. The `ChangelogUpdated` event carries no payload — clients must re-fetch via the REST API to see event content. There is no information disclosure via the SSE event itself.

**Verdict**: Pass — no sensitive data in SSE payload; auth is at the connection level.

### F7 — Concurrent writes

`append_event` reads then writes atomically. In a concurrent scenario (two processes appending simultaneously) a race condition could cause one event to be lost — the second write would overwrite the first's result if the reads are interleaved. In practice, `sdlc` is a single-process tool and the server invokes blocking tasks sequentially. This is a theoretical concern for multi-process use.

**Verdict**: Accept — document as a known limitation for multi-process environments.

## Actions taken

- F3: Adding task T12 to track changelog rotation as a future improvement.
- F5: The existing implementation uses only controlled `serde_json::json!({})` or small metadata objects — no sensitive data at risk currently.

## Verdict

No security blockers. Two known limitations (F3, F7) are accepted and tracked. The implementation correctly follows existing codebase patterns for file safety (atomic write), path handling (fixed paths, no injection), and SSE broadcast (no sensitive payload).
