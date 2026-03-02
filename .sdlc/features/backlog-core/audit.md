# Security Audit: backlog-core

## Surface

Pure in-process Rust library. No network I/O, no HTTP endpoints, no user input at
this layer. The CLI and server layers handle external input before reaching these functions.

## Findings

### Input Validation

- **Title field**: No length limit enforced in `backlog.rs`. A malicious caller could write
  an arbitrarily long title. Acceptable risk: all callers are the CLI (controlled input).
  The YAML file has no enforced size limit, but this matches all other sdlc-core data models
  (advisory findings, escalations, tasks). No change required.

- **park_reason validation**: `trim()` is used correctly — whitespace-only strings are
  rejected. The check is enforced before any write occurs.

- **ID parsing**: `strip_prefix('B').and_then(|n| n.parse::<u32>().ok())` — safe. Malformed
  IDs are silently skipped (non-parseable suffix → excluded from max calculation → sequence
  advances correctly). No panic possible.

### File I/O

- All writes go through `io::atomic_write` — consistent with the rest of the codebase.
  Concurrent write protection is OS-level (atomic rename). Same guarantee as advisory.yaml
  and escalations.yaml.

- `.sdlc/backlog.yaml` is a project-local file, not accessible over the network at this
  layer. Server routes (a separate feature) will expose it via HTTP with existing auth
  middleware.

### Serialization

- `serde_yaml` is used consistently with the rest of sdlc-core. No custom deserializers.
  Malformed YAML returns `SdlcError::Yaml` — handled by callers.

### Data Sensitivity

- Backlog items may contain file paths and function names from the codebase (evidence field).
  This is intentional — the value of the evidence field is grounding the concern. No secrets
  or credentials are expected in backlog items by design. No special handling required.

## Verdict

No security issues. Surface is minimal and consistent with existing data layer patterns.
