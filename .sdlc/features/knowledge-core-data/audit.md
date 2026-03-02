# Security Audit: knowledge-core-data

## Surface

Pure data layer writing YAML and Markdown files to `.sdlc/knowledge/` on disk. No network I/O, no subprocess execution, no external dependencies beyond serde_yaml and regex.

## Path traversal

**Risk:** Slug could be crafted to escape the knowledge directory (e.g., `"../secrets"`).

**Mitigation:** `validate_slug()` from `paths.rs` is called in `create()` and `load()` before any filesystem operation. The slug regex `^[a-z0-9][a-z0-9\-]*[a-z0-9]$|^[a-z0-9]$` rejects `.`, `/`, `\`, and `..`. This is the same validation used for feature and investigation slugs across the codebase.

**Artifact filenames:** `capture_named_artifact` and `read_named_artifact` delegate to `workspace::write_artifact` / `workspace::read_artifact`, which call `validate_artifact_filename()` — rejecting `/`, `\`, `..`, and null bytes.

**Verdict:** Path traversal is not possible.

## Content injection

Knowledge entries store user-provided content (title, summary, tags, content.md body). This content is stored as YAML/Markdown and read back as strings. No eval, no shell execution — content is never interpreted. No injection risk.

## File write safety

All writes use `crate::io::atomic_write` (write to temp file → rename). No partial writes visible to other processes.

## Catalog and maintenance log

Same patterns as above — validated slugs, atomic writes, no shell execution.

## Verdict

No security issues found. This is a local-only file I/O module with slug validation, artifact filename validation, and atomic writes throughout.
