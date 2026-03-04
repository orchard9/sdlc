# Feature Spec: dist-first-release

## Summary

Tag `v0.1.0` as the first official release of `ponder` / `sdlc`, trigger the GitHub Actions release workflow, and verify that the resulting binaries and install scripts function correctly on macOS, Linux, and Windows.

## Problem

The project has working CI infrastructure (`.github/workflows/release.yml`) and install scripts (`install.sh`, `install.ps1`), but no version tag has ever been pushed. There are no GitHub Releases yet, meaning the install scripts will fail because they query `api.github.com/repos/orchard9/sdlc/releases/latest` and get no result. Users cannot install `ponder`/`sdlc` via the documented one-liner.

## Goal

- Push the `v0.1.0` tag to trigger the release workflow.
- Confirm that the workflow succeeds and all 5 platform binaries are attached to the GitHub Release.
- Confirm the install scripts resolve the binary and execute without error on at least macOS (aarch64 or x86_64) and Linux (x86_64 musl).
- Windows install script verification is documented as a manual step or CI validation.

## Success Criteria

1. `git tag v0.1.0` exists and is pushed to `origin`.
2. GitHub Actions `Release` workflow completes with all 5 matrix targets green.
3. GitHub Release `v0.1.0` exists with 5 attached assets (`ponder-*.tar.gz` / `.zip`).
4. `curl -sSfL https://raw.githubusercontent.com/orchard9/sdlc/main/install.sh | sh` installs `ponder` and `sdlc` symlink to `~/.local/bin` and both binaries respond to `--version`.
5. The installed `ponder --version` output matches `0.1.0`.

## Out of Scope

- Homebrew tap or package manager integration (separate milestone).
- Windows native installer (PowerShell script is sufficient for v0.1.0).
- Any code changes to the binary itself — this is a release verification feature only.

## Dependencies

- `dist-cargo-toml-fixes`: workspace version must be `0.1.0` in `Cargo.toml`.
- `dist-release-infra`: GitHub Actions release workflow must be in place and `GITHUB_TOKEN` permissions must allow creating releases. (Both are already present per `release.yml` and the `7a9d44e` commit.)

## Risks

- If the `Release` workflow fails on any matrix target, the tag must be deleted and re-pushed after fixing the root cause. This feature tracks a clean first-pass success.
- The `api.github.com/repos/orchard9/sdlc/releases/latest` endpoint returns 404 if no releases exist — the install script will error before the first release is created. This is expected behavior and will be resolved the moment `v0.1.0` is published.

## Approach

1. Confirm Cargo workspace version is `0.1.0`.
2. Confirm `release.yml` workflow is correct and all targets are reachable.
3. Create and push the annotated tag `v0.1.0`.
4. Monitor the GitHub Actions run; record pass/fail per target.
5. Smoke-test the install script on the local macOS machine (aarch64 or x86_64).
6. Document results in review artifact.
