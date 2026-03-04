# Spec: dist-release-infra

## Feature

GitHub release infrastructure: create `sdlc-releases` and `homebrew-tap` repositories plus CI secrets.

## Problem

The `cargo-dist` release pipeline in CI is fully configured but cannot execute because four required GitHub resources do not exist:

1. `orchard9/sdlc-releases` — the public repository where cargo-dist uploads compiled binaries and install scripts
2. `orchard9/homebrew-tap` — the public repository where cargo-dist writes and updates the Homebrew formula
3. `GH_RELEASES_TOKEN` — a fine-grained PAT granting write access to `sdlc-releases`, set as a GitHub Actions secret on `orchard9/sdlc`
4. `HOMEBREW_TAP_TOKEN` — a fine-grained PAT granting write access to `homebrew-tap`, set as a GitHub Actions secret on `orchard9/sdlc`

Without these four resources, any `git tag vX.Y.Z && git push origin vX.Y.Z` invocation will trigger the release workflow, which will fail at the upload and tap-update steps.

## Goal

All four GitHub resources exist and are correctly configured so that tagging HEAD with a semver tag (`vX.Y.Z`) triggers a successful end-to-end cargo-dist release.

## Scope

This feature is infrastructure setup only — it involves no code changes to the `sdlc` Rust codebase. All work is performed by Jordan as four discrete manual steps in GitHub's web UI. The feature tracks those steps as tasks and verifies completion.

## Success Criteria

1. `orchard9/sdlc-releases` exists on GitHub (public, empty, managed by cargo-dist)
2. `orchard9/homebrew-tap` exists on GitHub (public, empty, managed by cargo-dist)
3. `GH_RELEASES_TOKEN` secret is set on `orchard9/sdlc` GitHub Actions with `Contents: Read and Write` on `sdlc-releases`
4. `HOMEBREW_TAP_TOKEN` secret is set on `orchard9/sdlc` GitHub Actions with `Contents: Read and Write` on `homebrew-tap`
5. A dry-run verification confirms the release workflow would succeed (CI shows secrets present, repos accessible)

## Out of Scope

- Code signing (Phase 4 — deferred)
- SBOM generation (Phase 4 — deferred)
- APT/RPM package repositories (Phase 4 — deferred)
- Gitea mirror releases (post-v1)
- Any changes to Cargo.toml or CI workflow YAML (tracked in `dist-cargo-toml-fixes`)

## Dependencies

- `dist-cargo-toml-fixes` should complete before `dist-first-release`, but not before this feature — infrastructure can be created in parallel with the code fixes

## References

- Ponder: `.sdlc/roadmap/install-distribution/prep-checklist.md` — four-step Jordan checklist
- Architecture: `.sdlc/roadmap/install-distribution/distribution-architecture.md` — cargo-dist platform matrix
- Milestone: `v25-install-distribution` — Cross-Platform Binary Distribution
