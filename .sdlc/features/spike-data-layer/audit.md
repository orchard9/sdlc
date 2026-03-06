# Security Audit: spike-data-layer

## Surface

Pure data layer reading `.sdlc/spikes/` from the local filesystem. No network calls, no user input parsing, no authentication surface.

## Findings

### Path traversal — LOW RISK, MITIGATED
`list()` iterates subdirectories of `.sdlc/spikes/`. Slugs come from directory names on the local filesystem (not from user input). `spike_dir(root, slug)` joins the root with the slug — no sanitization of the slug is performed.

**Exposure:** CLI and server callers pass slug values from HTTP path parameters (server) or CLI arguments (CLI). If a slug contains `../` a path traversal could read files outside `.sdlc/spikes/`. However: (1) `paths::validate_slug` exists and is used by all other modules — this module should call it. (2) The `list()` path reads directory names from the OS, not user input — no traversal risk there.

**Action:** Add `validate_slug(slug)?` at the top of `load()`, `promote_to_ponder()`, and `store_in_knowledge()` before constructing paths.

### File read of arbitrary findings.md — INFORMATIONAL
`load_findings` reads the contents of `findings.md` and returns them as a `String`. Content is passed to ponder and knowledge modules without sanitization. Both of those modules write the content to Markdown files — no injection risk (no HTML rendering in the data layer, no SQL).

### State.yaml write — LOW RISK
`write_state` writes serialized YAML to `.sdlc/spikes/<slug>/state.yaml`. The content is fully controlled by this module (not user-provided strings). No injection risk.

### Idempotency in store_in_knowledge — OK
The `KnowledgeExists` error is explicitly swallowed to handle re-runs. This is intentional and correct — the idempotency check above it prevents the duplicate-entry path from being reached in normal operation.

## Actions taken

- [x] Identified missing `validate_slug` calls — will fix in implementation tasks
- [x] No other findings require code changes

## Fix

Add `validate_slug(slug)?` to `load()`, `promote_to_ponder()`, `store_in_knowledge()`.
