# Security Audit: Server Startup Marks Orphaned Runs Failed

## Scope

Single function change in `crates/sdlc-server/src/state.rs`:
`load_run_history` — orphan recovery block during server startup.

## Security Surface Analysis

### 1. File I/O — read path

`load_run_history` reads every `.json` file in `.sdlc/.runs/`. This path existed before
this change.

**Finding:** No new attack surface introduced. The directory is a local, server-owned
path. An attacker who can write arbitrary JSON to this directory already has local file
system access — a security boundary that this function does not enforce or change.

**Action:** Accept — no change to I/O boundary.

### 2. File I/O — write path (best-effort persist on recovery)

The existing code already writes back to the `.json` file after mutating the record.
This change adds `rec.error = Some("server restarted")` to that write.

**Finding:** The write is bounded to the same file that was just read. No new file is
created; no user-controlled path is used. The `error` string is a hardcoded literal
(`"server restarted"`) — not sourced from any external input. No injection risk.

**Action:** Accept — hardcoded string, no injection surface.

### 3. Status field mutation

The change sets `rec.status = "failed"` for orphaned records. This is a purely
cosmetic/diagnostic mutation on a local file — it does not affect authentication,
authorization, or access control.

**Finding:** Run records have no security enforcement role. They are a display artifact
for the run history UI. Changing a run's status cannot escalate privilege or bypass any
security boundary.

**Action:** Accept — no security implication.

### 4. `error` field — new optional field populated on recovery

The `error: Option<String>` field already exists on `RunRecord`. This change populates
it for the orphan case. The field is serialized to local `.json` files only — not
transmitted over the network in this function.

**Finding:** No new data flows out of the server. The field is returned to UI consumers
via the existing `GET /api/runs` endpoint, but only to authenticated clients. Value is
always the literal `"server restarted"` — not user-controlled.

**Action:** Accept — controlled literal, no exfiltration or injection risk.

### 5. `unwrap()` / panic risk

The recovery block uses `serde_json::to_string_pretty(&rec).unwrap_or_default()` for
the best-effort write. `unwrap_or_default()` prevents panics. No `unwrap()` was added.

**Finding:** No new panic paths. Consistent with project's no-`unwrap()` policy.

**Action:** Accept.

## Summary

This change has no meaningful security surface. All mutations are to local disk files
using hardcoded string values, with no new file paths, no user-controlled input, and no
changes to the server's network-facing API contract. No findings require remediation.

**Verdict:** PASS
