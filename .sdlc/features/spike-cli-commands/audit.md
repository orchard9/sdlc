# Security Audit: Spike CLI — list, show, promote subcommands

## Scope

`crates/sdlc-cli/src/cmd/spike.rs` — new CLI command wrapping `sdlc_core::spikes`.

## Surface Analysis

This feature adds read-only CLI commands that:
1. Read files from `.sdlc/spikes/` (already on disk, written by the spike agent)
2. Write to `.sdlc/roadmap/` (ponder) via `promote_to_ponder` (existing code path)
3. Write to `.sdlc/spikes/<slug>/state.yaml` (update only, no new attack surface)

All I/O goes through the existing `sdlc_core` data layer which uses `atomic_write` for writes.

## Findings

### F1 — Slug validation: PASS

`spikes::load` and `spikes::promote_to_ponder` call `crate::paths::validate_slug(slug)` before
any file I/O. The slug is user-supplied from the CLI argument. This prevents path traversal
attacks (e.g., `../../etc/passwd`) at the core layer.

Action: none required — validation already present.

### F2 — No shell execution: PASS

No calls to `std::process::Command` or any shell. No injection surface.

### F3 — No network calls: PASS

Purely local file I/O. No HTTP, no sockets.

### F4 — No secrets exposure: PASS

The spike findings.md is written by the spike agent and may contain technical details but no
credentials. The CLI only reads and prints what already exists on disk.

### F5 — Error messages: PASS

Error messages include the slug (user-supplied) but are formatted with `format!()`, not
interpolated into shell commands or SQL. No injection risk.

### F6 — JSON output: PASS

`print_json` uses `serde_json::to_string_pretty` — no hand-rolled serialization. No XSS
risk in CLI context.

## Verdict

APPROVED. No security findings. The implementation is a thin wrapper over existing core
functions that already have slug validation and atomic writes.
