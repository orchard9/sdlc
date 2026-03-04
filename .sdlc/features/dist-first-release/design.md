# Design: dist-first-release

## Overview

This is a release operations feature, not a code change feature. There is no new Rust code to write. The design describes the sequence of operations to tag `v0.1.0`, trigger the GitHub Actions release workflow, and validate the resulting artifacts.

## Release Sequence

```
1. Pre-flight checks
   ├── Confirm Cargo workspace version = "0.1.0" (Cargo.toml)
   ├── Confirm release.yml workflow is present and correct
   └── Confirm main branch is clean and up-to-date

2. Tag and push
   ├── git tag -a v0.1.0 -m "Release v0.1.0"
   └── git push origin v0.1.0

3. GitHub Actions Release workflow executes
   ├── Build job (5 matrix targets in parallel)
   │   ├── aarch64-apple-darwin    (macos-latest)
   │   ├── x86_64-apple-darwin     (macos-13)
   │   ├── x86_64-unknown-linux-musl  (ubuntu-latest)
   │   ├── aarch64-unknown-linux-musl (ubuntu-latest, cross-compile)
   │   └── x86_64-pc-windows-msvc  (windows-latest)
   └── Release job (after all builds pass)
       └── softprops/action-gh-release creates GitHub Release with 5 assets

4. Validate
   ├── Confirm GitHub Release v0.1.0 exists with 5 attached assets
   ├── Smoke-test install.sh on local machine (macOS aarch64 or x86_64)
   └── Verify: ponder --version and sdlc --version both print "0.1.0"
```

## Binary Naming Convention

| Platform | Archive | Binary |
|---|---|---|
| macOS arm64 | `ponder-aarch64-apple-darwin.tar.gz` | `ponder` |
| macOS x86_64 | `ponder-x86_64-apple-darwin.tar.gz` | `ponder` |
| Linux x86_64 | `ponder-x86_64-unknown-linux-musl.tar.gz` | `ponder` |
| Linux arm64 | `ponder-aarch64-unknown-linux-musl.tar.gz` | `ponder` |
| Windows x86_64 | `ponder-x86_64-pc-windows-msvc.zip` | `ponder.exe` |

## Install Script Architecture

`install.sh` (Unix):
1. Detect OS and architecture → select target triple
2. Query `api.github.com/repos/orchard9/sdlc/releases/latest` for the latest tag
3. Download `ponder-<target>.tar.gz` and extract `ponder` binary
4. Install to `$PONDER_INSTALL` (default: `~/.local/bin`)
5. Create `sdlc` symlink pointing to `ponder`
6. Print PATH warning if needed

`install.ps1` (Windows):
1. Detect arch → target triple
2. Query GitHub releases API for latest tag
3. Download and expand ZIP
4. Install `ponder.exe` to `$PONDER_INSTALL` (default: `$env:USERPROFILE\.local\bin`)
5. Create `sdlc.cmd` alias
6. Add install dir to user PATH if needed

## Validation Environment

Local macOS machine (darwin). The install script will be run in a temp directory to avoid contaminating the Rust-built development binary. Validation uses `--version` flag to confirm correct version output.

## Risk Mitigation

- If any matrix target fails in CI, the tag must be deleted (`git tag -d v0.1.0 && git push origin :refs/tags/v0.1.0`) and re-pushed after fixing the build issue.
- The `api.github.com/releases/latest` endpoint returns HTTP 404 with `{"message":"Not Found"}` before any release exists. The install script correctly handles this as a fatal error.
- The `softprops/action-gh-release` action defaults to draft=false, so the release will be immediately public.

## No Code Changes Required

This feature validates existing infrastructure. If pre-flight checks reveal issues:
- Cargo version mismatch → handled by `dist-cargo-toml-fixes` feature
- Release workflow errors → handled by `dist-release-infra` feature

This feature's only deliverables are: the git tag, the resulting GitHub Release, and a verified install test.
