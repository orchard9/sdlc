# Design: SSH and make install in README

## Summary

This feature is documentation-only — three targeted edits to `README.md`'s install section. No code changes, no architecture decisions, no UI. The changes are fully specified in `spec.md`.

## Changes

All changes are in `README.md`, install section:

1. **SSH URL option** — add SSH URL as labeled primary option before the HTTPS URL in the `cargo install --git` block
2. **`make install` subsection** — add a "Build from source" block with `git clone` + `make install`
3. **DEVELOPER.md link** — add a callout line at the end of the install section

## No Diagrams Required

Pure prose documentation edits — no diagrams, prototypes, or companion files needed.
