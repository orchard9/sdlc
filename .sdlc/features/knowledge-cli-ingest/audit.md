# Security Audit: knowledge-cli-ingest

## Surface

CLI commands in `sdlc-cli` and REST routes in `sdlc-server`. All data I/O delegates to `sdlc-core::knowledge` which uses `validate_slug`, `validate_code`, and atomic writes throughout.

## Path Traversal

**CLI — slug derivation:** Slugs passed to the data layer from the CLI are derived by `slugify_title()` which produces only `[a-z0-9-]` characters. These are then validated again by `sdlc_core::paths::validate_slug` inside the data layer before any filesystem operation.

**Server — slug from URL path:** Slugs come from `Path(slug): Path<String>` (axum). These are passed directly to `sdlc_core::knowledge::load()` which calls `validate_slug()` before touching the filesystem. `validate_slug` rejects `.`, `/`, `\`, and `..`.

**Verdict:** Path traversal is not possible through either surface.

## URL Fetching (--from-url)

`fetch_page_title()` uses `ureq::get()` with a 10-second timeout. The URL is provided by the CLI user (local tool user, not network attacker). The response body is read via `into_string()` (ureq default 10MB limit). Title extraction is done via string search — no eval, no subprocess execution.

**Risk:** An attacker-controlled URL could cause the CLI user to fetch unintended content. However, since this is a local developer tool and the URL is explicitly provided by the user, this is accepted scope.

**Verdict:** No injection risk; best-effort fetch is benign.

## Content Injection

All content written to `content.md` is stored as raw markdown. No templating, no eval, no shell execution. The file is never executed — only read back as plain text.

## Server Route Input Validation

- `CreateKnowledgeBody.code` passes through `knowledge::create()` → `validate_code()` before touching disk.
- `UpdateKnowledgeBody.status` is parsed via `KnowledgeStatus::from_str()` which rejects invalid values with `InvalidKnowledgeStatus`.
- All slug parameters from URL paths go through `validate_slug()` in the data layer.

## Error Handling

- `BacklogItemNotFound` was already defined in `SdlcError` but missing from the server's HTTP status match. This was fixed as a side effect — the server now correctly returns 404 for that error variant.

## Verdict

No security issues found. The CLI and server route layers add no new attack surface — they delegate all validation and I/O to the audited `sdlc-core::knowledge` module.
