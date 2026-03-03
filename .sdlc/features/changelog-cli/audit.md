# Security Audit: changelog-cli

## Surface Area

`sdlc changelog` is a read-only CLI command. It:
1. Reads `.sdlc/.runs/*.json` from the local filesystem
2. Filters and formats entries for stdout

No network calls. No writes. No subprocess execution. No user-supplied values are used in file paths.

## Findings

### F1 — Path traversal: NONE
The `--since` and `--limit` flags affect filtering and display only. Neither is used to construct a file path. The run directory is hardcoded as `root.join(".sdlc").join(".runs")`. No path traversal risk.

**Action:** Accept — no vulnerability.

### F2 — Arbitrary file read: NONE
Only files matching `*.json` (excluding `*.events.json`) inside `.sdlc/.runs/` are read. The directory is not parameterized by user input.

**Action:** Accept — no vulnerability.

### F3 — Error output leaks internal state: LOW
Malformed run JSON files are silently skipped. This is intentional and matches the existing `load_run_history` behavior in `sdlc-server`. No sensitive information leaks to stderr.

**Action:** Accept — consistent with existing pattern, no sensitive data exposed.

### F4 — Relative time calculation: NOT APPLICABLE
Relative time is computed from `DateTime<Utc>` subtraction — no overflow risk for typical timestamps, and any unusual values (far future, far past) produce a valid string output.

**Action:** Accept.

### F5 — JSON deserialization of run files: LOW
Run files are written by `sdlc-server`. A malformed or adversarially-crafted run file would be silently skipped via `.ok()?`. No panic, no arbitrary code execution.

**Action:** Accept — handled safely.

## Security Verdict

No security issues found. The feature has minimal attack surface (read-only, local filesystem, no user-controlled paths). All findings accepted with rationale.

**APPROVED.**
