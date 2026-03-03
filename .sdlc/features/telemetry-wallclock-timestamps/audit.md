# Security Audit: Add Wall-Clock Timestamp to Every Telemetry Event

## Scope

This audit covers the single change in this feature:

- **File:** `crates/sdlc-server/src/routes/runs.rs`
- **Change:** `message_to_event` now captures `chrono::Utc::now()` as an RFC-3339 string and
  inserts it as `"timestamp"` into every returned JSON event object.

## Attack Surface Analysis

### 1. Information Disclosure

**Finding:** Telemetry events now carry a precise wall-clock timestamp (millisecond precision).

**Assessment:** The telemetry endpoint (`GET /api/runs/:id/telemetry`) is already
authenticated behind the tunnel auth middleware (`crates/sdlc-server/src/auth.rs`). Run IDs
are non-guessable. The timestamp reveals when a specific agent run event occurred, but this
is low-sensitivity data that is subordinate to the event content that was already exposed.
No new information class is introduced.

**Action:** Accept — no change to authentication or authorization requirements.

### 2. Timing Oracles / Side-Channel

**Finding:** An attacker with access to the endpoint could potentially infer server
system time from the `timestamp` field.

**Assessment:** System time disclosure is a negligible risk in this context. The server
is already behind a private tunnel. The `started_at` / `completed_at` fields on `RunRecord`
already expose RFC-3339 timestamps through the runs list endpoint. This change adds no new
timing surface.

**Action:** Accept.

### 3. Injection / Serialization

**Finding:** The timestamp string is produced by `chrono::Utc::now().to_rfc3339_opts(...)`,
a well-tested standard-library function. It is inserted as a `serde_json::Value::String`,
not as raw string interpolation.

**Assessment:** No injection risk. The value is always a valid RFC-3339 datetime with a
fixed format and no user-controlled components.

**Action:** Accept.

### 4. Denial of Service

**Finding:** `chrono::Utc::now()` is called once per event. This is a single syscall with
negligible overhead.

**Assessment:** No DoS surface introduced.

**Action:** Accept.

### 5. Dependencies

**Finding:** No new crate dependencies. `chrono` was already in `sdlc-server/Cargo.toml`.

**Assessment:** No supply chain risk introduced.

**Action:** Accept.

## Summary

| Finding | Risk | Action |
|---|---|---|
| Timestamp exposes event wall-clock time | Negligible (auth-gated; data already accessible) | Accept |
| Server system time inferrable | Negligible (already exposed via RunRecord fields) | Accept |
| Injection via timestamp value | None (chrono-generated, serde-typed) | Accept |
| DoS via overhead | None (single syscall per event) | Accept |
| New dependencies | None | Accept |

## Verdict

APPROVED — no security findings require remediation. The change is a pure additive field
injection with no new attack surface beyond what was already present in the telemetry
endpoint.
