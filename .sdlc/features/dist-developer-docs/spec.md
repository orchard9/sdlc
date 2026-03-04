# Spec: DEVELOPER.md — Cross-Platform Dev Install Path

## Problem

The DEVELOPER.md contributor guide must provide clear, self-sufficient installation instructions
for every supported platform (macOS, Linux, Windows) without requiring `make` or any system build
tool that isn't already available in a standard Rust development environment.

Key gaps to address:
1. Document `cargo install` as a first-class fallback path for contributors who do not have `just`.
2. Ensure `just` installation instructions are prominent and cross-platform (Homebrew, Winget, cargo).
3. Remove any implicit assumption that `make` is available.
4. Verify the existing `just` recipe table in DEVELOPER.md is complete and accurate.

## Goals

- Zero confusion for a new contributor opening DEVELOPER.md for the first time on any platform.
- At least two install paths documented: `just install` (preferred) and a manual `cargo install` path.
- Windows-specific `just` install instruction (`winget install just`) present alongside macOS/Linux.
- No reference to `make` anywhere in contributor-facing docs.

## Non-Goals

- Changing the Justfile or any Rust source code.
- Adding CI-specific instructions; focus is on local developer experience.
- Modifying QUICKSTART.md or README.md (those have separate features if needed).

## Acceptance Criteria

1. `DEVELOPER.md` contains a clearly marked "cargo install (no just)" fallback section.
2. `just` install line shows all three package managers: `cargo install just`, `brew install just`, `winget install just`.
3. A search for `make` in DEVELOPER.md returns no matches.
4. The existing `just` recipe table is reviewed and kept accurate (build, test, lint, clean).
5. Dev loop section documents both macOS/Linux and Windows variants for `cargo watch`.
