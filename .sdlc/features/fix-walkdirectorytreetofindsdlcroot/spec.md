# Spec: Fix resolve_root to Walk Directory Tree

## Problem

`crates/sdlc-cli/src/root.rs:resolve_root` falls back directly to `std::env::current_dir()` when no explicit root is provided. It does not search ancestor directories for a `.sdlc/` directory.

When `sdlc` is run from a project subdirectory (e.g., `src/`, `crates/foo/`), the CLI reports that the project is not initialized even though a valid `.sdlc/` directory exists in a parent directory. LLM agents frequently run commands from non-root directories, making this a common and disruptive failure mode.

## Proposed Solution

Change `resolve_root` to walk ancestor directories from the current working directory upward, returning the first directory that contains a `.sdlc/` subdirectory. If no such directory is found, fall back to the current working directory (preserving existing behavior for uninitialized projects).

### Algorithm

1. If an explicit root is provided (`--root` flag or `SDLC_ROOT` env var), return it as-is (existing behavior, unchanged).
2. Starting from `std::env::current_dir()`, walk up the directory tree (via `Path::parent()`).
3. At each directory, check whether a `.sdlc/` subdirectory exists.
4. Return the first directory that contains `.sdlc/`.
5. If no ancestor contains `.sdlc/`, return the current working directory (fallback, same as current behavior).

## Affected File

- `crates/sdlc-cli/src/root.rs` — `resolve_root` function and associated tests.

## Acceptance Criteria

- Running any `sdlc` command from a subdirectory of an initialized project succeeds and operates against the correct `.sdlc/` root.
- Running `sdlc` from a directory with no `.sdlc/` ancestor still returns the current directory (no regression for uninitialized projects).
- Explicit `--root` / `SDLC_ROOT` overrides are unaffected.
- Unit tests cover: explicit root, `.sdlc/` in current dir, `.sdlc/` in grandparent dir, and no `.sdlc/` found.

## Out of Scope

- Changes to CLI argument parsing or error messages.
- Symlink resolution or filesystem edge cases beyond basic `std::fs` behavior.
