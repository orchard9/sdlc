# Security Audit: changelog-api

## Scope

Files audited:
- `crates/sdlc-core/src/event_log.rs`
- `crates/sdlc-server/src/routes/changelog.rs`
- `frontend/src/hooks/useChangelog.ts`

## Surface Analysis

### Authentication

The `GET /api/changelog` endpoint is protected by the existing `sdlc-server` auth middleware (token/cookie gate with local bypass). No new authentication surface is introduced. The endpoint does not bypass or weaken the existing auth layer.

**Finding: None — auth coverage is inherited from the existing middleware stack.**

### Input Validation

**`since` parameter:** Parsed as `DateTime<Utc>` via `chrono`. An invalid value returns `AppError::bad_request("invalid since timestamp")` (HTTP 400) before entering `spawn_blocking`. No path injection or format string risk — the value is parsed by chrono's date parser, not interpreted as a path or command.

**`limit` parameter:** Parsed by Axum's `Query<ChangelogQuery>` extractor as `Option<usize>`. Invalid values (e.g. `limit=abc`, `limit=-1`) are rejected by Axum's deserialization before the handler runs, returning 422 Unprocessable Entity. Defaults to 100 — no unbounded memory allocation risk.

**Finding: None — input validation is correct and complete.**

### File System Access

`query_events` constructs the path as `root.join(".sdlc").join("changelog.yaml")` where `root` is the pre-configured project root from `AppState`. No user-supplied path components are used in file path construction. No path traversal risk.

**Finding: None.**

### Data Exposure

The changelog events contain feature slugs, event kinds, and timestamps. No secrets, tokens, or user PII are written to or read from `changelog.yaml`. The data exposed matches what is already visible in the `sdlc feature show` and `sdlc state` CLI outputs.

**Finding: None — no sensitive data exposure.**

### Denial of Service

- `limit` caps the number of events returned (default 100). An attacker cannot force unbounded memory allocation via the `limit` parameter.
- `since` with a very old timestamp could cause `load_events` to read a large `changelog.yaml`. This is an inherent property of the v1 implementation (noted as a [user-gap] in changelog-core). For a local dev tool, this is acceptable. For a public-facing deployment, rate limiting at the reverse proxy layer (e.g. Cloudflare) is the appropriate mitigation.
- `spawn_blocking` prevents blocking the async runtime for large file reads.

**Finding: Acceptable for current use case. Tracked as changelog-core T10 for future tail-read optimization.**

### Dependency Risk

No new crate dependencies are introduced. `chrono` (already in the dependency graph) is used for timestamp parsing and comparison.

**Finding: None.**

### CORS

The existing `CorsLayer::new().allow_origin(Any)` applies to all routes including `/api/changelog`. This is the project's current policy for development convenience; it is not changed by this feature.

**Finding: Inherited — no change.**

## Verdict: APPROVED

No security findings require immediate action. All identified risks are either mitigated by existing infrastructure or tracked as future improvements in the `changelog-core` feature.
